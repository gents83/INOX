use crate::{
    CommandBuffer, ComputePipeline, Material, OutputPass, RenderContext, RenderContextRw,
    RenderPass, RenderPipeline, Texture, TextureId, TextureUsage, TextureView,
};
use inox_core::ContextRc;

use inox_messenger::MessageHubRc;

use inox_platform::Handle;
use inox_resources::{ResourceTrait, SharedData, SharedDataRc};

use std::sync::{Arc, RwLock, RwLockReadGuard};

pub const DEFAULT_WIDTH: u32 = 1920;
pub const DEFAULT_HEIGHT: u32 = 1080;
pub const DEFAULT_FOV: f32 = 45.;
pub const DEFAULT_ASPECT_RATIO: f32 = DEFAULT_WIDTH as f32 / DEFAULT_HEIGHT as f32;
pub const DEFAULT_NEAR: f32 = 0.01;
pub const DEFAULT_FAR: f32 = 100000.;

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum RendererState {
    Init,
    Preparing,
    Prepared,
    Drawing,
    Submitted,
    Rendered,
}

pub struct Renderer {
    render_context: Option<RenderContextRw>,
    shared_data: SharedDataRc,
    message_hub: MessageHubRc,
    state: RendererState,
    passes: Vec<(Box<dyn OutputPass>, bool)>,
    command_buffer: Option<CommandBuffer>,
    surface_texture: Option<wgpu::SurfaceTexture>,
    surface_view: Option<TextureView>,
    need_recreate: bool,
    need_commands_rebind: bool,
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
    pub fn new<F>(handle: &Handle, context: &ContextRc, on_create_func: F) -> RendererRw
    where
        F: FnOnce(&mut Renderer) + 'static,
    {
        crate::register_resource_types(context.shared_data(), context.message_hub());

        let renderer = Arc::new(RwLock::new(Renderer {
            state: RendererState::Init,
            render_context: None,
            shared_data: context.shared_data().clone(),
            message_hub: context.message_hub().clone(),
            passes: Vec::new(),
            command_buffer: None,
            surface_texture: None,
            surface_view: None,
            need_recreate: false,
            need_commands_rebind: true,
        }));

        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(RenderContext::create_render_context(
            handle.clone(),
            renderer.clone(),
            on_create_func,
        ));

        #[cfg(not(target_arch = "wasm32"))]
        futures::executor::block_on(RenderContext::create_render_context(
            handle.clone(),
            renderer.clone(),
            on_create_func,
        ));

        renderer
    }
    pub fn set_render_context(&mut self, context: RenderContextRw) {
        self.render_context = Some(context);
    }
    pub fn render_context(&self) -> RwLockReadGuard<RenderContext> {
        self.render_context.as_ref().unwrap().read().unwrap()
    }
    pub fn num_passes(&self) -> usize {
        self.passes.len()
    }
    pub fn is_pass_enabled(&self, index: usize) -> bool {
        if let Some(v) = self.passes.get(index) {
            return v.1;
        }
        false
    }
    pub fn set_pass_enabled(&mut self, index: usize, is_enabled: bool) {
        if let Some(v) = self.passes.get_mut(index) {
            if v.1 != is_enabled {
                v.1 = is_enabled;
                self.need_recreate = true;
                self.need_commands_rebind = true;
            }
        }
    }
    pub fn pass_at(&self, index: usize) -> Option<&dyn OutputPass> {
        self.passes.get(index).map(|v| v.0.as_ref())
    }
    pub fn pass<T>(&self) -> Option<&T>
    where
        T: OutputPass,
    {
        if let Some(p) = self
            .passes
            .iter()
            .find(|(pass, _)| pass.name() == T::static_name())
        {
            return p.0.downcast_ref::<T>();
        }
        None
    }
    pub fn pass_mut<T>(&mut self) -> Option<&mut T>
    where
        T: OutputPass,
    {
        if let Some(p) = self
            .passes
            .iter_mut()
            .find(|(pass, _)| pass.name() == T::static_name())
        {
            return p.0.downcast_mut::<T>();
        }
        None
    }

    pub fn add_pass(&mut self, pass: impl OutputPass, is_enabled: bool) -> &mut Self {
        self.passes.push((Box::new(pass), is_enabled));
        self
    }

    pub fn check_initialization(&mut self) {
        if self.render_context.is_none() {
            self.state = RendererState::Init;
        } else {
            self.state = RendererState::Submitted;
        }
    }

    pub fn state(&self) -> RendererState {
        self.state
    }
    pub fn change_state(&mut self, render_state: RendererState) -> &mut Self {
        self.state = render_state;
        self
    }

    pub fn need_redraw(&self) -> bool {
        self.state != RendererState::Submitted
    }

    pub fn recreate(&mut self) {
        inox_profiler::scoped_profile!("renderer::recreate");

        self.need_recreate = false;

        self.render_context().core.configure();

        SharedData::for_each_resource_mut(
            &self.shared_data,
            |_id, pipeline: &mut RenderPipeline| {
                pipeline.invalidate();
            },
        );
        SharedData::for_each_resource_mut(
            &self.shared_data,
            |_id, pipeline: &mut ComputePipeline| {
                pipeline.invalidate();
            },
        );
        SharedData::for_each_resource_mut(
            &self.shared_data,
            |_id, render_pass: &mut RenderPass| {
                render_pass.invalidate();
            },
        );
    }

    pub fn set_surface_size(&mut self, width: u32, height: u32) {
        self.render_context().core.set_surface_size(width, height);
        self.recreate();
    }

    pub fn on_texture_changed(
        &mut self,
        texture_id: &TextureId,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        inox_profiler::scoped_profile!("renderer::on_texture_changed");
        let mut render_context = self.render_context.as_ref().unwrap().write().unwrap();
        if let Some(texture) = self.shared_data.get_resource::<Texture>(texture_id) {
            if !texture.get().is_initialized()
                && texture.get().width() > 0
                && texture.get().height() > 0
            {
                if texture
                    .get()
                    .usage()
                    .contains(TextureUsage::RenderAttachment)
                {
                    let uniform_index = render_context.add_image(encoder, &texture);
                    texture.get_mut().set_texture_index(uniform_index);
                } else if render_context
                    .texture_handler
                    .texture_info(texture_id)
                    .is_none()
                {
                    render_context.add_image(encoder, &texture);
                    if let Some(texture_info) =
                        render_context.texture_handler.texture_info(texture_id)
                    {
                        let uniform_index = render_context
                            .render_buffers
                            .add_texture(texture_id, &texture_info);
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
                }
            }
        }
    }

    pub fn obtain_surface_texture(&mut self) -> bool {
        if self.need_recreate {
            return false;
        }
        let surface_texture = {
            inox_profiler::scoped_profile!("wgpu::get_current_texture");

            self.render_context().core.surface.get_current_texture()
        };
        if let Ok(screen_texture) = surface_texture {
            let screen_view = screen_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            self.surface_view = Some(TextureView::new(screen_view));
            self.surface_texture = Some(screen_texture);
            true
        } else {
            inox_log::debug_log!("Unable to retrieve surface texture");
            self.recreate();
            false
        }
    }

    pub fn prepare(&mut self) {
        inox_profiler::scoped_profile!("renderer::prepare");

        let mut render_context = self.render_context.as_ref().unwrap().write().unwrap();
        let render_context: &mut RenderContext = &mut render_context;
        let render_buffers = &mut render_context.render_buffers;
        let render_core_context = &render_context.core;
        let binding_data_buffer = &render_context.binding_data_buffer;
        render_buffers.bind_commands(
            binding_data_buffer,
            render_core_context,
            self.need_commands_rebind,
        );
        self.need_commands_rebind = false;
    }
    pub fn update_passes(&mut self, command_buffer: CommandBuffer) {
        inox_profiler::scoped_profile!("renderer::update_passes");

        let render_context = self.render_context.as_ref().unwrap().read().unwrap();
        let render_context: &RenderContext = &render_context;
        self.passes.iter_mut().for_each(|(pass, is_enabled)| {
            if *is_enabled && pass.is_active(render_context) {
                pass.init(render_context);
            }
        });
        self.command_buffer = Some(command_buffer);
        if let Some(surface_view) = &self.surface_view {
            self.passes.iter_mut().for_each(|(pass, is_enabled)| {
                if *is_enabled && pass.is_active(render_context) {
                    pass.update(
                        render_context,
                        surface_view,
                        self.command_buffer.as_mut().unwrap(),
                    );
                }
            });
        }
    }

    pub fn submit_command_buffer(&mut self) {
        inox_profiler::scoped_profile!("renderer::submit_command_buffer");
        if let Some(mut command_buffer) = self.command_buffer.take() {
            let render_context = self.render_context.as_ref().unwrap().read().unwrap();

            render_context.binding_data_buffer.reset_buffers_changed();
            {
                inox_profiler::gpu_profiler_pre_submit!(&mut command_buffer.encoder);
                render_context.core.submit(command_buffer);
            }
        }
    }

    pub fn present(&mut self) {
        inox_profiler::scoped_profile!("renderer::present");
        self.surface_view = None;
        if let Some(surface_texture) = self.surface_texture.take() {
            surface_texture.present();
            inox_profiler::gpu_profiler_post_present!();
        }
    }
}
