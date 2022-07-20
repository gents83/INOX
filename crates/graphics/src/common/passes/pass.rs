use inox_core::ContextRc;

use crate::{CommandBuffer, DrawCommandType, MeshFlags, RenderContext};
use downcast_rs::{impl_downcast, Downcast};

pub trait Pass: Downcast + Send + Sync + 'static {
    fn name(&self) -> &str;
    fn static_name() -> &'static str
    where
        Self: Sized;
    fn is_active(&self) -> bool;
    fn draw_command_type(&self) -> DrawCommandType;
    fn mesh_flags(&self) -> MeshFlags;
    fn create(context: &ContextRc) -> Self
    where
        Self: Sized;
    fn init(&mut self, render_context: &mut RenderContext);
    fn update(&self, render_context: &mut RenderContext, command_buffer: &mut CommandBuffer);
}
impl_downcast!(Pass);
