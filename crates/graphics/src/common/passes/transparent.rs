use std::path::PathBuf;

use crate::{
    platform::is_indirect_mode_enabled, BindingData, LoadOperation, Pass, RenderContext,
    RenderMode, RenderPass, RenderPassData, RenderTarget, ShaderStage, StoreOperation,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;

pub const TRANSPARENT_PIPELINE: &str = "pipelines/Transparent.pipeline";
pub const TRANSPARENT_PASS_NAME: &str = "TransparentPass";

pub struct TransparentPass {
    render_pass: Resource<RenderPass>,
    binding_data: BindingData,
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
            binding_data: BindingData::default(),
        }
    }
    fn prepare(&mut self, render_context: &RenderContext) {
        let pass = self.render_pass.get();

        let encoder = render_context.new_encoder();

        let render_texture = pass.render_texture_id();
        let depth_texture = pass.depth_texture_id();

        self.binding_data.clear();

        self.binding_data
            .add_uniform_data(
                render_context,
                0,
                &render_context.constant_data,
                ShaderStage::VertexAndFragment,
            )
            .add_storage_data(
                render_context,
                0,
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

        let pipelines = pass.pipelines();
        pipelines.iter().for_each(|pipeline| {
            if render_context
                .graphics_data
                .get()
                .instance_count(pipeline.id())
                == 0
            {
                return;
            }

            let render_format = render_context.render_format(&pass);
            let depth_format = render_context.depth_format(&pass);

            if !pipeline.get_mut().init(
                render_context,
                render_format,
                depth_format,
                &self.binding_data,
            ) {
                return;
            }

            if is_indirect_mode_enabled() && pass.data().render_mode == RenderMode::Indirect {
                render_context
                    .graphics_data
                    .get_mut()
                    .fill_command_buffer(render_context, pipeline.id());
            }
        });

        render_context.submit(encoder);
    }
    fn update(&mut self, render_context: &RenderContext) {
        let pass = self.render_pass.get();

        let mut encoder = render_context.new_encoder();

        let render_pass = pass.begin(render_context, &self.binding_data, &mut encoder);
        pass.draw(render_context, render_pass);

        render_context.submit(encoder);
    }
}

impl TransparentPass {
    pub fn render_pass(&self) -> &Resource<RenderPass> {
        &self.render_pass
    }
}
