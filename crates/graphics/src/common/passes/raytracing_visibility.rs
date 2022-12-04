use std::path::PathBuf;

use crate::{
    BindingData, BindingInfo, CommandBuffer, ConstantDataRw, DrawCommandType, IndicesBuffer,
    MeshFlags, MeshesAABBsBuffer, MeshesBuffer, MeshletsAABBsBuffer, MeshletsBuffer, OutputPass,
    OutputRenderPass, Pass, RenderContext, RenderPass, RenderPassBeginData, RenderPassData,
    RenderTarget, ShaderStage, StoreOperation, TextureId, TextureView, VertexPositionsBuffer,
    VerticesBuffer,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource, ResourceTrait};
use inox_uid::generate_random_uid;

pub const RAYTRACING_VISIBILITY_PIPELINE: &str = "pipelines/RayTracingVisibility.render_pipeline";
pub const RAYTRACING_VISIBILITY_NAME: &str = "RayTracingVisibilityPass";

pub struct RayTracingVisibilityPass {
    render_pass: Resource<RenderPass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    meshes: MeshesBuffer,
    meshlets: MeshletsBuffer,
    meshes_aabb: MeshesAABBsBuffer,
    meshlets_aabb: MeshletsAABBsBuffer,
    vertices: VerticesBuffer,
    indices: IndicesBuffer,
    vertex_positions: VertexPositionsBuffer,
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
            constant_data: render_context.constant_data.clone(),
            meshes: render_context.render_buffers.meshes.clone(),
            meshlets: render_context.render_buffers.meshlets.clone(),
            meshes_aabb: render_context.render_buffers.meshes_aabb.clone(),
            meshlets_aabb: render_context.render_buffers.meshlets_aabb.clone(),
            vertices: render_context.render_buffers.vertices.clone(),
            indices: render_context.render_buffers.indices.clone(),
            vertex_positions: render_context.render_buffers.vertex_positions.clone(),
            binding_data: BindingData::new(render_context, RAYTRACING_VISIBILITY_NAME),
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("raytracing_visibility_pass::init");

        if self.meshlets.read().unwrap().is_empty() {
            return;
        }

        self.binding_data
            .add_uniform_buffer(
                &mut *self.constant_data.write().unwrap(),
                Some("ConstantData"),
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
                    is_index: true,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.vertices.write().unwrap(),
                Some("Vertices"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
                    stage: ShaderStage::Fragment,
                    is_vertex: true,
                    ..Default::default()
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
                &mut *self.meshes_aabb.write().unwrap(),
                Some("MeshesAABB"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 6,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.meshlets_aabb.write().unwrap(),
                Some("MeshletsAABB"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 7,
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
        inox_profiler::scoped_profile!("raytracing_visibility_pass::update");

        let num_meshlets = self.meshlets.read().unwrap().item_count();
        if num_meshlets == 0 {
            return;
        }
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

impl OutputPass for RayTracingVisibilityPass {
    fn render_targets_id(&self) -> Vec<TextureId> {
        let pass = self.render_pass.get();
        pass.render_textures_id().iter().map(|&id| *id).collect()
    }
}
