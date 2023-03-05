use std::path::PathBuf;

use crate::{
    AsBinding, BHVBuffer, BindingData, BindingFlags, BindingInfo, CommandBuffer, CullingResults,
    DrawCommandType, GpuBuffer, IndicesBuffer, MeshFlags, MeshesBuffer, MeshesInverseMatrixBuffer,
    MeshletsBuffer, MeshletsCullingBuffer, OutputRenderPass, Pass, RaysBuffer, RenderContext,
    RenderCoreContext, RenderPass, RenderPassBeginData, RenderPassData, RenderTarget, ShaderStage,
    StoreOperation, TextureView, VertexPositionsBuffer, VerticesBuffer,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource, ResourceTrait};
use inox_uid::generate_random_uid;

pub const RAYTRACING_VISIBILITY_PIPELINE: &str = "pipelines/RayTracingVisibility.render_pipeline";
pub const RAYTRACING_VISIBILITY_NAME: &str = "RayTracingVisibilityPass";

#[derive(Default)]
struct Data {
    width: u32,
    height: u32,
    is_dirty: bool,
}

impl AsBinding for Data {
    fn is_dirty(&self) -> bool {
        self.is_dirty
    }
    fn set_dirty(&mut self, is_dirty: bool) {
        self.is_dirty = is_dirty;
    }
    fn size(&self) -> u64 {
        std::mem::size_of_val(&self.width) as u64 + std::mem::size_of_val(&self.height) as u64
    }
    fn fill_buffer(&self, render_core_context: &RenderCoreContext, buffer: &mut GpuBuffer) {
        buffer.add_to_gpu_buffer(render_core_context, &[self.width]);
        buffer.add_to_gpu_buffer(render_core_context, &[self.height]);
    }
}
pub struct RayTracingVisibilityPass {
    render_pass: Resource<RenderPass>,
    binding_data: BindingData,
    meshes: MeshesBuffer,
    meshes_inverse_matrix: MeshesInverseMatrixBuffer,
    meshlets: MeshletsBuffer,
    meshlets_culling: MeshletsCullingBuffer,
    culling_result: CullingResults,
    tlas: BHVBuffer,
    bhv: BHVBuffer,
    vertices: VerticesBuffer,
    indices: IndicesBuffer,
    vertex_positions: VertexPositionsBuffer,
    rays: RaysBuffer,
    data: Data,
}
unsafe impl Send for RayTracingVisibilityPass {}
unsafe impl Sync for RayTracingVisibilityPass {}

impl Pass for RayTracingVisibilityPass {
    fn name(&self) -> &str {
        RAYTRACING_VISIBILITY_NAME
    }
    fn static_name() -> &'static str {
        RAYTRACING_VISIBILITY_NAME
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
        let data = RenderPassData {
            name: RAYTRACING_VISIBILITY_NAME.to_string(),
            store_color: StoreOperation::Store,
            store_depth: StoreOperation::Store,
            render_target: RenderTarget::Screen,
            pipeline: PathBuf::from(RAYTRACING_VISIBILITY_PIPELINE),
            ..Default::default()
        };

        Self {
            render_pass: RenderPass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &data,
                None,
            ),
            meshes: render_context.render_buffers.meshes.clone(),
            meshes_inverse_matrix: render_context.render_buffers.meshes_inverse_matrix.clone(),
            meshlets: render_context.render_buffers.meshlets.clone(),
            meshlets_culling: render_context.render_buffers.meshlets_culling.clone(),
            culling_result: render_context.render_buffers.culling_result.clone(),
            tlas: render_context.render_buffers.tlas.clone(),
            bhv: render_context.render_buffers.bhv.clone(),
            vertices: render_context.render_buffers.vertices.clone(),
            indices: render_context.render_buffers.indices.clone(),
            vertex_positions: render_context.render_buffers.vertex_positions.clone(),
            binding_data: BindingData::new(render_context, RAYTRACING_VISIBILITY_NAME),
            rays: render_context.render_buffers.rays.clone(),
            data: Data::default(),
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("raytracing_visibility_pass::init");

        if self.meshlets.read().unwrap().is_empty() {
            return;
        }

        self.binding_data
            .add_uniform_buffer(
                &mut self.data,
                Some("Data"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 0,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.indices.write().unwrap(),
                Some("Indices"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Read | BindingFlags::Index,
                },
            )
            .add_storage_buffer(
                &mut *self.vertices.write().unwrap(),
                Some("Vertices"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Read | BindingFlags::Vertex,
                },
            )
            .add_storage_buffer(
                &mut *self.vertex_positions.write().unwrap(),
                Some("VertexPositions"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 3,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.meshes.write().unwrap(),
                Some("Meshes"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 4,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.meshlets.write().unwrap(),
                Some("Meshlets"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 5,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.meshlets_culling.write().unwrap(),
                Some("Meshlets Culling"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 6,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.culling_result.write().unwrap(),
                Some("Culling Results"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 7,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.tlas.write().unwrap(),
                Some("TLAS"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 0,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.bhv.write().unwrap(),
                Some("BHV"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 1,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.meshes_inverse_matrix.write().unwrap(),
                Some("Meshes Inverse Matrix"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 2,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.rays.write().unwrap(),
                Some("Rays"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 3,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            );

        let mut pass = self.render_pass.get_mut();
        pass.init(render_context, &mut self.binding_data, None, None);
    }

    fn update(
        &mut self,
        render_context: &RenderContext,
        surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
        if self.meshlets.read().unwrap().is_empty() {
            return;
        }

        inox_profiler::scoped_profile!("raytracing_visibility_pass::update");

        let pass = self.render_pass.get();
        let pipeline = pass.pipeline().get();
        if !pipeline.is_initialized() {
            return;
        }
        let buffers = render_context.buffers();
        let render_targets = render_context.texture_handler.render_targets();

        let render_pass_begin_data = RenderPassBeginData {
            render_core_context: &render_context.core,
            buffers: &buffers,
            render_targets: render_targets.as_slice(),
            surface_view,
            command_buffer,
        };
        let mut render_pass = pass.begin(&mut self.binding_data, &pipeline, render_pass_begin_data);
        {
            inox_profiler::gpu_scoped_profile!(
                &mut render_pass,
                &render_context.core.device,
                "raytracing_visibility_pass",
            );
            pass.draw(render_context, render_pass, 0..3, 0..1);
        }
    }
}

impl OutputRenderPass for RayTracingVisibilityPass {
    fn render_pass(&self) -> &Resource<RenderPass> {
        &self.render_pass
    }
}

impl RayTracingVisibilityPass {
    pub fn set_resolution(&mut self, width: u32, height: u32) -> &mut Self {
        self.data.width = width;
        self.data.height = height;
        self.data.set_dirty(true);
        self
    }
}
