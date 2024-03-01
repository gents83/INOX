use crate::{RenderContext, RenderContextRc};
use inox_core::ContextRc;

use inox_messenger::MessageHubRc;

use inox_platform::Handle;
use inox_resources::SharedDataRc;

use std::sync::{Arc, RwLock};

pub const DEFAULT_WIDTH: u32 = 1920;
pub const DEFAULT_HEIGHT: u32 = 1080;
pub const DEFAULT_FOV: f32 = 45.;
pub const DEFAULT_ASPECT_RATIO: f32 = DEFAULT_WIDTH as f32 / DEFAULT_HEIGHT as f32;
pub const DEFAULT_NEAR: f32 = 0.01;
pub const DEFAULT_FAR: f32 = 10000.;

pub struct Renderer {
    render_context: Option<RenderContextRc>,
    shared_data: SharedDataRc,
    message_hub: MessageHubRc,
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
    async fn create_render_context<F>(
        handle: Handle,
        context: ContextRc,
        renderer: RendererRw,
        on_create_func: F,
    ) where
        F: FnOnce(&RenderContextRc),
    {
        let render_context = RenderContext::create_render_context(handle.clone(), &context).await;
        let render_context_rc = Arc::new(render_context);
        renderer
            .write()
            .unwrap()
            .set_render_context(render_context_rc.clone());
        on_create_func(&render_context_rc);
    }

    pub fn new<F>(handle: &Handle, context: &ContextRc, on_create_func: F) -> RendererRw
    where
        F: FnOnce(&RenderContextRc) + 'static,
    {
        crate::register_resource_types(context.shared_data(), context.message_hub());

        let renderer = Arc::new(RwLock::new(Renderer {
            render_context: None,
            shared_data: context.shared_data().clone(),
            message_hub: context.message_hub().clone(),
        }));

        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(Self::create_render_context(
            handle.clone(),
            context.clone(),
            renderer.clone(),
            on_create_func,
        ));

        #[cfg(not(target_arch = "wasm32"))]
        futures::executor::block_on(Self::create_render_context(
            handle.clone(),
            context.clone(),
            renderer.clone(),
            on_create_func,
        ));

        renderer
    }
    pub fn set_render_context(&mut self, context: RenderContextRc) {
        self.render_context = Some(context);
    }
    pub fn shared_data(&self) -> &SharedDataRc {
        &self.shared_data
    }
    pub fn render_context(&self) -> &RenderContextRc {
        self.render_context.as_ref().unwrap()
    }
}
