use std::path::PathBuf;

use crate::{ComputePass, ComputePassData, Pass, RenderPass};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Handle, Resource};
use inox_uid::generate_random_uid;

pub const CULLING_PIPELINE: &str = "pipelines/Culling.pipeline";
pub const CULLING_PASS_NAME: &str = "CullingPass";

#[derive(Clone)]
pub struct CullingPass {
    compute_pass: Resource<ComputePass>,
}
unsafe impl Send for CullingPass {}
unsafe impl Sync for CullingPass {}

impl Pass for CullingPass {
    fn create(context: &ContextRc) -> Self
    where
        Self: Sized,
    {
        let data = ComputePassData {
            name: CULLING_PASS_NAME.to_string(),
            pipelines: vec![PathBuf::from(CULLING_PIPELINE)],
        };
        Self {
            compute_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                data,
            ),
        }
    }
    fn render_pass(&self) -> Handle<RenderPass> {
        None
    }
    fn compute_pass(&self) -> Handle<ComputePass> {
        Some(self.compute_pass.clone())
    }
}
