use std::path::PathBuf;

use crate::{
    BHVBuffer, BindingData, BindingInfo, CommandBuffer, ConstantDataRw, DrawCommandType,
    DrawVertex, IndicesBuffer, MaterialsBuffer, MeshFlags, MeshesBuffer, MeshletsBuffer,
    OutputRenderPass, Pass, RenderContext, RenderPass, RenderPassBeginData, RenderPassData,
    RenderTarget, ShaderStage, StoreOperation, TextureView, TexturesBuffer, VertexColorsBuffer,
    VertexNormalsBuffer, VertexPositionsBuffer, VertexUVsBuffer, VerticesBuffer,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource, ResourceTrait};
use inox_uid::generate_random_uid;

pub const GBUFFER_PIPELINE: &str = "pipelines/GBuffer.render_pipeline";
pub const GBUFFER_PASS_NAME: &str = "GBufferPass";

pub struct GBufferPass {
    render_pass: Resource<RenderPass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    textures: TexturesBuffer,
    materials: MaterialsBuffer,
    meshes: MeshesBuffer,
    bhv: BHVBuffer,
    meshlets: MeshletsBuffer,
    vertices: VerticesBuffer,
    indices: IndicesBuffer,
    vertex_positions: VertexPositionsBuffer,
    vertex_colors: VertexColorsBuffer,
    vertex_normals: VertexNormalsBuffer,
    vertex_uvs: VertexUVsBuffer,
}
unsafe impl Send for GBufferPass {}
unsafe impl Sync for GBufferPass {}

impl Pass for GBufferPass {
    fn name(&self) -> &str {
        GBUFFER_PASS_NAME
    }
    fn static_name() -> &'static str {
        GBUFFER_PASS_NAME
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
        inox_profiler::scoped_profile!("gbuffer_pass::create");

        let data = RenderPassData {
            name: GBUFFER_PASS_NAME.to_string(),
            store_color: StoreOperation::Store,
            store_depth: StoreOperation::Store,
            render_target: RenderTarget::Screen,
            pipeline: PathBuf::from(GBUFFER_PIPELINE),
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
            textures: render_context.render_buffers.textures.clone(),
            materials: render_context.render_buffers.materials.clone(),
            meshes: render_context.render_buffers.meshes.clone(),
            bhv: render_context.render_buffers.bhv.clone(),
            meshlets: render_context.render_buffers.meshlets.clone(),
            vertices: render_context.render_buffers.vertices.clone(),
            indices: render_context.render_buffers.indices.clone(),
            vertex_positions: render_context.render_buffers.vertex_positions.clone(),
            vertex_colors: render_context.render_buffers.vertex_colors.clone(),
            vertex_normals: render_context.render_buffers.vertex_normals.clone(),
            vertex_uvs: render_context.render_buffers.vertex_uvs.clone(),
            binding_data: BindingData::new(render_context, GBUFFER_PASS_NAME),
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("gbuffer_pass::init");

        let mut pass = self.render_pass.get_mut();

        if self.textures.read().unwrap().is_empty()
            || self.meshes.read().unwrap().is_empty()
            || self.meshlets.read().unwrap().is_empty()
            || self.materials.read().unwrap().is_empty()
            || self.vertex_positions.read().unwrap().is_empty()
            || self.vertex_normals.read().unwrap().is_empty()
            || self.vertex_colors.read().unwrap().is_empty()
            || self.vertex_uvs.read().unwrap().is_empty()
        {
            return;
        }

        self.binding_data
            .add_uniform_buffer(
                &mut *self.constant_data.write().unwrap(),
                Some("ConstantData"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 0,
                    stage: ShaderStage::VertexAndFragment,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.vertex_positions.write().unwrap(),
                Some("VertexPositions"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Vertex,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.vertex_colors.write().unwrap(),
                Some("VertexColors"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
                    stage: ShaderStage::Vertex,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.vertex_normals.write().unwrap(),
                Some("VertexNormals"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 3,
                    stage: ShaderStage::Vertex,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.vertex_uvs.write().unwrap(),
                Some("VertexUVs"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 4,
                    stage: ShaderStage::Vertex,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.meshes.write().unwrap(),
                Some("Meshes"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 0,
                    stage: ShaderStage::VertexAndFragment,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.materials.write().unwrap(),
                Some("Materials"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 1,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.textures.write().unwrap(),
                Some("Textures"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 2,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.meshlets.write().unwrap(),
                Some("Meshlets"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 3,
                    stage: ShaderStage::Vertex,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.bhv.write().unwrap(),
                Some("BHV"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 4,
                    stage: ShaderStage::Vertex,
                    ..Default::default()
                },
            )
            .add_default_sampler(BindingInfo {
                group_index: 2,
                binding_index: 0,
                stage: ShaderStage::Fragment,
                ..Default::default()
            })
            .add_material_textures(BindingInfo {
                group_index: 2,
                binding_index: 1,
                stage: ShaderStage::Fragment,
                ..Default::default()
            })
            .set_vertex_buffer(0, &mut *self.vertices.write().unwrap(), Some("Vertices"))
            .set_index_buffer(&mut *self.indices.write().unwrap(), Some("Indices"));

        let vertex_layout = DrawVertex::descriptor(0);
        pass.init(
            render_context,
            &mut self.binding_data,
            Some(vertex_layout),
            None,
        );
    }
    fn update(
        &mut self,
        render_context: &RenderContext,
        surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
        inox_profiler::scoped_profile!("gbuffer_pass::update");

        let pass = self.render_pass.get();
        let pipeline = pass.pipeline().get();
        if !pipeline.is_initialized() {
            return;
        }
        let buffers = render_context.buffers();
        let render_targets = render_context.texture_handler.render_targets();
        let draw_commands_type = self.draw_commands_type();

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
                "gbuffer_pass",
            );
            pass.indirect_indexed_draw(render_context, &buffers, draw_commands_type, render_pass);
        }
    }
}

impl OutputRenderPass for GBufferPass {
    fn render_pass(&self) -> &Resource<RenderPass> {
        &self.render_pass
    }
}
