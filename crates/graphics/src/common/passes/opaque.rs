use std::path::PathBuf;

use crate::{ComputePass, Pass, RenderPass, RenderPassData, RenderTarget, StoreOperation};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Handle, Resource};
use inox_uid::generate_random_uid;

pub const DEFAULT_PIPELINE: &str = "pipelines/Default.pipeline";
pub const WIREFRAME_PIPELINE: &str = "pipelines/Wireframe.pipeline";
pub const OPAQUE_PASS_NAME: &str = "OpaquePass";

#[derive(Clone)]
pub struct OpaquePass {
    render_pass: Resource<RenderPass>,
}
unsafe impl Send for OpaquePass {}
unsafe impl Sync for OpaquePass {}

impl Pass for OpaquePass {
    fn create(context: &ContextRc) -> Self
    where
        Self: Sized,
    {
        let data = RenderPassData {
            name: OPAQUE_PASS_NAME.to_string(),
            store_color: StoreOperation::Store,
            store_depth: StoreOperation::Store,
            render_target: RenderTarget::Screen,
            pipelines: vec![
                PathBuf::from(DEFAULT_PIPELINE),
                PathBuf::from(WIREFRAME_PIPELINE),
            ],
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
