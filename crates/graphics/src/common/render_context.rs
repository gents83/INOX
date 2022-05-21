use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use inox_core::ContextRc;
use inox_math::{Matrix4, Vector2};
use inox_platform::Handle;
use inox_resources::Resource;
use inox_uid::Uid;

use crate::{
    platform::{platform_limits, required_gpu_features},
    AsBufferBinding, ConstantData, DataBuffer, DynamicData, GraphicsData, LightData, LightId,
    Material, RenderPass, TextureData, TextureHandler, TextureId, CONSTANT_DATA_FLAGS_SUPPORT_SRGB,
    DEFAULT_HEIGHT, DEFAULT_WIDTH, GRAPHICS_DATA_UID,
};

pub struct RenderContext {
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface,
    pub config: wgpu::SurfaceConfiguration,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_texture: Option<wgpu::SurfaceTexture>,
    pub surface_view: Option<wgpu::TextureView>,
    pub constant_data: ConstantData,
    pub dynamic_data: DynamicData,
    pub texture_handler: TextureHandler,
    pub graphics_data: Resource<GraphicsData>,
    pub bind_data_buffer: RwLock<HashMap<Uid, (DataBuffer, bool)>>,
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

impl RenderContext {
    pub async fn create_render_context(
        handle: Handle,
        app_context: ContextRc,
        render_context: RenderContextRw,
    ) {
        let backend = wgpu::Backends::all();
        let instance = wgpu::Instance::new(backend);
        let surface = unsafe { instance.create_surface(&handle) };

        let adapter =
            wgpu::util::initialize_adapter_from_env_or_default(&instance, backend, Some(&surface))
                .await
                .expect("No suitable GPU adapters found on the system!");
        let required_features = required_gpu_features();
        let limits = platform_limits();

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

        //debug_log!("Surface format: {:?}", config.format);
        surface.configure(&device, &config);

        let graphics_data = app_context.shared_data().add_resource(
            app_context.message_hub(),
            GRAPHICS_DATA_UID,
            GraphicsData::default(),
        );

        render_context.write().unwrap().replace(RenderContext {
            texture_handler: TextureHandler::create(&device),
            instance,
            device,
            adapter,
            surface,
            config,
            queue,
            surface_texture: None,
            surface_view: None,
            constant_data: ConstantData::default(),
            dynamic_data: DynamicData::default(),
            graphics_data,
            bind_data_buffer: RwLock::new(HashMap::new()),
        });
    }

    pub fn new_encoder(&self) -> wgpu::CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            })
    }
    pub fn submit(&self, encoder: wgpu::CommandEncoder) {
        self.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn update_constant_data(&mut self, view: Matrix4, proj: Matrix4, screen_size: Vector2) {
        let mut is_changed = false;
        is_changed |= self.constant_data.update(view, proj, screen_size);
        if self.config.format.describe().srgb {
            is_changed |= self
                .constant_data
                .set_flags(CONSTANT_DATA_FLAGS_SUPPORT_SRGB);
        }
        if is_changed {
            self.constant_data.mark_as_dirty(self);
        }
    }
    pub fn add_texture_data(&mut self, id: &TextureId, data: TextureData) -> usize {
        let uniform_index = self.dynamic_data.add_texture_data(id, data);
        self.dynamic_data.mark_as_dirty(self);
        uniform_index
    }
    pub fn add_light_data(&mut self, id: &LightId, data: LightData) -> usize {
        let uniform_index = self.dynamic_data.add_light_data(id, data);
        self.dynamic_data.mark_as_dirty(self);
        uniform_index
    }
    pub fn add_material_data(&mut self, id: &LightId, material: &Material) -> usize {
        let uniform_index = self.dynamic_data.add_material_data(id, material);
        self.dynamic_data.mark_as_dirty(self);
        uniform_index
    }

    pub fn is_buffer_dirty<T>(&self) -> bool
    where
        T: AsBufferBinding,
    {
        let id = T::id();

        let bind_data_buffer = self.bind_data_buffer.read().unwrap();
        if let Some(entry) = bind_data_buffer.get(&id) {
            return entry.1;
        }
        true
    }

    pub fn mark_buffer_as_dirty<T>(&self)
    where
        T: AsBufferBinding,
    {
        let id = T::id();

        let mut bind_data_buffer = self.bind_data_buffer.write().unwrap();
        let entry = bind_data_buffer
            .entry(id)
            .or_insert((DataBuffer::default(), false));
        entry.1 = false;
    }

    pub fn bind_buffer<T>(&self, data: &T, usage: wgpu::BufferUsages) -> Uid
    where
        T: AsBufferBinding,
    {
        let id = T::id();

        let mut bind_data_buffer = self.bind_data_buffer.write().unwrap();
        let entry = bind_data_buffer
            .entry(id)
            .or_insert((DataBuffer::default(), false));

        if !entry.1 {
            entry.1 = true;
            entry.0.init_from_type::<T>(self, data.size(), usage);
            data.fill_buffer(self, &mut entry.0);
        }

        id
    }

    pub fn prepare_bindings(&self) {
        self.bind_data_buffer
            .write()
            .unwrap()
            .iter_mut()
            .for_each(|(_, (_buffer, is_dirty))| {
                *is_dirty = false;
            });
    }

    pub fn render_target<'a>(&'a self, render_pass: &'a RenderPass) -> &'a wgpu::TextureView {
        if let Some(texture) = render_pass.render_texture() {
            if let Some(atlas) = self.texture_handler.get_texture_atlas(texture.id()) {
                return atlas.texture();
            }
        }
        debug_assert!(self.surface_view.is_some());
        self.surface_view.as_ref().unwrap()
    }

    pub fn depth_target<'a>(
        &'a self,
        render_pass: &'a RenderPass,
    ) -> Option<&'a wgpu::TextureView> {
        if let Some(texture) = render_pass.depth_texture() {
            if let Some(atlas) = self.texture_handler.get_texture_atlas(texture.id()) {
                return Some(atlas.texture());
            }
        }
        None
    }

    pub fn render_format(&self, render_pass: &RenderPass) -> &wgpu::TextureFormat {
        let mut render_format = &self.config.format;

        if let Some(texture) = render_pass.render_texture() {
            if let Some(atlas) = self.texture_handler.get_texture_atlas(texture.id()) {
                render_format = atlas.texture_format();
            }
        }
        render_format
    }

    pub fn depth_format(&self, render_pass: &RenderPass) -> Option<&wgpu::TextureFormat> {
        if let Some(texture) = render_pass.depth_texture() {
            self.texture_handler
                .get_texture_atlas(texture.id())
                .map(|atlas| atlas.texture_format())
        } else {
            None
        }
    }
}
