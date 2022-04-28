use inox_core::ContextRc;
use inox_resources::Resource;

use crate::RenderPass;
use downcast_rs::{impl_downcast, Downcast};

pub trait Pass: Downcast + 'static + Send + Sync {
    fn create(context: &ContextRc) -> Self
    where
        Self: Sized;
    fn pass(&self) -> &Resource<RenderPass>;
}
impl_downcast!(Pass);
