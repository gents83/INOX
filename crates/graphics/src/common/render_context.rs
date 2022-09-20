use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use inox_math::{Matrix4, Vector2};
use inox_platform::Handle;
use inox_resources::Resource;

use crate::{
    platform::{platform_limits, required_gpu_features},
    AsBinding, BufferId, ConstantData, DrawCommandType, GpuBuffer, MeshFlags, RenderBuffers,
    RenderPass, RendererRw, Texture, TextureFormat, TextureHandler,
    CONSTANT_DATA_FLAGS_SUPPORT_SRGB, DEFAULT_HEIGHT, DEFAULT_WIDTH,
};

#[derive(Default)]
pub struct BindingDataBuffer {
    pub buffers: RwLock<HashMap<BufferId, GpuBuffer>>,
}

impl BindingDataBuffer {
    pub fn has_buffer(&self, uid: &BufferId) -> bool {
        self.buffers.read().unwrap().contains_key(uid)
    }
    pub fn bind_buffer<T>(
        &self,
        id: BufferId,
        data: &mut T,
        usage: wgpu::BufferUsages,
        render_core_context: &RenderCoreContext,
    ) -> (bool, BufferId)
    where
        T: AsBinding,
    {
        let mut bind_data_buffer = self.buffers.write().unwrap();
        let buffer = bind_data_buffer
            .entry(id)
            .or_insert_with(GpuBuffer::default);
        buffer.bind(id, data, usage, render_core_context)
    }
}

pub struct CommandBuffer {
    pub encoder: wgpu::CommandEncoder,
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
    pub fn new_command_buffer(&self) -> CommandBuffer {
        CommandBuffer {
            encoder: self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Command Encoder"),
                }),
        }
    }
    pub fn submit(&self, command_buffer: CommandBuffer) {
        let command_buffer = command_buffer.encoder.finish();
        self.queue.submit(std::iter::once(command_buffer));
    }
    pub fn set_surface_size(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        inox_log::debug_log!("Surface size: {}x{}", width, height);
    }
    pub fn configure(&mut self) {
        self.surface.configure(&self.device, &self.config);
    }
}

pub struct RenderContext {
    pub core: RenderCoreContext,
    pub frame_commands: Option<wgpu::CommandEncoder>,
    pub surface_texture: Option<wgpu::SurfaceTexture>,
    pub surface_view: Option<wgpu::TextureView>,
    pub constant_data: ConstantData,
    pub texture_handler: TextureHandler,
    pub binding_data_buffer: BindingDataBuffer,
    pub render_buffers: RenderBuffers,
}

pub type RenderContextRw = Arc<RwLock<RenderContext>>;

impl RenderContext {
    pub async fn create_render_context(handle: Handle, renderer: RendererRw) {
        inox_profiler::scoped_profile!("render_context::create_render_context");

        let (instance, surface, adapter, device, queue) = {
            let backend = wgpu::Backends::all();
            let instance = wgpu::Instance::new(backend);
            let surface = unsafe { instance.create_surface(&handle) };

            let adapter = wgpu::util::initialize_adapter_from_env_or_default(
                &instance,
                backend,
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
                let vulkan_backend = wgpu::Backends::VULKAN;
                let vulkan_instance = wgpu::Instance::new(vulkan_backend);
                let vulkan_surface = unsafe { vulkan_instance.create_surface(&handle) };

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

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: *surface.get_supported_formats(&adapter).first().unwrap(),
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: *surface.get_supported_alpha_modes(&adapter).first().unwrap(),
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
                frame_commands: None,
                surface_texture: None,
                surface_view: None,
                constant_data: ConstantData::default(),
                binding_data_buffer: BindingDataBuffer::default(),
                render_buffers: RenderBuffers::default(),
            })));
    }

    pub fn update_constant_data(&mut self, view: Matrix4, proj: Matrix4, screen_size: Vector2) {
        inox_profiler::scoped_profile!("render_context::update_constant_data");
        self.constant_data.update(view, proj, screen_size);
        if self.core.config.format.describe().srgb {
            self.constant_data
                .add_flag(CONSTANT_DATA_FLAGS_SUPPORT_SRGB);
        } else {
            self.constant_data
                .remove_flag(CONSTANT_DATA_FLAGS_SUPPORT_SRGB);
        }
    }

    pub fn has_meshes(&self, flags: MeshFlags) -> bool {
        let mesh_flags: u32 = flags.into();
        self.render_buffers
            .meshes
            .data()
            .iter()
            .any(|m| m.mesh_flags == mesh_flags)
    }

    pub fn has_commands(
        &self,
        draw_command_type: &DrawCommandType,
        mesh_flags: &MeshFlags,
    ) -> bool {
        if let Some(commands) = self.render_buffers.commands.get(mesh_flags) {
            if let Some(entry) = commands.map.get(draw_command_type) {
                return !entry.commands.is_empty();
            }
        }
        false
    }

    pub fn resolution(&self) -> (u32, u32) {
        (self.core.config.width, self.core.config.height)
    }

    pub fn buffers(&self) -> RwLockReadGuard<HashMap<BufferId, GpuBuffer>> {
        self.binding_data_buffer.buffers.read().unwrap()
    }

    pub fn buffers_mut(&mut self) -> RwLockWriteGuard<HashMap<BufferId, GpuBuffer>> {
        self.binding_data_buffer.buffers.write().unwrap()
    }

    pub fn render_targets<'a>(&'a self, render_pass: &'a RenderPass) -> Vec<&'a wgpu::TextureView> {
        let mut render_targets = Vec::new();
        let render_textures = render_pass.render_textures_id();
        if render_textures.is_empty() {
            debug_assert!(self.surface_view.is_some());
            render_targets.push(self.surface_view.as_ref().unwrap());
        } else {
            render_textures.iter().for_each(|&id| {
                if let Some(texture_view) = self.texture_handler.texture_view(id) {
                    render_targets.push(texture_view);
                }
            });
        }
        render_targets
    }

    pub fn depth_target<'a>(
        &'a self,
        render_pass: &'a RenderPass,
    ) -> Option<&'a wgpu::TextureView> {
        if let Some(texture) = render_pass.depth_texture() {
            if let Some(texture_view) = self.texture_handler.texture_view(texture.id()) {
                return Some(texture_view);
            }
        }
        None
    }

    pub fn render_formats(&self, render_pass: &RenderPass) -> Vec<&TextureFormat> {
        let mut render_formats = Vec::new();
        let render_textures = render_pass.render_textures_id();
        render_textures.iter().for_each(|&id| {
            if let Some(format) = self.texture_handler.texture_format(id) {
                render_formats.push(format);
            }
        });
        render_formats
    }

    pub fn depth_format(&self, render_pass: &RenderPass) -> Option<&TextureFormat> {
        if let Some(texture) = render_pass.depth_texture() {
            self.texture_handler.texture_format(texture.id())
        } else {
            None
        }
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
