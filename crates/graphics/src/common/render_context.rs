use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use inox_core::ContextRc;
use inox_messenger::MessageHubRc;
use inox_platform::Handle;
use inox_resources::{Resource, ResourceTrait, SharedDataRc};

use crate::{
    platform::{platform_limits, required_gpu_features, setup_env},
    BindingDataBuffer, BindingDataBufferRc, BufferId, ComputePipeline, DrawCommandType,
    GlobalBuffers, GpuBuffer, Material, MeshFlags, Pass, RenderPass, RenderPipeline, Texture,
    TextureFormat, TextureHandler, TextureHandlerRc, TextureId, TextureUsage, DEFAULT_HEIGHT,
    DEFAULT_WIDTH,
};

const USE_FORCED_VULKAN: bool = false;

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum RendererState {
    Preparing,
    Prepared,
    Drawing,
    Submitted,
    Rendered,
}

pub struct CommandBuffer {
    pub encoder: wgpu::CommandEncoder,
}

pub struct WebGpuContext {
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface<'static>,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: RwLock<wgpu::SurfaceConfiguration>,
}

pub type WebGpuContextRc = Arc<WebGpuContext>;

impl WebGpuContext {
    pub fn new_command_buffer(&self) -> CommandBuffer {
        inox_profiler::scoped_profile!("render_context::new_command_buffer");
        CommandBuffer {
            encoder: self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Command Encoder"),
                }),
        }
    }
    pub fn submit(&self, command_buffer: CommandBuffer) {
        inox_profiler::scoped_profile!("render_context::submit");
        let command_buffer = command_buffer.encoder.finish();
        self.queue.submit(std::iter::once(command_buffer));
    }
    pub fn configure(&self) {
        inox_profiler::scoped_profile!("render_context::configure");
        self.surface
            .configure(&self.device, &self.config.read().unwrap());
    }
}

struct SurfaceData {
    surface_texture: wgpu::SurfaceTexture,
    surface_view: crate::TextureView,
}

pub struct RenderContext {
    pub webgpu: WebGpuContextRc,
    global_buffers: GlobalBuffers,
    binding_data_buffer: BindingDataBufferRc,
    texture_handler: TextureHandlerRc,
    shared_data: SharedDataRc,
    message_hub: MessageHubRc,
    command_buffer: RwLock<Option<CommandBuffer>>,
    surface: RwLock<Option<SurfaceData>>,
    render_targets: RwLock<Vec<Resource<Texture>>>,
    state: RwLock<RendererState>,
    passes: RwLock<Vec<(Box<dyn Pass>, bool)>>,
}

pub type RenderContextRc = Arc<RenderContext>;

impl RenderContext {
    fn create_surface(instance: &wgpu::Instance, handle: Handle) -> wgpu::Surface<'static> {
        instance.create_surface(handle).unwrap()
    }

    pub fn state(&self) -> RendererState {
        *self.state.read().unwrap()
    }

    pub fn change_state(&self, render_state: RendererState) -> &Self {
        *self.state.write().unwrap() = render_state;
        self
    }

    pub fn binding_data_buffer(&self) -> &BindingDataBufferRc {
        &self.binding_data_buffer
    }

    pub fn global_buffers(&self) -> &GlobalBuffers {
        &self.global_buffers
    }

    pub fn texture_handler(&self) -> &TextureHandlerRc {
        &self.texture_handler
    }

    pub fn new_command_buffer(&self) -> CommandBuffer {
        self.webgpu.new_command_buffer()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn log_adapters(instance: &wgpu::Instance, backends: &wgpu::Backends) {
        use wgpu::Adapter;

        let all_adapters = instance.enumerate_adapters(*backends);
        let mut available_adapters: Vec<Adapter> = Vec::new();
        all_adapters.into_iter().for_each(|a| {
            if !available_adapters
                .iter()
                .any(|ad| ad.get_info().name == a.get_info().name)
            {
                available_adapters.push(a);
            }
        });
        inox_log::debug_log!("Available adapters:");
        available_adapters.into_iter().for_each(|a| {
            inox_log::debug_log!("{}", a.get_info().name);
        });
    }

    pub async fn create_render_context(handle: Handle, context: &ContextRc) -> Self {
        inox_profiler::scoped_profile!("render_context::create_render_context");

        setup_env();

        let (instance, surface, adapter, device, queue) = {
            let backends = if USE_FORCED_VULKAN {
                wgpu::Backends::VULKAN
            } else {
                wgpu::Backends::all()
            };
            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends,
                flags: wgpu::InstanceFlags::all(),
                ..Default::default()
            });
            let surface = Self::create_surface(&instance, handle.clone());

            #[cfg(not(target_arch = "wasm32"))]
            Self::log_adapters(&instance, &backends);

            let adapter =
                wgpu::util::initialize_adapter_from_env_or_default(&instance, Some(&surface))
                    .await
                    .expect("No suitable GPU adapters found on the system!");
            let (device, queue) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: None,
                        required_features: required_gpu_features(),
                        required_limits: platform_limits(),
                    },
                    // Some(&std::path::Path::new("trace")), // Trace path
                    None,
                )
                .await
                .unwrap();

            (instance, surface, adapter, device, queue)
        };

        inox_log::debug_log!("Using {:?}", adapter.get_info());

        let capabilities = surface.get_capabilities(&adapter);
        let format = wgpu::TextureFormat::Bgra8Unorm;

        inox_log::debug_log!("Format {:?}", format);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            view_formats: vec![format],
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode: *capabilities.alpha_modes.first().unwrap(),
            desired_maximum_frame_latency: 2,
        };

        //debug_log!("Surface format: {:?}", config.format);
        surface.configure(&device, &config);
        let _ = surface.get_current_texture();

        inox_profiler::create_gpu_profiler!();

        let webgpu = WebGpuContext {
            instance,
            surface,
            adapter,
            device,
            queue,
            config: RwLock::new(config),
        };

        RenderContext {
            texture_handler: Arc::new(TextureHandler::create(&webgpu.device)),
            shared_data: context.shared_data().clone(),
            message_hub: context.message_hub().clone(),
            state: RwLock::new(RendererState::Submitted),
            webgpu: Arc::new(webgpu),
            binding_data_buffer: Arc::new(BindingDataBuffer::default()),
            global_buffers: GlobalBuffers::default(),
            command_buffer: RwLock::new(None),
            surface: RwLock::new(None),
            render_targets: RwLock::new(Vec::new()),
            passes: RwLock::new(Vec::new()),
        }
    }

    pub fn has_commands(
        &self,
        draw_command_type: &DrawCommandType,
        mesh_flags: &MeshFlags,
    ) -> bool {
        if let Some(commands) = self
            .global_buffers
            .draw_commands
            .read()
            .unwrap()
            .get(mesh_flags)
        {
            if let Some(entry) = commands.map.get(draw_command_type) {
                return !entry.commands.is_empty();
            }
        }
        false
    }

    pub fn resolution(&self) -> (u32, u32) {
        let config = self.webgpu.config.read().unwrap();
        (config.width, config.height)
    }

    pub fn buffers(&self) -> RwLockReadGuard<HashMap<BufferId, GpuBuffer>> {
        self.binding_data_buffer.buffers.read().unwrap()
    }

    pub fn buffers_mut(&self) -> RwLockWriteGuard<HashMap<BufferId, GpuBuffer>> {
        self.binding_data_buffer.buffers.write().unwrap()
    }

    fn update_image(&self, encoder: &mut wgpu::CommandEncoder, texture: &Resource<Texture>) {
        let texture_id = texture.id();
        let texture_blocks = texture.get_mut().blocks_to_update();
        for block in texture_blocks {
            self.texture_handler.update_texture_atlas(
                &self.webgpu.device,
                encoder,
                texture_id,
                &block,
            );
        }
    }

    fn add_image(&self, encoder: &mut wgpu::CommandEncoder, texture: &Resource<Texture>) -> usize {
        let texture_id = texture.id();
        let width = texture.get().width();
        let height = texture.get().height();
        let format = texture.get().format();
        let sample_count = texture.get().sample_count();
        let is_lut = texture.get().is_LUT();
        let mut blocks_to_update = texture.get_mut().blocks_to_update();
        let image_data = blocks_to_update.remove(0);
        let info = self.texture_handler.add_image_to_texture_atlas(
            &self.webgpu.device,
            encoder,
            texture_id,
            (width, height, format, sample_count, is_lut),
            &image_data.data,
        );
        for block in blocks_to_update {
            self.texture_handler.update_texture_atlas(
                &self.webgpu.device,
                encoder,
                texture_id,
                &block,
            );
        }
        info.texture_index as _
    }

    fn add_render_target(&self, texture: &Resource<Texture>) -> usize {
        let texture_id = texture.id();
        let width = texture.get().width();
        let height = texture.get().height();
        let format = texture.get().format();
        let usage = texture.get().usage();
        let sample_count = texture.get().sample_count();
        let index = self.texture_handler.add_render_target(
            &self.webgpu.device,
            texture_id,
            (width, height),
            format,
            usage,
            sample_count,
        );
        index as _
    }

    pub fn on_texture_changed(&self, texture_id: &TextureId, encoder: &mut wgpu::CommandEncoder) {
        inox_profiler::scoped_profile!("render_context::on_texture_changed");
        if let Some(texture) = self.shared_data.get_resource::<Texture>(texture_id) {
            if !texture.get().is_initialized()
                && texture.get().width() > 0
                && texture.get().height() > 0
            {
                if texture.get().usage().contains(TextureUsage::RenderTarget) {
                    let uniform_index = self.add_render_target(&texture);
                    texture.get_mut().set_texture_index(uniform_index);
                } else if self.texture_handler.texture_info(texture_id).is_none() {
                    self.add_image(encoder, &texture);
                    if let Some(texture_info) = self.texture_handler.texture_info(texture_id) {
                        let uniform_index = self.global_buffers.add_texture(
                            texture_id,
                            &texture_info,
                            texture.get().LUT_id(),
                        );
                        texture
                            .get_mut()
                            .set_texture_index(uniform_index)
                            .set_texture_size(texture_info.width(), texture_info.height());
                        //Need to update all materials that use this texture
                        self.shared_data
                            .for_each_resource_mut(|_, m: &mut Material| {
                                if m.has_texture_id(texture_id) {
                                    m.mark_as_dirty();
                                }
                            });
                    }
                } else {
                    //updating an existing texture
                    self.update_image(encoder, &texture);
                }
            }
        }
    }

    pub fn set_command_buffer(&self, command_buffer: CommandBuffer) {
        *self.command_buffer.write().unwrap() = Some(command_buffer);
    }

    pub fn submit_command_buffer(&self) {
        inox_profiler::scoped_profile!("renderer::submit_command_buffer");
        if let Some(mut command_buffer) = self.command_buffer.write().unwrap().take() {
            self.binding_data_buffer.reset_buffers_changed();
            {
                inox_profiler::gpu_profiler_pre_submit!(&mut command_buffer.encoder);
                self.webgpu.submit(command_buffer);
            }
        }
    }

    pub fn obtain_surface_texture(&self) -> bool {
        let screen_texture = {
            inox_profiler::scoped_profile!("wgpu::get_current_texture");

            self.webgpu.surface.get_current_texture()
        };
        if let Ok(surface_texture) = screen_texture {
            let surface_view = surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut surface = self.surface.write().unwrap();
            *surface = Some(SurfaceData {
                surface_view: surface_view.into(),
                surface_texture,
            });
            return true;
        } else {
            self.recreate();
        }
        false
    }

    pub fn present(&self) {
        inox_profiler::scoped_profile!("renderer::present");
        if let Some(surface_data) = self.surface.write().unwrap().take() {
            surface_data.surface_texture.present();
            inox_profiler::gpu_profiler_post_present!(&self.webgpu.queue);
        }
    }

    pub fn set_surface_size(&self, width: u32, height: u32) {
        self.webgpu.config.write().unwrap().width = width;
        self.webgpu.config.write().unwrap().height = height;
        inox_log::debug_log!("Surface size: {}x{}", width, height);
        self.recreate();
    }

    pub fn recreate(&self) {
        inox_profiler::scoped_profile!("renderer::recreate");
        self.webgpu.configure();

        self.shared_data
            .for_each_resource_mut(|_id, pipeline: &mut RenderPipeline| {
                pipeline.invalidate();
            });
        self.shared_data
            .for_each_resource_mut(|_id, pipeline: &mut ComputePipeline| {
                pipeline.invalidate();
            });
        self.shared_data
            .for_each_resource_mut(|_id, render_pass: &mut RenderPass| {
                render_pass.invalidate();
            });
    }

    pub fn create_render_target(
        &self,
        width: u32,
        height: u32,
        format: TextureFormat,
        usage: TextureUsage,
        sample_count: u32,
    ) -> usize {
        let texture = Texture::create_from_format(
            &self.shared_data,
            &self.message_hub,
            width,
            height,
            format,
            usage,
            sample_count,
        );
        let mut render_targets = self.render_targets.write().unwrap();
        render_targets.push(texture);
        render_targets.len() - 1
    }
    pub fn num_render_targets(&self) -> usize {
        self.render_targets.read().unwrap().len()
    }
    pub fn render_target(&self, index: usize) -> Resource<Texture> {
        self.render_targets.read().unwrap()[index].clone()
    }
    pub fn render_target_id(&self, index: usize) -> TextureId {
        *self.render_targets.read().unwrap()[index].id()
    }
    pub fn num_passes(&self) -> usize {
        self.passes.read().unwrap().len()
    }
    pub fn pass_name(&self, index: usize) -> String {
        if let Some(v) = self.passes.read().unwrap().get(index) {
            return v.0.name().to_string();
        }
        String::default()
    }
    pub fn is_pass_enabled(&self, index: usize) -> bool {
        if let Some(v) = self.passes.read().unwrap().get(index) {
            return v.1;
        }
        false
    }
    pub fn set_pass_enabled(&self, index: usize, is_enabled: bool) {
        if let Some(v) = self.passes.write().unwrap().get_mut(index) {
            if v.1 != is_enabled {
                v.1 = is_enabled;
                self.recreate();
            }
        }
    }
    pub fn passes(&self) -> RwLockReadGuard<Vec<(Box<dyn Pass>, bool)>> {
        self.passes.read().unwrap()
    }

    pub fn add_pass(&self, pass: impl Pass, is_enabled: bool) -> &Self {
        self.passes
            .write()
            .unwrap()
            .push((Box::new(pass), is_enabled));
        self
    }

    pub fn update_passes(&self, mut command_buffer: CommandBuffer) {
        inox_profiler::scoped_profile!("renderer::update_passes");

        let mut passes = self.passes.write().unwrap();
        passes.iter_mut().for_each(|(pass, is_enabled)| {
            if *is_enabled && pass.is_active(self) {
                pass.init(self);
            }
        });
        if let Some(surface) = self.surface.read().unwrap().as_ref() {
            passes.iter_mut().for_each(|(pass, is_enabled)| {
                if *is_enabled && pass.is_active(self) {
                    pass.update(self, &surface.surface_view, &mut command_buffer);
                }
            });
        }
        self.set_command_buffer(command_buffer);
    }
}
