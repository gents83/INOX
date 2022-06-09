use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use inox_math::{Matrix4, Vector2};
use inox_platform::Handle;

use inox_uid::Uid;

use crate::{
    platform::{platform_limits, required_gpu_features},
    AsBinding, ConstantData, GpuBuffer, MeshId, RenderBuffers, RenderPass, RenderPipelineId,
    RendererRw, TextureData, TextureHandler, TextureId, CONSTANT_DATA_FLAGS_SUPPORT_SRGB,
    DEFAULT_HEIGHT, DEFAULT_WIDTH,
};

#[derive(Default)]
pub struct BindingDataBuffer {
    pub buffers: RwLock<HashMap<Uid, GpuBuffer>>,
}

impl BindingDataBuffer {
    pub fn has_buffer(&self, uid: &Uid) -> bool {
        self.buffers.read().unwrap().contains_key(uid)
    }
    pub fn bind_buffer_with_id<T>(
        &self,
        id: &Uid,
        data: &mut T,
        usage: wgpu::BufferUsages,
        render_core_context: &RenderCoreContext,
    ) -> bool
    where
        T: AsBinding,
    {
        let mut bind_data_buffer = self.buffers.write().unwrap();
        let buffer = bind_data_buffer
            .entry(*id)
            .or_insert_with(GpuBuffer::default);

        let mut is_changed = false;
        if data.is_dirty() {
            let typename = std::any::type_name::<T>()
                .split(':')
                .collect::<Vec<&str>>()
                .last()
                .unwrap()
                .to_string();
            let label = format!("{}[{}]", typename, id);
            is_changed |= buffer.init(render_core_context, data.size(), usage, label.as_str());
            data.fill_buffer(render_core_context, buffer);
            data.set_dirty(false);
        }

        is_changed
    }
    pub fn bind_buffer<T>(
        &self,
        data: &mut T,
        usage: wgpu::BufferUsages,
        render_core_context: &RenderCoreContext,
    ) -> (Uid, bool)
    where
        T: AsBinding,
    {
        let id = T::id();
        let is_changed = self.bind_buffer_with_id(&id, data, usage, render_core_context);
        (id, is_changed)
    }
}

pub struct RenderCoreContext {
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface,
    pub config: wgpu::SurfaceConfiguration,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl RenderCoreContext {
    pub fn new_encoder(&self) -> wgpu::CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            })
    }
    pub fn submit(&self, encoder: wgpu::CommandEncoder) {
        self.queue.submit(std::iter::once(encoder.finish()));
    }
    pub fn set_surface_size(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        inox_log::debug_log!("Surface size: {}x{}", width, height);
    }
}

pub struct RenderContext {
    pub core: RenderCoreContext,
    pub surface_texture: Option<wgpu::SurfaceTexture>,
    pub surface_view: Option<wgpu::TextureView>,
    pub constant_data: ConstantData,
    pub texture_handler: TextureHandler,
    pub binding_data_buffer: BindingDataBuffer,
    pub render_buffers: RenderBuffers,
    pub pipeline_meshes: HashMap<RenderPipelineId, Vec<MeshId>>,
}

pub type RenderContextRw = Arc<RwLock<RenderContext>>;

impl RenderContext {
    pub async fn create_render_context(handle: Handle, renderer: RendererRw) {
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

        let render_core_context = RenderCoreContext {
            instance,
            surface,
            config,
            adapter,
            device,
            queue,
        };

        renderer
            .write()
            .unwrap()
            .set_render_context(Arc::new(RwLock::new(RenderContext {
                texture_handler: TextureHandler::create(&render_core_context.device),
                core: render_core_context,
                surface_texture: None,
                surface_view: None,
                constant_data: ConstantData::default(),
                binding_data_buffer: BindingDataBuffer::default(),
                render_buffers: RenderBuffers::default(),
                pipeline_meshes: HashMap::default(),
            })));
    }

    pub fn update_constant_data(&mut self, view: Matrix4, proj: Matrix4, screen_size: Vector2) {
        let mut is_changed = false;
        is_changed |= self.constant_data.update(view, proj, screen_size);
        if self.core.config.format.describe().srgb {
            is_changed |= self
                .constant_data
                .add_flag(CONSTANT_DATA_FLAGS_SUPPORT_SRGB);
        } else {
            is_changed |= self
                .constant_data
                .remove_flag(CONSTANT_DATA_FLAGS_SUPPORT_SRGB);
        }
        if is_changed {
            self.constant_data.set_dirty(true);
        }
    }

    pub fn resolution(&self) -> (u32, u32) {
        (self.core.config.width, self.core.config.height)
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
        let mut render_format = &self.core.config.format;

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

    pub fn add_image(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        texture_id: &TextureId,
        dimensions: (u32, u32),
        image_data: &[u8],
    ) -> TextureData {
        self.texture_handler.add_image(
            &self.core.device,
            encoder,
            texture_id,
            dimensions,
            image_data,
        )
    }
}
