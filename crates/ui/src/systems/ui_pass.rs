use std::path::PathBuf;

use inox_core::ContextRc;
use inox_graphics::{Pass, RenderPass, RenderPassData, RenderTarget, StoreOperation};
use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;

const UI_PIPELINE: &str = "pipelines/UI.pipeline";
pub const UI_PASS_NAME: &str = "UIPass";

pub struct UIPass {
    render_pass: Resource<RenderPass>,
}
unsafe impl Send for UIPass {}
unsafe impl Sync for UIPass {}

impl Pass for UIPass {
    fn create(context: &ContextRc) -> Self
    where
        Self: Sized,
    {
        let data = RenderPassData {
            name: UI_PASS_NAME.to_string(),
            store_color: StoreOperation::Store,
            render_target: RenderTarget::Screen,
            pipelines: vec![PathBuf::from(UI_PIPELINE)],
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
    fn pass(&self) -> &Resource<RenderPass> {
        &self.render_pass
    }
}
