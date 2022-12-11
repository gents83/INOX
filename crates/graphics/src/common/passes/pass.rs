use inox_core::ContextRc;
use inox_resources::Resource;

use crate::{
    CommandBuffer, DrawCommandType, MeshFlags, RenderContext, RenderPass, TextureId, TextureView,
};
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

pub trait OutputPass: Pass {
    fn render_targets_id(&self) -> Vec<TextureId>;
}
pub trait OutputRenderPass: OutputPass {
    fn render_pass(&self) -> &Resource<RenderPass>;
}
impl<T> OutputPass for T
where
    T: OutputRenderPass,
{
    fn render_targets_id(&self) -> Vec<TextureId> {
        self.render_pass()
            .get()
            .render_textures_id()
            .iter()
            .map(|&id| *id)
            .collect()
    }
}
impl_downcast!(Pass);
impl_downcast!(OutputPass);
impl_downcast!(OutputRenderPass);
