use inox_core::ContextRc;

use crate::{CommandBuffer, DrawCommandType, MeshFlags, RenderContext, TextureView};
use downcast_rs::{impl_downcast, Downcast};

pub trait Pass: Downcast + Send + Sync + 'static {
    fn name(&self) -> &str;
    fn static_name() -> &'static str
    where
        Self: Sized;
    fn is_active(&self, render_context: &RenderContext) -> bool;
    fn draw_commands_type(&self) -> DrawCommandType;
    fn mesh_flags(&self) -> MeshFlags;
    fn create(context: &ContextRc, render_context: &RenderContext) -> Self
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
