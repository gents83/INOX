use crate::{
    utils::to_u8_slice, ConstantData, GraphicsMesh, LightData, Mesh, Pipeline, RenderPass,
    ShaderMaterialData, ShaderTextureData, Texture, TextureHandler,
};
use sabi_math::{matrix4_to_array, Matrix4};
use sabi_resources::DataTypeResource;

use sabi_platform::Handle;
use sabi_resources::{SharedData, SharedDataRc};
use wgpu::util::DeviceExt;

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
    light_data: Vec<LightData>,
    texture_data: Vec<ShaderTextureData>,
    material_data: Vec<ShaderMaterialData>,
    state: RendererState,
    graphics_mesh: GraphicsMesh,
    constant_data: ConstantData,
    constant_data_buffer: wgpu::Buffer,
    constant_data_bind_group_layout: wgpu::BindGroupLayout,
    constant_data_bind_group: wgpu::BindGroup,
}
pub type RendererRw = Arc<RwLock<Renderer>>;

unsafe impl Send for Renderer {}
unsafe impl Sync for Renderer {}

impl Renderer {
    pub fn new(handle: &Handle, shared_data: &SharedDataRc, _enable_debug: bool) -> Self {
        let render_context = futures::executor::block_on(Self::create_render_context(handle));
        let texture_handler = TextureHandler::create(&render_context);
        let constant_data = ConstantData::default();
        let constant_data_buffer =
            render_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("ConstantData Buffer"),
                    contents: to_u8_slice(&[constant_data]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let constant_data_bind_group_layout =
            render_context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("constant_data_bind_group_layout"),
                });

        let constant_data_bind_group =
            render_context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &constant_data_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: constant_data_buffer.as_entire_binding(),
                    }],
                    label: Some("constant_data_bind_group"),
                });
        Renderer {
            shared_data: shared_data.clone(),
            context: render_context,
            texture_handler,
            light_data: Vec::new(),
            texture_data: Vec::new(),
            material_data: Vec::new(),
            state: RendererState::Submitted,
            graphics_mesh: GraphicsMesh::default(),
            constant_data,
            constant_data_buffer,
            constant_data_bind_group_layout,
            constant_data_bind_group,
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

    pub fn light_data(&self) -> &[LightData] {
        self.light_data.as_slice()
    }
    pub fn texture_data(&self) -> &[ShaderTextureData] {
        self.texture_data.as_slice()
    }
    pub fn material_data(&self) -> &[ShaderMaterialData] {
        self.material_data.as_slice()
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

        self.init_pipelines();
        self.init_meshes();
        self.init_textures();
        /*
        self.init_materials();
        self.init_lights();
        */

        self.send_to_gpu();
        self
    }

    pub fn update_constant(&mut self, view: Matrix4, proj: Matrix4) {
        self.constant_data.view = matrix4_to_array(view);
        self.constant_data.proj = matrix4_to_array(OPENGL_TO_WGPU_MATRIX * proj);

        self.context.queue.write_buffer(
            &self.constant_data_buffer,
            0,
            to_u8_slice(&[self.constant_data]),
        );
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
            |_id, render_pass: &mut RenderPass| render_pass.invalidate(),
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

    pub fn draw(&self) {
        if let Ok(output) = self.context.surface.get_current_texture() {
            let view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut encoder =
                self.context
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Render Encoder"),
                    });

            {
                let debug_should_draw_only_first = true;
                let mut index = 0;
                let graphics_mesh = &self.graphics_mesh;
                self.shared_data
                    .for_each_resource_mut(|_id, r: &mut RenderPass| {
                        if !debug_should_draw_only_first || index == 0 {
                            r.draw(
                                &mut encoder,
                                &view,
                                graphics_mesh,
                                &self.constant_data_bind_group,
                            );
                        }
                        index += 1;
                    });
            }

            self.context.queue.submit(std::iter::once(encoder.finish()));
            output.present();
        } else {
            eprintln!("Error drawing");
        }
    }
}

impl Renderer {
    fn init_pipelines(&mut self) {
        sabi_profiler::scoped_profile!("renderer::init_pipelines");
        let render_context = &self.context;
        let constant_data_bind_group_layout = &self.constant_data_bind_group_layout;
        self.shared_data
            .for_each_resource_mut(|_h, p: &mut Pipeline| {
                if !p.is_initialized() {
                    p.init(render_context, constant_data_bind_group_layout);
                }
            });
    }
    fn init_meshes(&mut self) {
        sabi_profiler::scoped_profile!("renderer::init_meshes");

        let graphic_mesh = &mut self.graphics_mesh;
        self.shared_data
            .for_each_resource_mut(|handle, m: &mut Mesh| {
                if !m.is_initialized() {
                    graphic_mesh.add_mesh(handle.id(), m);
                } else {
                    graphic_mesh.update_mesh(handle.id(), m);
                }
            });
    }
    fn init_textures(&mut self) {
        sabi_profiler::scoped_profile!("renderer::init_textures");
        let render_context = &self.context;
        let texture_handler = &mut self.texture_handler;
        let shared_data = &self.shared_data;

        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Update Encoder"),
                });

        SharedData::for_each_resource_mut(shared_data, |texture_handle, texture: &mut Texture| {
            if !texture.is_initialized() {
                if let Some(texture_data) = texture_handler.get_texture_data(texture_handle.id()) {
                    let uniform_index = self.texture_data.len();
                    texture.set_texture_data(
                        uniform_index,
                        texture_data.get_width(),
                        texture_data.get_height(),
                    );
                    self.texture_data.push(texture_data);
                } else {
                    let width = texture.width();
                    let height = texture.height();
                    if let Some(image_data) = texture.image_data() {
                        let texture_data = texture_handler.add_image(
                            render_context,
                            &mut encoder,
                            texture_handle.id(),
                            (width, height),
                            image_data,
                        );
                        let uniform_index = self.texture_data.len();
                        texture.set_texture_data(
                            uniform_index,
                            texture_data.get_width(),
                            texture_data.get_height(),
                        );
                        self.texture_data.push(texture_data);
                    }
                }
            }
        });

        render_context
            .queue
            .submit(std::iter::once(encoder.finish()));
    }
    /*
    fn init_lights(&mut self) {
        sabi_profiler::scoped_profile!("renderer::init_lights");
        self.light_data.clear();
        self.shared_data.for_each_resource(|_id, light: &Light| {
            if light.is_active() {
                self.light_data.push(*light.data());
            }
        });
    }
    fn init_materials(&mut self) {
        sabi_profiler::scoped_profile!("renderer::init_materials");
        self.shared_data
            .for_each_resource_mut(|_id, material: &mut Material| {
                if !material.is_initialized() {
                    let uniform_index = self.material_data.len() as i32;
                    self.material_data
                        .push(material.create_uniform_material_data());
                    material.set_uniform_index(uniform_index);
                }
            });
    }
    */

    fn send_to_gpu(&mut self) {
        sabi_profiler::scoped_profile!("renderer::send_to_gpu");
        let render_context = &self.context;
        let graphic_mesh = &mut self.graphics_mesh;

        graphic_mesh.send_to_gpu(render_context);

        self.shared_data
            .for_each_resource_mut(|_h, p: &mut Pipeline| {
                if p.is_initialized() {
                    p.send_to_gpu(render_context);
                }
            });
    }
}
