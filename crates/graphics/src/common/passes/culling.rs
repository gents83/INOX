use std::path::PathBuf;

use crate::{
    AsBufferBinding, BindingData, ComputePass, ComputePassData, DataBuffer, MeshletData, Pass,
    RenderContext, ShaderStage, VertexFormat,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;

pub const CULLING_PIPELINE: &str = "pipelines/Culling.compute_pipeline";
pub const CULLING_PASS_NAME: &str = "CullingPass";

#[repr(C, align(16))]
#[derive(Default, Clone)]
pub struct CullPassData {
    pub meshlets: Vec<MeshletData>,
}

impl AsBufferBinding for CullPassData {
    fn size(&self) -> u64 {
        (std::mem::size_of::<MeshletData>() * self.meshlets.len()) as _
    }

    fn fill_buffer(&self, render_context: &RenderContext, buffer: &mut DataBuffer) {
        buffer.add_to_gpu_buffer(render_context, self.meshlets.as_slice());
    }
}

pub struct CullingPass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
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
        }
    }
    fn init(&mut self, render_context: &mut RenderContext) {
        let vertex_format = VertexFormat::pbr();
        let vertex_format = VertexFormat::to_bits(&vertex_format);

        let mut cull_meshlets = CullPassData { meshlets: vec![] };
        if let Some(meshlets) = render_context
            .graphics_data
            .get()
            .get_meshlets(&vertex_format)
        {
            cull_meshlets.meshlets = meshlets.to_vec();
        }

        self.binding_data
            .add_uniform_data(
                render_context,
                0,
                0,
                &render_context.constant_data,
                ShaderStage::Compute,
            )
            .add_storage_data(
                render_context,
                0,
                1,
                &cull_meshlets,
                ShaderStage::Compute,
                true,
            )
            .send_to_gpu(render_context);

        let mut pass = self.compute_pass.get_mut();
        pass.init(render_context, &self.binding_data);
    }
    fn update(&mut self, _render_context: &RenderContext) {}
}
