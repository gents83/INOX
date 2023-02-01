use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use inox_math::{Degrees, Matrix4, Vector2};
use inox_platform::Handle;
use inox_resources::Resource;

use crate::{
    platform::{platform_limits, required_gpu_features},
    BindingDataBuffer, BindingDataBufferRc, BufferId, ConstantData, ConstantDataRw,
    DrawCommandType, GpuBuffer, MeshFlags, RenderBuffers, Renderer, RendererRw, Texture,
    TextureHandler, TextureHandlerRc, CONSTANT_DATA_FLAGS_SUPPORT_SRGB, DEFAULT_HEIGHT,
    DEFAULT_WIDTH,
};

#[cfg(target_arch = "wasm32")]
const USE_VULKAN: bool = true;
#[cfg(all(not(target_arch = "wasm32")))]
const USE_VULKAN: bool = false;

pub struct CommandBuffer {
    pub encoder: wgpu::CommandEncoder,
}

pub struct RenderCoreContext {
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: RwLock<wgpu::SurfaceConfiguration>,
}

pub type RenderCoreContextRc = Arc<RenderCoreContext>;

impl RenderCoreContext {
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
    pub fn set_surface_size(&self, width: u32, height: u32) {
        self.config.write().unwrap().width = width;
        self.config.write().unwrap().height = height;
        inox_log::debug_log!("Surface size: {}x{}", width, height);
    }
    pub fn configure(&self) {
        inox_profiler::scoped_profile!("render_context::configure");
        self.surface
            .configure(&self.device, &self.config.read().unwrap());
    }
}

pub struct RenderContext {
    pub core: RenderCoreContextRc,
    pub texture_handler: TextureHandlerRc,
    pub binding_data_buffer: BindingDataBufferRc,
    pub render_buffers: RenderBuffers,
    pub constant_data: ConstantDataRw,
}

pub type RenderContextRw = Arc<RwLock<RenderContext>>;

impl RenderContext {
    #[cfg(all(not(target_arch = "wasm32")))]
    fn create_surface(instance: &wgpu::Instance, handle: &Handle) -> wgpu::Surface {
        unsafe { instance.create_surface(&handle).unwrap() }
    }
    #[cfg(target_arch = "wasm32")]
    fn create_surface(instance: &wgpu::Instance, handle: &Handle) -> wgpu::Surface {
        let canvas = handle.handle_impl.canvas();
        instance
            .create_surface_from_canvas(&canvas)
            .expect("Could not create surface from canvas")
    }

    pub async fn create_render_context<F>(handle: Handle, renderer: RendererRw, on_create_func: F)
    where
        F: FnOnce(&mut Renderer),
    {
        inox_profiler::scoped_profile!("render_context::create_render_context");

        let (instance, surface, adapter, device, queue) = {
            let dx12_shader_compiler =
                wgpu::util::dx12_shader_compiler_from_env().unwrap_or_default();
            let backends = if USE_VULKAN {
                wgpu::Backends::VULKAN
            } else {
                wgpu::Backends::all()
            };
            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends,
                dx12_shader_compiler,
            });
            let surface = Self::create_surface(&instance, &handle);

            let adapter = wgpu::util::initialize_adapter_from_env_or_default(
                &instance,
                backends,
                Some(&surface),
            )
            .await
            .expect("No suitable GPU adapters found on the system!");
            if let Ok((device, queue)) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: None,
                        features: required_gpu_features(),
                        limits: platform_limits(),
                    },
                    // Some(&std::path::Path::new("trace")), // Trace path
                    None,
                )
                .await
            {
                (instance, surface, adapter, device, queue)
            } else {
                let dx12_shader_compiler =
                    wgpu::util::dx12_shader_compiler_from_env().unwrap_or_default();
                let vulkan_backend = wgpu::Backends::VULKAN;
                let vulkan_instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                    backends: vulkan_backend,
                    dx12_shader_compiler,
                });
                let vulkan_surface = Self::create_surface(&vulkan_instance, &handle);

                let vulkan_adapter = wgpu::util::initialize_adapter_from_env_or_default(
                    &vulkan_instance,
                    vulkan_backend,
                    Some(&vulkan_surface),
                )
                .await
                .expect("No suitable VULKAN GPU adapter found on the system!");
                let (vulkan_device, vulkan_queue) = vulkan_adapter
                    .request_device(
                        &wgpu::DeviceDescriptor {
                            label: None,
                            features: required_gpu_features(),
                            limits: platform_limits(),
                        },
                        // Some(&std::path::Path::new("trace")), // Trace path
                        None,
                    )
                    .await
                    .expect("Failed to create device");

                (
                    vulkan_instance,
                    vulkan_surface,
                    vulkan_adapter,
                    vulkan_device,
                    vulkan_queue,
                )
            }
        };

        inox_log::debug_log!("Using {:?} adapter", adapter.get_info().backend);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: *surface.get_capabilities(&adapter).formats.first().unwrap(),
            view_formats: vec![wgpu::TextureFormat::Rgba8Unorm],
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode: *surface
                .get_capabilities(&adapter)
                .alpha_modes
                .first()
                .unwrap(),
        };

        //debug_log!("Surface format: {:?}", config.format);
        surface.configure(&device, &config);

        inox_profiler::create_gpu_profiler!(&device, &queue, false);

        let render_core_context = RenderCoreContext {
            instance,
            surface,
            adapter,
            device,
            queue,
            config: RwLock::new(config),
        };

        renderer
            .write()
            .unwrap()
            .set_render_context(Arc::new(RwLock::new(RenderContext {
                texture_handler: Arc::new(TextureHandler::create(&render_core_context.device)),
                core: Arc::new(render_core_context),
                constant_data: Arc::new(RwLock::new(ConstantData::default())),
                binding_data_buffer: Arc::new(BindingDataBuffer::default()),
                render_buffers: RenderBuffers::default(),
            })));

        let mut renderer = renderer.write().unwrap();
        on_create_func(&mut renderer);
    }

    pub fn update_constant_data(
        &self,
        view: Matrix4,
        proj: Matrix4,
        screen_size: Vector2,
        fov_in_degrees: Degrees,
    ) {
        inox_profiler::scoped_profile!("render_context::update_constant_data");
        self.constant_data
            .write()
            .unwrap()
            .update(view, proj, screen_size, fov_in_degrees);
        if self.core.config.read().unwrap().format.describe().srgb {
            self.constant_data
                .write()
                .unwrap()
                .add_flag(CONSTANT_DATA_FLAGS_SUPPORT_SRGB);
        } else {
            self.constant_data
                .write()
                .unwrap()
                .remove_flag(CONSTANT_DATA_FLAGS_SUPPORT_SRGB);
        }
    }

    pub fn has_commands(
        &self,
        draw_command_type: &DrawCommandType,
        mesh_flags: &MeshFlags,
    ) -> bool {
        if let Some(commands) = self.render_buffers.commands.read().unwrap().get(mesh_flags) {
            if let Some(entry) = commands.map.get(draw_command_type) {
                return !entry.commands.is_empty();
            }
        }
        false
    }

    pub fn resolution(&self) -> (u32, u32) {
        let config = self.core.config.read().unwrap();
        (config.width, config.height)
    }

    pub fn buffers(&self) -> RwLockReadGuard<HashMap<BufferId, GpuBuffer>> {
        self.binding_data_buffer.buffers.read().unwrap()
    }

    pub fn buffers_mut(&mut self) -> RwLockWriteGuard<HashMap<BufferId, GpuBuffer>> {
        self.binding_data_buffer.buffers.write().unwrap()
    }

    pub fn add_image(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        texture: &Resource<Texture>,
    ) -> usize {
        let texture_id = texture.id();
        let width = texture.get().width();
        let height = texture.get().height();
        let format = texture.get().format();
        let index = if let Some(image_data) = texture.get().image_data() {
            let info = self.texture_handler.add_image_to_texture_atlas(
                &self.core.device,
                encoder,
                texture_id,
                (width, height),
                format,
                image_data,
            );
            info.texture_index as _
        } else {
            let usage = texture.get().usage();
            let index = self.texture_handler.add_render_target(
                &self.core.device,
                texture_id,
                (width, height),
                format,
                usage,
            );
            index as _
        };
        index
    }
}
