use inox_core::ContextRc;
use inox_resources::Handle;

use crate::{ComputePass, RenderPass};
use downcast_rs::{impl_downcast, Downcast};

pub trait Pass: Downcast + 'static + Send + Sync {
    fn create(context: &ContextRc) -> Self
    where
        Self: Sized;
    fn render_pass(&self) -> Handle<RenderPass>;
    fn compute_pass(&self) -> Handle<ComputePass>;
}
impl_downcast!(Pass);
