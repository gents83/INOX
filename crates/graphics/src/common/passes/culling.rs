use std::path::PathBuf;

use crate::{
    BindingData, BindingInfo, ComputePass, ComputePassData, Pass, RenderContext, ShaderStage,
    VertexFormat,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;

pub const CULLING_PIPELINE: &str = "pipelines/Culling.compute_pipeline";
pub const CULLING_PASS_NAME: &str = "CullingPass";

pub struct CullingPass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    is_active: bool,
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
            binding_data: BindingData::default(),
            is_active: false,
        }
    }
    fn init(&mut self, render_context: &mut RenderContext) {
        let vertex_format = VertexFormat::pbr();
        let vertex_format = VertexFormat::to_bits(&vertex_format);

        if let Some(meshlets) = render_context
            .graphics_data
            .get_mut()
            .get_meshlets(&vertex_format)
        {
            self.binding_data
                .add_uniform_data(
                    &render_context.core,
                    &render_context.binding_data_buffer,
                    &mut render_context.constant_data,
                    BindingInfo {
                        group_index: 0,
                        binding_index: 0,
                        stage: ShaderStage::Compute,
                        ..Default::default()
                    },
                )
                .add_storage_data(
                    &render_context.core,
                    &render_context.binding_data_buffer,
                    meshlets,
                    BindingInfo {
                        group_index: 0,
                        binding_index: 1,
                        stage: ShaderStage::Compute,
                        read_only: true,
                    },
                )
                .send_to_gpu(render_context);
            self.is_active = true;
        } else {
            self.is_active = false;
            return;
        }

        let mut pass = self.compute_pass.get_mut();
        pass.init(render_context, &self.binding_data);
    }

    fn update(&mut self, render_context: &RenderContext) {
        if !self.is_active {
            return;
        }
        let pass = self.compute_pass.get();

        let mut encoder = render_context.core.new_encoder();
        let compute_pass = pass.begin(&self.binding_data, &mut encoder);
        pass.dispatch(compute_pass, 32, 1, 1);

        render_context.core.submit(encoder);
    }
}
