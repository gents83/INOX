use std::path::PathBuf;

use crate::{
    BHVBuffer, BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData,
    ConstantDataRw, DrawCommandType, MeshFlags, MeshesBuffer, Pass, RenderContext,
    RuntimeVerticesBuffer, ShaderStage, TextureView, VertexPositionsBuffer,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;

pub const COMPUTE_RUNTIME_VERTICES_PIPELINE: &str =
    "pipelines/ComputeRuntimeVertices.compute_pipeline";
pub const COMPUTE_RUNTIME_VERTICES_PASS_NAME: &str = "ComputeRuntimeVerticesPass";

pub struct ComputeRuntimeVerticesPass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    bhv: BHVBuffer,
    meshes: MeshesBuffer,
    vertices_positions: VertexPositionsBuffer,
    runtime_vertices: RuntimeVerticesBuffer,
}
unsafe impl Send for ComputeRuntimeVerticesPass {}
unsafe impl Sync for ComputeRuntimeVerticesPass {}

impl Pass for ComputeRuntimeVerticesPass {
    fn name(&self) -> &str {
        COMPUTE_RUNTIME_VERTICES_PASS_NAME
    }
    fn static_name() -> &'static str {
        COMPUTE_RUNTIME_VERTICES_PASS_NAME
    }
    fn is_active(&self, render_context: &RenderContext) -> bool {
        render_context.has_commands(&self.draw_commands_type(), &self.mesh_flags())
    }
    fn mesh_flags(&self) -> MeshFlags {
        MeshFlags::Visible | MeshFlags::Opaque
    }
    fn draw_commands_type(&self) -> DrawCommandType {
        DrawCommandType::PerMeshlet
    }
    fn create(context: &ContextRc, render_context: &RenderContext) -> Self
    where
        Self: Sized,
    {
        let compute_data = ComputePassData {
            name: COMPUTE_RUNTIME_VERTICES_PASS_NAME.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_RUNTIME_VERTICES_PIPELINE)],
        };

        Self {
            compute_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &compute_data,
                None,
            ),
            binding_data: BindingData::new(render_context, COMPUTE_RUNTIME_VERTICES_PASS_NAME),
            constant_data: render_context.constant_data.clone(),
            bhv: render_context.render_buffers.bhv.clone(),
            meshes: render_context.render_buffers.meshes.clone(),
            vertices_positions: render_context.render_buffers.vertex_positions.clone(),
            runtime_vertices: render_context.render_buffers.runtime_vertices.clone(),
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("compute_runtime_vertices_pass::init");

        if self.vertices_positions.read().unwrap().is_empty() {
            return;
        }
        self.binding_data
            .add_uniform_buffer(
                &mut *self.constant_data.write().unwrap(),
                Some("ConstantData"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.bhv.write().unwrap(),
                Some("BHV"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.meshes.write().unwrap(),
                Some("Meshes"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.vertices_positions.write().unwrap(),
                Some("Vertices Positions"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 3,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.runtime_vertices.write().unwrap(),
                Some("Runtime Vertices"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 4,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::ReadWrite | BindingFlags::Vertex,
                },
            );

        let mut pass = self.compute_pass.get_mut();
        pass.init(render_context, &mut self.binding_data);
    }

    fn update(
        &mut self,
        render_context: &RenderContext,
        _surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
        let num_vertices = self.runtime_vertices.read().unwrap().item_count();
        if num_vertices == 0 {
            return;
        }

        let workgroup_size = 256;
        let count = (num_vertices as u32 + workgroup_size - 1) / workgroup_size;

        let pass = self.compute_pass.get();
        pass.dispatch(
            render_context,
            &mut self.binding_data,
            command_buffer,
            count,
            1,
            1,
        );
    }
}
