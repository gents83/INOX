use std::path::PathBuf;

use crate::{
    ComputePass, LoadOperation, Pass, RenderPass, RenderPassData, RenderTarget, StoreOperation,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Handle, Resource};
use inox_uid::generate_random_uid;

pub const TRANSPARENT_PIPELINE: &str = "pipelines/Transparent.pipeline";
pub const TRANSPARENT_PASS_NAME: &str = "TransparentPass";

#[derive(Clone)]
pub struct TransparentPass {
    render_pass: Resource<RenderPass>,
}
unsafe impl Send for TransparentPass {}
unsafe impl Sync for TransparentPass {}

impl Pass for TransparentPass {
    fn create(context: &ContextRc) -> Self
    where
        Self: Sized,
    {
        let data = RenderPassData {
            name: TRANSPARENT_PASS_NAME.to_string(),
            load_color: LoadOperation::Load,
            store_color: StoreOperation::Store,
            load_depth: LoadOperation::Load,
            store_depth: StoreOperation::Store,
            render_target: RenderTarget::Screen,
            pipelines: vec![PathBuf::from(TRANSPARENT_PIPELINE)],
            ..Default::default()
        };
        Self {
            render_pass: RenderPass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                data,
            ),
        }
    }
    fn render_pass(&self) -> Handle<RenderPass> {
        Some(self.render_pass.clone())
    }
    fn compute_pass(&self) -> Handle<ComputePass> {
        None
    }
}
