use crate::{
    ComputePipeline, Material, Pass, RenderContext, RenderContextRw, RenderPass, RenderPipeline,
    Texture, TextureId,
};
use inox_core::ContextRc;

use inox_messenger::MessageHubRc;
use inox_resources::DataTypeResource;

use inox_platform::Handle;
use inox_resources::{SharedData, SharedDataRc};

use std::sync::{Arc, RwLock};

pub const DEFAULT_WIDTH: u32 = 1920;
pub const DEFAULT_HEIGHT: u32 = 1080;

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
    passes: Vec<Box<dyn Pass>>,
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
    pub fn new(handle: &Handle, context: &ContextRc, _enable_debug: bool) -> RendererRw {
        crate::register_resource_types(context.shared_data(), context.message_hub());

        let renderer = Arc::new(RwLock::new(Renderer {
            state: RendererState::Init,
            render_context: None,
            shared_data: context.shared_data().clone(),
            message_hub: context.message_hub().clone(),
            passes: Vec::new(),
        }));

        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(RenderContext::create_render_context(
            handle.clone(),
            renderer.clone(),
        ));

        #[cfg(all(not(target_arch = "wasm32")))]
        futures::executor::block_on(RenderContext::create_render_context(
            handle.clone(),
            renderer.clone(),
        ));

        renderer
    }
    pub fn set_render_context(&mut self, context: RenderContextRw) {
        self.render_context = Some(context);
    }
    pub fn render_context(&self) -> &RenderContextRw {
        self.render_context.as_ref().unwrap()
    }
    pub fn passes(&self) -> &[Box<dyn Pass>] {
        self.passes.as_slice()
    }
    pub fn pass<T>(&self) -> Option<&T>
    where
        T: Pass,
    {
        if let Some(p) = self
            .passes
            .iter()
            .find(|pass| pass.name() == T::static_name())
        {
            return p.downcast_ref::<T>();
        }
        None
    }
    pub fn pass_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Pass,
    {
        if let Some(p) = self
            .passes
            .iter_mut()
            .find(|pass| pass.name() == T::static_name())
        {
            return p.downcast_mut::<T>();
        }
        None
    }

    pub fn add_pass(&mut self, pass: impl Pass) -> &mut Self {
        self.passes.push(Box::new(pass));
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

    pub fn recreate(&self) {
        inox_profiler::scoped_profile!("renderer::recreate");

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

    pub fn set_surface_size(&self, width: u32, height: u32) {
        let mut render_context = self.render_context().write().unwrap();
        render_context.core.set_surface_size(width, height);
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
            if !texture.get().is_initialized() {
                if render_context
                    .texture_handler
                    .texture_info(texture_id)
                    .is_none()
                {
                    render_context.add_image(encoder, &texture);
                }
                if let Some(texture_data) = render_context.texture_handler.texture_info(texture_id)
                {
                    let uniform_index = render_context
                        .render_buffers
                        .add_texture(texture_id, &texture_data);
                    texture
                        .get_mut()
                        .set_texture_index(uniform_index)
                        .set_texture_size(texture_data.width(), texture_data.height());
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

    pub fn obtain_surface_texture(&self) -> bool {
        let surface_texture = {
            inox_profiler::scoped_profile!("wgpu::get_current_texture");

            let render_context = self.render_context().read().unwrap();
            render_context.core.surface.get_current_texture()
        };
        if let Ok(output) = surface_texture {
            let screen_view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut render_context = self.render_context().write().unwrap();
            render_context.surface_view = Some(screen_view);
            render_context.surface_texture = Some(output);
            true
        } else {
            inox_log::debug_log!("Unable to retrieve surface texture");
            false
        }
    }

    pub fn init_passes(&mut self) {
        inox_profiler::scoped_profile!("renderer::send_to_gpu");

        let mut render_context = self.render_context.as_ref().unwrap().write().unwrap();
        let render_context: &mut RenderContext = &mut render_context;
        self.passes.iter_mut().for_each(|pass| {
            pass.init(render_context);
        });
        let render_buffers = &mut render_context.render_buffers;
        let render_core_context = &render_context.core;
        let binding_data_buffer = &render_context.binding_data_buffer;
        render_buffers.create_commands(binding_data_buffer, render_core_context);
    }

    pub fn update_passes(&mut self) {
        inox_profiler::scoped_profile!("renderer::execute_passes");

        let mut render_context = self.render_context.as_ref().unwrap().write().unwrap();
        let render_context: &mut RenderContext = &mut render_context;
        let mut command_buffer = render_context.core.new_command_buffer();
        self.passes.iter_mut().for_each(|pass| {
            pass.update(render_context, &mut command_buffer);
        });
        render_context.core.submit(command_buffer);
    }

    pub fn present(&self) {
        let surface_texture = {
            let mut render_context = self.render_context().write().unwrap();
            render_context.surface_view = None;
            render_context.surface_texture.take()
        };

        if let Some(surface_texture) = surface_texture {
            surface_texture.present();
        }
        let render_context = self.render_context().read().unwrap();
        render_context.core.device.poll(wgpu::MaintainBase::Wait);
    }
}
