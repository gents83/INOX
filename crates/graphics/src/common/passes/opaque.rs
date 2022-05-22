use std::path::PathBuf;

use crate::{
    platform::is_indirect_mode_enabled, BindingData, Pass, RenderContext, RenderMode, RenderPass,
    RenderPassData, RenderTarget, ShaderStage, StoreOperation,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;

pub const DEFAULT_PIPELINE: &str = "pipelines/Default.render_pipeline";
pub const WIREFRAME_PIPELINE: &str = "pipelines/Wireframe.render_pipeline";
pub const OPAQUE_PASS_NAME: &str = "OpaquePass";

pub struct OpaquePass {
    render_pass: Resource<RenderPass>,
    binding_data: BindingData,
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
            binding_data: BindingData::default(),
        }
    }
    fn init(&mut self, render_context: &mut RenderContext) {
        let mut pass = self.render_pass.get_mut();
        let render_texture = pass.render_texture_id();
        let depth_texture = pass.depth_texture_id();

        self.binding_data
            .add_uniform_data(
                render_context,
                0,
                0,
                &render_context.constant_data,
                ShaderStage::VertexAndFragment,
            )
            .add_storage_data(
                render_context,
                0,
                1,
                &render_context.dynamic_data,
                ShaderStage::VertexAndFragment,
                true,
            )
            .add_textures_data(
                1,
                &render_context.texture_handler,
                render_texture,
                depth_texture,
                ShaderStage::Fragment,
            );
        self.binding_data.send_to_gpu(render_context);

        pass.init_pipelines(render_context, &self.binding_data);
    }
    fn update(&mut self, render_context: &RenderContext) {
        let pass = self.render_pass.get();

        pass.pipelines().iter().for_each(|pipeline| {
            if is_indirect_mode_enabled() && pass.data().render_mode == RenderMode::Indirect {
                render_context
                    .graphics_data
                    .get_mut()
                    .fill_command_buffer(render_context, pipeline.id());
            }
        });

        let mut encoder = render_context.new_encoder();
        let render_pass = pass.begin(render_context, &self.binding_data, &mut encoder);
        pass.draw(render_context, render_pass);

        render_context.submit(encoder);
    }
}

impl OpaquePass {
    pub fn render_pass(&self) -> &Resource<RenderPass> {
        &self.render_pass
    }
}
