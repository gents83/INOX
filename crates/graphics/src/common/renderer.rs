use crate::{
    GraphicsMesh, Light, LightId, Material, MaterialId, Mesh, MeshId, Pipeline, RenderPass,
    RenderPassDrawContext, ShaderData, Texture, TextureHandler,
};
use sabi_math::{matrix4_to_array, Matrix4, Vector2};
use sabi_resources::{DataTypeResource, HashIndexer};

use sabi_platform::Handle;
use sabi_resources::{SharedData, SharedDataRc};

use std::sync::{Arc, RwLock};

const DEFAULT_WIDTH: u32 = 1920;
const DEFAULT_HEIGHT: u32 = 1080;

#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: Matrix4 = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum RendererState {
    Preparing,
    Prepared,
    Drawing,
    Submitted,
}

pub struct RenderContext {
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface,
    pub config: wgpu::SurfaceConfiguration,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

pub struct Renderer {
    context: RenderContext,
    shared_data: SharedDataRc,
    texture_handler: TextureHandler,
    state: RendererState,
    graphics_mesh: GraphicsMesh,
    material_hash_indexer: HashIndexer<MaterialId>,
    texture_hash_indexer: HashIndexer<MaterialId>,
    light_hash_indexer: HashIndexer<LightId>,
    shader_data: ShaderData,
}
pub type RendererRw = Arc<RwLock<Renderer>>;

unsafe impl Send for Renderer {}
unsafe impl Sync for Renderer {}

impl Renderer {
    pub fn new(handle: &Handle, shared_data: &SharedDataRc, _enable_debug: bool) -> Self {
        let render_context = futures::executor::block_on(Self::create_render_context(handle));
        let texture_handler = TextureHandler::create(&render_context);

        Renderer {
            texture_handler,
            shader_data: ShaderData::new(&render_context),
            graphics_mesh: GraphicsMesh::default(),
            texture_hash_indexer: HashIndexer::default(),
            material_hash_indexer: HashIndexer::default(),
            light_hash_indexer: HashIndexer::default(),
            state: RendererState::Submitted,
            context: render_context,
            shared_data: shared_data.clone(),
        }
    }

    async fn create_render_context(handle: &Handle) -> RenderContext {
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(handle) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let required_features = wgpu::Features::all_webgpu_mask()
            | wgpu::Features::POLYGON_MODE_LINE
            | wgpu::Features::INDIRECT_FIRST_INSTANCE
            | wgpu::Features::UNSIZED_BINDING_ARRAY
            | wgpu::Features::TEXTURE_BINDING_ARRAY
            | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
            | wgpu::Features::SPIRV_SHADER_PASSTHROUGH
            | wgpu::Features::MULTI_DRAW_INDIRECT
            | wgpu::Features::MULTI_DRAW_INDIRECT_COUNT;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: required_features,
                    limits: wgpu::Limits::default(),
                },
                // Some(&std::path::Path::new("trace")), // Trace path
                None,
            )
            .await
            .unwrap();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);
        RenderContext {
            instance,
            device,
            adapter,
            surface,
            config,
            queue,
        }
    }

    pub fn context(&self) -> &RenderContext {
        &self.context
    }

    pub fn state(&self) -> RendererState {
        self.state
    }
    pub fn change_state(&mut self, render_state: RendererState) -> &mut Self {
        self.state = render_state;
        self
    }

    pub fn prepare_frame(&mut self) -> &mut Self {
        sabi_profiler::scoped_profile!("renderer::prepare_frame");

        self.init_render_passes();
        self.init_materials();
        self.init_meshes();
        self.init_textures();
        self.init_lights();

        self.send_to_gpu();
        self
    }

    pub fn update_shader_data(&mut self, view: Matrix4, proj: Matrix4, screen_size: Vector2) {
        let constant_data = self.shader_data.constant_data_mut();
        constant_data.view = matrix4_to_array(view);
        constant_data.proj = matrix4_to_array(OPENGL_TO_WGPU_MATRIX * proj);
        constant_data.screen_width = screen_size.x;
        constant_data.screen_height = screen_size.y;
        self.shader_data.send_to_gpu(&self.context);
    }

    pub fn get_texture_handler(&self) -> &TextureHandler {
        &self.texture_handler
    }
    pub fn get_texture_handler_mut(&mut self) -> &mut TextureHandler {
        &mut self.texture_handler
    }

    pub fn need_redraw(&self) -> bool {
        self.state != RendererState::Submitted
    }

    pub fn recreate(&self) {
        sabi_profiler::scoped_profile!("renderer::recreate");

        SharedData::for_each_resource_mut(&self.shared_data, |_id, pipeline: &mut Pipeline| {
            pipeline.invalidate();
        });
        SharedData::for_each_resource_mut(
            &self.shared_data,
            |_id, render_pass: &mut RenderPass| {
                render_pass.invalidate();
            },
        );
    }

    pub fn set_surface_size(&mut self, width: u32, height: u32) {
        self.context.config.width = width;
        self.context.config.height = height;
        self.context
            .surface
            .configure(&self.context.device, &self.context.config);
        self.recreate();
    }

    pub fn on_mesh_removed(&mut self, mesh_id: &MeshId) {
        self.graphics_mesh.remove_mesh(mesh_id);
    }

    pub fn draw(&self) {
        if let Ok(output) = self.context.surface.get_current_texture() {
            let screen_view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut encoder =
                self.context
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Render Encoder"),
                    });

            {
                let debug_should_draw_only_first = false;
                let mut index = 0;
                let graphics_mesh = &self.graphics_mesh;

                let bind_group_layouts = vec![
                    self.shader_data.bind_group_layout(),
                    self.texture_handler.bind_group_layout(),
                ];
                let mut render_target = &screen_view;
                let mut render_format = &self.context.config.format;
                self.shared_data
                    .for_each_resource_mut(|_id, r: &mut RenderPass| {
                        if !debug_should_draw_only_first || index == 0 {
                            let texture_bind_group = self.texture_handler.bind_group(
                                &self.context,
                                r.render_target().as_ref().map(|t| t.id()),
                            );
                            let bind_group =
                                vec![self.shader_data.bind_group(), &texture_bind_group];

                            if let Some(texture) = r.render_target() {
                                if let Some(atlas) =
                                    self.texture_handler.get_texture_atlas(texture.id())
                                {
                                    render_target = atlas.texture();
                                    render_format = atlas.texture_format();
                                }
                            } else {
                                render_target = &screen_view;
                                render_format = &self.context.config.format;
                            }

                            r.draw(RenderPassDrawContext {
                                context: &self.context,
                                encoder: &mut encoder,
                                texture_view: render_target,
                                format: render_format,
                                graphics_mesh,
                                bind_groups: bind_group.as_slice(),
                                bind_group_layouts: bind_group_layouts.as_slice(),
                            });
                        }
                        index += 1;
                    });
            }

            self.context.queue.submit(std::iter::once(encoder.finish()));
            output.present();
        } else {
            eprintln!("Error drawing on screen");
        }
    }
}

impl Renderer {
    fn init_render_passes(&mut self) {
        sabi_profiler::scoped_profile!("renderer::init_render_passes");
        let render_context = &self.context;
        let texture_handler = &mut self.texture_handler;
        self.shared_data
            .for_each_resource_mut(|_h, r: &mut RenderPass| {
                if !r.is_initialized() {
                    r.init(render_context, texture_handler);
                }
            });
    }
    fn init_meshes(&mut self) {
        sabi_profiler::scoped_profile!("renderer::init_meshes");

        let graphic_mesh = &mut self.graphics_mesh;
        self.shared_data
            .for_each_resource_mut(|handle, m: &mut Mesh| {
                if m.is_dirty() {
                    if m.is_visible() && !m.mesh_data().vertices.is_empty() {
                        //println!("Adding Mesh {:?}", handle.id());
                        if graphic_mesh.add_mesh(handle.id(), m) {
                            m.init();
                        }
                    } else {
                        //println!("Removing Mesh {:?}", handle.id());
                        graphic_mesh.remove_mesh(handle.id());
                        m.invalidate();
                        m.init();
                    }
                }
            });
    }
    fn init_textures(&mut self) {
        sabi_profiler::scoped_profile!("renderer::init_textures");
        let render_context = &self.context;
        let texture_handler = &mut self.texture_handler;
        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Update Encoder"),
                });
        let mut is_dirty = false;
        self.shared_data
            .for_each_resource_mut(|handle, texture: &mut Texture| {
                if !texture.is_initialized() {
                    is_dirty = true;
                    if texture_handler.get_texture_index(handle.id()) == None {
                        let width = texture.width();
                        let height = texture.height();
                        if let Some(image_data) = texture.image_data() {
                            texture_handler.add_image(
                                render_context,
                                &mut encoder,
                                handle.id(),
                                (width, height),
                                image_data,
                            );
                        }
                    }
                    if let Some(texture_data) = texture_handler.get_texture_data(handle.id()) {
                        let uniform_index = self.texture_hash_indexer.insert(handle.id());
                        self.shader_data.textures_data_mut()[uniform_index] = texture_data;
                        texture.set_texture_data(
                            uniform_index,
                            texture_data.total_width(),
                            texture_data.total_height(),
                        );
                    }
                }
            });
        if is_dirty {
            render_context
                .queue
                .submit(std::iter::once(encoder.finish()));
        }
    }
    fn init_materials(&mut self) {
        sabi_profiler::scoped_profile!("renderer::init_materials");
        self.shared_data
            .for_each_resource_mut(|handle, material: &mut Material| {
                let uniform_index = self.material_hash_indexer.insert(handle.id());
                material.update_uniform(
                    uniform_index as _,
                    &mut self.shader_data.material_data_mut()[uniform_index],
                );
            });
    }

    fn init_lights(&mut self) {
        sabi_profiler::scoped_profile!("renderer::init_lights");
        self.shared_data
            .for_each_resource_mut(|handle, light: &mut Light| {
                let uniform_index = self.light_hash_indexer.insert(handle.id());
                light.update_uniform(
                    uniform_index as _,
                    &mut self.shader_data.light_data_mut()[uniform_index],
                );
            });
    }

    fn send_to_gpu(&mut self) {
        sabi_profiler::scoped_profile!("renderer::send_to_gpu");
        let render_context = &self.context;
        let graphic_mesh = &mut self.graphics_mesh;

        graphic_mesh.send_to_gpu(render_context);
    }
}
