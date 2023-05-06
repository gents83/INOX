use std::path::PathBuf;

use crate::{
    BindingData, BindingInfo, CommandBuffer, DrawCommandType, MeshFlags, MeshesBuffer,
    MeshletsBuffer, OutputRenderPass, Pass, RenderContext, RenderPass, RenderPassBeginData,
    RenderPassData, RenderTarget, ShaderStage, StoreOperation, TextureId, TextureView,
    VertexColorsBuffer, VertexNormalsBuffer, VertexUVsBuffer, VerticesBuffer,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource, ResourceTrait};
use inox_uid::{generate_random_uid, INVALID_UID};

pub const VISIBILITY_TO_GBUFFER_PIPELINE: &str = "pipelines/VisibilityToGBuffer.render_pipeline";
pub const VISIBILITY_TO_GBUFFER_PASS_NAME: &str = "VisibilityToGBufferPass";

#[allow(dead_code)]
pub struct VisibilityToGBufferPass {
    render_pass: Resource<RenderPass>,
    binding_data: BindingData,
    visibility_texture_id: TextureId,
    depth_texture_id: TextureId,
    meshlets: MeshletsBuffer,
    meshes: MeshesBuffer,
    vertices: VerticesBuffer,
    vertex_colors: VertexColorsBuffer,
    vertex_normals: VertexNormalsBuffer,
    vertex_uvs: VertexUVsBuffer,
}
unsafe impl Send for VisibilityToGBufferPass {}
unsafe impl Sync for VisibilityToGBufferPass {}

impl Pass for VisibilityToGBufferPass {
    fn name(&self) -> &str {
        VISIBILITY_TO_GBUFFER_PASS_NAME
    }
    fn static_name() -> &'static str {
        VISIBILITY_TO_GBUFFER_PASS_NAME
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
        inox_profiler::scoped_profile!("visibility_to_gbuffer_pass::create");

        let data = RenderPassData {
            name: VISIBILITY_TO_GBUFFER_PASS_NAME.to_string(),
            store_color: StoreOperation::Store,
            store_depth: StoreOperation::Store,
            render_target: RenderTarget::Screen,
            pipeline: PathBuf::from(VISIBILITY_TO_GBUFFER_PIPELINE),
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
            binding_data: BindingData::new(render_context, VISIBILITY_TO_GBUFFER_PASS_NAME),
            visibility_texture_id: INVALID_UID,
            depth_texture_id: INVALID_UID,
            meshes: render_context.render_buffers.meshes.clone(),
            meshlets: render_context.render_buffers.meshlets.clone(),
            vertices: render_context.render_buffers.vertices.clone(),
            vertex_colors: render_context.render_buffers.vertex_colors.clone(),
            vertex_normals: render_context.render_buffers.vertex_normals.clone(),
            vertex_uvs: render_context.render_buffers.vertex_uvs.clone(),
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("visibility_to_gbuffer_pass::init");

        if self.visibility_texture_id.is_nil() || self.depth_texture_id.is_nil() {
            return;
        }

        let mut pass = self.render_pass.get_mut();
        self.binding_data
            .add_texture(
                &self.visibility_texture_id,
                BindingInfo {
                    group_index: 0,
                    binding_index: 0,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_texture(
                &self.depth_texture_id,
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            );

        pass.init(render_context, &mut self.binding_data, None, None);
    }
    fn update(
        &mut self,
        render_context: &RenderContext,
        surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
        inox_profiler::scoped_profile!("visibility_to_gbuffer_pass::update");

        if self.visibility_texture_id.is_nil() || self.depth_texture_id.is_nil() {
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
                "blit_pass",
            );
            pass.draw(render_context, render_pass, 0..3, 0..1);
        }
    }
}

impl OutputRenderPass for VisibilityToGBufferPass {
    fn render_pass(&self) -> &Resource<RenderPass> {
        &self.render_pass
    }
}

impl VisibilityToGBufferPass {
    pub fn set_visibility_texture(&mut self, id: &TextureId) -> &mut Self {
        self.visibility_texture_id = *id;
        self
    }
    pub fn set_depth_texture(&mut self, id: &TextureId) -> &mut Self {
        self.depth_texture_id = *id;
        self
    }
}
