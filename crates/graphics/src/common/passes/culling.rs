use std::path::PathBuf;

use crate::{ComputePass, ComputePassData, Pass, RenderContext};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;

pub const CULLING_PIPELINE: &str = "pipelines/Culling.pipeline";
pub const CULLING_PASS_NAME: &str = "CullingPass";

#[derive(Clone)]
pub struct CullingPass {
    _compute_pass: Resource<ComputePass>,
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
            _compute_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                data,
            ),
        }
    }
    fn prepare(&mut self, _render_context: &RenderContext) {}
    fn update(&mut self, _render_context: &RenderContext) {}
}
