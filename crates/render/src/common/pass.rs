use inox_core::ContextRc;

use crate::{CommandBuffer, RenderContext, RenderContextRc, TextureView};
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
    fn update(
        &mut self,
        render_context: &RenderContext,
        surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    );
}

impl_downcast!(Pass);
