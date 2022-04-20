use crate::{
    platform::required_gpu_features, ConstantData, DataBuffer, DynamicData, GraphicsData, Light,
    LightData, LightId, Material, MaterialId, Mesh, MeshId, Pipeline, RenderPass,
    RenderPassDrawContext, RenderPassId, ShaderMaterialData, Texture, TextureData, TextureHandler,
    TextureId, CONSTANT_DATA_FLAGS_SUPPORT_SRGB, GRAPHICS_DATA_UID, MAX_NUM_LIGHTS,
    MAX_NUM_MATERIALS, MAX_NUM_TEXTURES,
};
use inox_log::debug_log;
use inox_math::{matrix4_to_array, Matrix4, Vector2};
use inox_messenger::MessageHubRc;
use inox_resources::{DataTypeResource, Resource};

use inox_platform::Handle;
use inox_resources::{SharedData, SharedDataRc};

use std::{
    mem::size_of,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

const DEFAULT_WIDTH: u32 = 1280;
const DEFAULT_HEIGHT: u32 = 720;

#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: Matrix4 = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum RendererState {
    Init,
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
    pub texture_handler: TextureHandler,
}

pub type RenderContextRw = Arc<RwLock<Option<RenderContext>>>;

pub trait GetRenderContext {
    fn get(&self) -> RwLockReadGuard<Option<RenderContext>>;
    fn get_mut(&self) -> RwLockWriteGuard<Option<RenderContext>>;
}

impl GetRenderContext for RenderContextRw {
    fn get(&self) -> RwLockReadGuard<Option<RenderContext>> {
        self.read().unwrap()
    }
    fn get_mut(&self) -> RwLockWriteGuard<Option<RenderContext>> {
        self.write().unwrap()
    }
}

pub struct Renderer {
    context: RenderContextRw,
    shared_data: SharedDataRc,
    message_hub: MessageHubRc,
    state: RendererState,
    constant_data: ConstantData,
    dynamic_data: DynamicData,
    constant_data_buffer: DataBuffer,
    dynamic_data_buffer: DataBuffer,
    graphics_mesh: Resource<GraphicsData>,
}
pub type RendererRw = Arc<RwLock<Renderer>>;

unsafe impl Send for Renderer {}
unsafe impl Sync for Renderer {}

impl Drop for Renderer {
    fn drop(&mut self) {
        crate::unregister_resource_types(&self.shared_data, &self.message_hub);
    }
}

impl Renderer {
    pub fn new(
        handle: &Handle,
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        _enable_debug: bool,
    ) -> Self {
        crate::register_resource_types(shared_data, message_hub);

        let graphics_mesh =
            shared_data.add_resource(message_hub, GRAPHICS_DATA_UID, GraphicsData::default());

        let render_context = Arc::new(RwLock::new(None));

        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(Self::create_render_context(
            handle.clone(),
            render_context.clone(),
        ));

        #[cfg(all(not(target_arch = "wasm32")))]
        futures::executor::block_on(Self::create_render_context(
            handle.clone(),
            render_context.clone(),
        ));

        Renderer {
            constant_data: ConstantData::default(),
            dynamic_data: DynamicData::default(),
            constant_data_buffer: DataBuffer::default(),
            dynamic_data_buffer: DataBuffer::default(),
            state: RendererState::Init,
            context: render_context,
            shared_data: shared_data.clone(),
            message_hub: message_hub.clone(),
            graphics_mesh,
        }
    }

    pub fn render_context(&self) -> &RenderContextRw {
        &self.context
    }

    pub fn check_initialization(&mut self) {
        if self.context.read().unwrap().is_none() {
            self.state = RendererState::Init;
        } else {
            self.state = RendererState::Submitted;
        }
    }

    async fn create_render_context(handle: Handle, render_context: RenderContextRw) {
        let backend = wgpu::Backends::VULKAN | wgpu::Backends::BROWSER_WEBGPU;
        let instance = wgpu::Instance::new(backend);
        let surface = unsafe { instance.create_surface(&handle) };

        let adapter =
            wgpu::util::initialize_adapter_from_env_or_default(&instance, backend, Some(&surface))
                .await
                .expect("No suitable GPU adapters found on the system!");
        let required_features = required_gpu_features();
        let limits = wgpu::Limits::default();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: required_features,
                    limits,
                },
                // Some(&std::path::Path::new("trace")), // Trace path
                None,
            )
            .await
            .expect("Failed to create device");

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        debug_log!("Surface format: {:?}", config.format);
        surface.configure(&device, &config);

        render_context.write().unwrap().replace(RenderContext {
            texture_handler: TextureHandler::create(&device),
            instance,
            device,
            adapter,
            surface,
            config,
            queue,
        });
    }

    pub fn resolution(&self) -> (u32, u32) {
        (
            self.context.get().as_ref().unwrap().config.width,
            self.context.get().as_ref().unwrap().config.height,
        )
    }

    pub fn state(&self) -> RendererState {
        self.state
    }
    pub fn change_state(&mut self, render_state: RendererState) -> &mut Self {
        self.state = render_state;
        self
    }

    pub fn update_shader_data(&mut self, view: Matrix4, proj: Matrix4, screen_size: Vector2) {
        self.constant_data.view = matrix4_to_array(view);
        self.constant_data.proj = matrix4_to_array(OPENGL_TO_WGPU_MATRIX * proj);
        self.constant_data.screen_width = screen_size.x;
        self.constant_data.screen_height = screen_size.y;
        if self
            .context
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .config
            .format
            .describe()
            .srgb
        {
            self.constant_data.flags |= CONSTANT_DATA_FLAGS_SUPPORT_SRGB;
        }
        /*
        self.binding_data
        .send_to_gpu(self.context.get().as_ref().unwrap());
        */
    }

    pub fn need_redraw(&self) -> bool {
        self.state != RendererState::Submitted
    }

    pub fn recreate(&self) {
        inox_profiler::scoped_profile!("renderer::recreate");

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
        let mut context = self.context.get_mut();
        let context = context.as_mut().unwrap();
        context.config.width = width;
        context.config.height = height;
        context.surface.configure(&context.device, &context.config);
        self.recreate();
    }

    pub fn on_texture_changed(&mut self, texture_id: &TextureId) {
        inox_profiler::scoped_profile!("renderer::on_texture_changed");
        let mut render_context = self.context.get_mut();
        let render_context = render_context.as_mut().unwrap();
        let texture_handler = &mut render_context.texture_handler;

        if let Some(texture) = self.shared_data.get_resource::<Texture>(texture_id) {
            if !texture.get().is_initialized() {
                if texture_handler.get_texture_index(texture_id) == None {
                    let width = texture.get().width();
                    let height = texture.get().height();
                    if let Some(image_data) = texture.get().image_data() {
                        texture_handler.add_image(
                            &render_context.device,
                            texture_id,
                            (width, height),
                            image_data,
                        );
                    }
                }
                if let Some(texture_data) = texture_handler.get_texture_data(texture_id) {
                    let uniform_index = self
                        .dynamic_data
                        .textures_data
                        .insert(texture_id, texture_data);
                    texture.get_mut().set_texture_data(
                        uniform_index,
                        texture_data.width(),
                        texture_data.height(),
                    );
                    //Need to update all materials that use this texture
                    self.shared_data
                        .for_each_resource_mut(|_, m: &mut Material| {
                            if m.has_texture_id(texture_id) {
                                m.mark_as_dirty();
                            }
                        });
                }
            }
        }
    }

    pub fn on_light_changed(&mut self, light_id: &LightId) {
        inox_profiler::scoped_profile!("renderer::on_light_changed");
        if let Some(light) = self.shared_data.get_resource::<Light>(light_id) {
            let uniform_index = self
                .dynamic_data
                .lights_data
                .insert(light_id, *light.get().data());
            light.get_mut().update_uniform(uniform_index as _);
        }
    }

    pub fn on_pipeline_changed(&mut self, pipeline_id: &MaterialId) {
        inox_profiler::scoped_profile!("renderer::on_pipeline_changed");
        if let Some(pipeline) = self.shared_data.get_resource::<Pipeline>(pipeline_id) {
            let vertex_format = pipeline.get().vertex_format();
            self.graphics_mesh
                .get_mut()
                .set_pipeline_vertex_format(pipeline.id(), vertex_format);
        }
    }

    pub fn on_material_changed(&mut self, material_id: &MaterialId) {
        inox_profiler::scoped_profile!("renderer::on_material_changed");
        if let Some(material) = self.shared_data.get_resource::<Material>(material_id) {
            let uniform_index = self
                .dynamic_data
                .set_material_data(material_id, &material.get());
            material.get_mut().update_uniform(uniform_index as _);
            //Need to update all meshes that use this material
            self.shared_data.for_each_resource_mut(|_, m: &mut Mesh| {
                if let Some(material) = m.material() {
                    if material.id() == material_id {
                        m.mark_as_dirty();
                    }
                }
            });
        }
    }

    pub fn on_render_pass_changed(&mut self, render_pass_id: &RenderPassId) {
        inox_profiler::scoped_profile!("renderer::on_render_pass_changed");
        let mut render_context = self.context.get_mut();
        let render_context = render_context.as_mut().unwrap();
        if let Some(render_pass) = self.shared_data.get_resource::<RenderPass>(render_pass_id) {
            if !render_pass.get().is_initialized() {
                render_pass.get_mut().init(render_context);
            }
        }
    }

    pub fn on_mesh_added(&mut self, mesh: &Resource<Mesh>) {
        inox_profiler::scoped_profile!("renderer::on_mesh_added");
        self.graphics_mesh
            .get_mut()
            .update_mesh(mesh.id(), &mesh.get());
    }
    pub fn on_mesh_changed(&mut self, mesh_id: &MeshId) {
        inox_profiler::scoped_profile!("renderer::on_mesh_changed");
        if let Some(mesh) = self.shared_data.get_resource::<Mesh>(mesh_id) {
            self.graphics_mesh
                .get_mut()
                .update_mesh(mesh.id(), &mesh.get());
        }
    }
    pub fn on_mesh_removed(&mut self, mesh_id: &MeshId) {
        inox_profiler::scoped_profile!("renderer::on_mesh_removed");
        self.graphics_mesh.get_mut().remove_mesh(mesh_id);
    }

    fn set_constant_data(&mut self) {
        let render_context = self.context.get();
        let render_context = render_context.as_ref().unwrap();

        let usage = wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST;

        self.constant_data_buffer.init_from_type::<ConstantData>(
            render_context,
            size_of::<ConstantData>() as _,
            usage,
        );
        self.constant_data_buffer
            .add_to_gpu_buffer(render_context, &[self.constant_data]);
    }
    fn set_dynamic_data(&mut self) {
        let render_context = self.context.get();
        let render_context = render_context.as_ref().unwrap();

        let usage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;

        let total_size = size_of::<TextureData>() * MAX_NUM_TEXTURES
            + size_of::<ShaderMaterialData>() * MAX_NUM_MATERIALS
            + size_of::<LightData>() * MAX_NUM_LIGHTS;

        self.dynamic_data_buffer.init_from_type::<DynamicData>(
            render_context,
            total_size as _,
            usage,
        );

        self.dynamic_data_buffer
            .add_to_gpu_buffer(render_context, self.dynamic_data.textures_data.data());
        self.dynamic_data_buffer
            .add_to_gpu_buffer(render_context, self.dynamic_data.materials_data.data());
        self.dynamic_data_buffer
            .add_to_gpu_buffer(render_context, self.dynamic_data.lights_data.data());
    }

    fn prepare(&mut self) {
        inox_profiler::scoped_profile!("renderer::prepare");

        self.set_constant_data();
        self.set_dynamic_data();

        let render_context = self.context.get();
        let render_context = render_context.as_ref().unwrap();

        let mut render_format = &render_context.config.format;
        let texture_handler = &render_context.texture_handler;

        self.shared_data
            .for_each_resource_mut(|_id, r: &mut RenderPass| {
                let graphics_mesh = &mut self.graphics_mesh.get_mut();

                if let Some(texture) = r.render_target() {
                    if let Some(atlas) = texture_handler.get_texture_atlas(texture.id()) {
                        render_format = atlas.texture_format();
                    }
                } else {
                    render_format = &render_context.config.format;
                }

                r.prepare(
                    render_context,
                    graphics_mesh,
                    render_format,
                    texture_handler.bind_group_layout(),
                    &self.constant_data_buffer,
                    &self.dynamic_data_buffer,
                );
            });
    }

    pub fn draw(&self) {
        inox_profiler::scoped_profile!("renderer::draw");

        if let Ok(output) = self
            .context
            .get()
            .as_ref()
            .unwrap()
            .surface
            .get_current_texture()
        {
            let screen_view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut encoder = self
                .context
                .get()
                .as_ref()
                .unwrap()
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

            {
                let mut render_target = &screen_view;
                let render_context = self.context.get();
                let render_context = render_context.as_ref().unwrap();
                let texture_handler = &render_context.texture_handler;

                let mut render_format = &render_context.config.format;
                self.shared_data
                    .for_each_resource_mut(|_id, r: &mut RenderPass| {
                        let texture_bind_group = texture_handler.bind_group(
                            &render_context.device,
                            r.render_target().as_ref().map(|t| t.id()),
                        );

                        if let Some(texture) = r.render_target() {
                            if let Some(atlas) = texture_handler.get_texture_atlas(texture.id()) {
                                render_target = atlas.texture();
                                render_format = atlas.texture_format();
                            }
                        } else {
                            render_target = &screen_view;
                            render_format = &render_context.config.format;
                        }

                        r.draw(RenderPassDrawContext {
                            context: render_context,
                            encoder: &mut encoder,
                            texture_view: render_target,
                            format: render_format,
                            graphics_mesh: &self.graphics_mesh,
                            texture_bind_group: &texture_bind_group,
                        });
                    });
            }

            self.context
                .get()
                .as_ref()
                .unwrap()
                .queue
                .submit(std::iter::once(encoder.finish()));
            output.present();
        } else {
            eprintln!("Error drawing on screen");
        }
    }

    pub fn send_to_gpu(&mut self) {
        inox_profiler::scoped_profile!("renderer::send_to_gpu");
        {
            let mut render_context = self.context.get_mut();
            let render_context = render_context.as_mut().unwrap();
            let texture_handler = &mut render_context.texture_handler;
            texture_handler.send_to_gpu(&render_context.queue);
        }
        self.prepare();
        {
            let render_context = self.context.get();
            let render_context = render_context.as_ref().unwrap();
            let graphics_mesh = &mut self.graphics_mesh.get_mut();
            graphics_mesh.send_to_gpu(render_context);
        }
    }
}
