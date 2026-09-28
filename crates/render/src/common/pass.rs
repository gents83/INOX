use inox_core::ContextRc;

use crate::{CommandBuffer, FrameDimensions, RenderContext, RenderContextRc, TextureView};
use downcast_rs::{impl_downcast, Downcast};

pub trait Pass: Downcast + Send + Sync + 'static {
    fn name(&self) -> &str;
    fn static_name() -> &'static str
    where
        Self: Sized;
    fn is_active(&self, render_context: &RenderContext) -> bool;
    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self
    where
        Self: Sized;
    fn init(&mut self, render_context: &RenderContext);
    /// Allow a pass to retry initialization while it is not yet active.
    fn should_init(&self, render_context: &RenderContext) -> bool {
        self.is_active(render_context)
    }
    /// Reset GPU-produced queue state even when the stage has no work this frame.
    fn reset(&mut self, _render_context: &RenderContext) {}
    /// Propagate an authoritative resize before the next frame is encoded.
    fn resize(&mut self, _dimensions: FrameDimensions) {}
    fn update(
        &mut self,
        render_context: &RenderContext,
        surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    );
}

impl_downcast!(Pass);
