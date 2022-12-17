use std::path::PathBuf;

use crate::{
    BHVBuffer, BindingData, BindingInfo, CommandBuffer, ConstantDataRw, DrawCommandType,
    DrawVertex, IndicesBuffer, LoadOperation, MeshFlags, MeshesBuffer, OutputRenderPass, Pass,
    RenderContext, RenderPass, RenderPassBeginData, RenderPassData, RenderTarget, ShaderStage,
    StoreOperation, TextureView, VertexColorsBuffer, VertexPositionsBuffer, VerticesBuffer,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource, ResourceTrait};
use inox_uid::generate_random_uid;

pub const WIREFRAME_PIPELINE: &str = "pipelines/Wireframe.render_pipeline";
pub const WIREFRAME_PASS_NAME: &str = "WireframePass";

pub struct WireframePass {
    render_pass: Resource<RenderPass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    meshes: MeshesBuffer,
    bhv: BHVBuffer,
    vertices: VerticesBuffer,
    indices: IndicesBuffer,
    vertex_positions: VertexPositionsBuffer,
    vertex_colors: VertexColorsBuffer,
}
unsafe impl Send for WireframePass {}
unsafe impl Sync for WireframePass {}

impl Pass for WireframePass {
    fn name(&self) -> &str {
        WIREFRAME_PASS_NAME
    }
    fn static_name() -> &'static str {
        WIREFRAME_PASS_NAME
    }
    fn is_active(&self, render_context: &RenderContext) -> bool {
        render_context.has_commands(&self.draw_commands_type(), &self.mesh_flags())
    }
    fn mesh_flags(&self) -> MeshFlags {
        MeshFlags::Visible | MeshFlags::Wireframe
    }
    fn draw_commands_type(&self) -> DrawCommandType {
        DrawCommandType::PerMeshlet
    }
    fn create(context: &ContextRc, render_context: &RenderContext) -> Self
    where
        Self: Sized,
    {
        inox_profiler::scoped_profile!("wireframe_pass::create");

        let data = RenderPassData {
            name: WIREFRAME_PASS_NAME.to_string(),
            load_color: LoadOperation::Load,
            load_depth: LoadOperation::Load,
            store_color: StoreOperation::Store,
            store_depth: StoreOperation::Store,
            render_target: RenderTarget::Screen,
            pipeline: PathBuf::from(WIREFRAME_PIPELINE),
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
            bhv: render_context.render_buffers.bhvs.clone(),
            vertices: render_context.render_buffers.vertices.clone(),
            indices: render_context.render_buffers.indices.clone(),
            vertex_positions: render_context.render_buffers.vertex_positions.clone(),
            vertex_colors: render_context.render_buffers.vertex_colors.clone(),
            binding_data: BindingData::new(render_context, WIREFRAME_PASS_NAME),
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("wireframe_pass::init");

        let mut pass = self.render_pass.get_mut();

        self.binding_data
            .add_uniform_buffer(
                &mut *self.constant_data.write().unwrap(),
                Some("ConstantData"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 0,
                    stage: ShaderStage::Vertex,
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
                &mut *self.meshes.write().unwrap(),
                Some("Meshes"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 3,
                    stage: ShaderStage::Vertex,

                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.bhv.write().unwrap(),
                Some("BHV"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 4,
                    stage: ShaderStage::Vertex,

                    ..Default::default()
                },
            )
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
        inox_profiler::scoped_profile!("wireframe_pass::update");

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
                "wireframe_pass",
            );
            pass.indirect_indexed_draw(render_context, &buffers, draw_commands_type, render_pass);
        }
    }
}

impl OutputRenderPass for WireframePass {
    fn render_pass(&self) -> &Resource<RenderPass> {
        &self.render_pass
    }
}
