use std::path::PathBuf;

use crate::{
    BindingData, BindingInfo, CommandBuffer, DrawCommandType, DrawVertex, MeshFlags, Pass,
    RenderContext, RenderPass, RenderPassData, RenderTarget, ShaderStage, StoreOperation,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;

pub const GBUFFER_PIPELINE: &str = "pipelines/GBuffer.render_pipeline";
pub const GBUFFER_PASS_NAME: &str = "GBufferPass";

pub struct GBufferPass {
    render_pass: Resource<RenderPass>,
    binding_data: BindingData,
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
    fn is_active(&self, render_context: &mut RenderContext) -> bool {
        render_context.has_commands(&self.draw_command_type(), &self.mesh_flags())
    }
    fn mesh_flags(&self) -> MeshFlags {
        MeshFlags::Visible | MeshFlags::Opaque
    }
    fn draw_command_type(&self) -> DrawCommandType {
        DrawCommandType::PerMeshlet
    }
    fn create(context: &ContextRc) -> Self
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
                data,
                None,
            ),
            binding_data: BindingData::default(),
        }
    }
    fn init(&mut self, render_context: &mut RenderContext) {
        inox_profiler::scoped_profile!("gbuffer_pass::init");

        let mut pass = self.render_pass.get_mut();
        let render_textures = pass.render_textures_id();
        let depth_texture = pass.depth_texture_id();

        self.binding_data.add_uniform_buffer(
            &render_context.core,
            &render_context.binding_data_buffer,
            &mut render_context.constant_data,
            BindingInfo {
                group_index: 0,
                binding_index: 0,
                stage: ShaderStage::VertexAndFragment,
                ..Default::default()
            },
        );
        if !render_context.render_buffers.vertex_positions.is_empty() {
            self.binding_data.add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.vertex_positions,
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Vertex,

                    ..Default::default()
                },
            );
        }
        if !render_context.render_buffers.vertex_colors.is_empty() {
            self.binding_data.add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.vertex_colors,
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
                    stage: ShaderStage::Vertex,

                    ..Default::default()
                },
            );
        }
        if !render_context.render_buffers.vertex_normals.is_empty() {
            self.binding_data.add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.vertex_normals,
                BindingInfo {
                    group_index: 0,
                    binding_index: 3,
                    stage: ShaderStage::Vertex,

                    ..Default::default()
                },
            );
        }
        if !render_context.render_buffers.vertex_uvs.is_empty() {
            self.binding_data.add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.vertex_uvs,
                BindingInfo {
                    group_index: 0,
                    binding_index: 4,
                    stage: ShaderStage::Vertex,

                    ..Default::default()
                },
            );
        }
        if !render_context.render_buffers.meshes.is_empty() {
            self.binding_data.add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.meshes,
                BindingInfo {
                    group_index: 1,
                    binding_index: 0,
                    stage: ShaderStage::VertexAndFragment,

                    ..Default::default()
                },
            );
        }
        if !render_context.render_buffers.materials.is_empty() {
            self.binding_data.add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.materials,
                BindingInfo {
                    group_index: 1,
                    binding_index: 1,
                    stage: ShaderStage::Fragment,

                    ..Default::default()
                },
            );
        }
        if !render_context.render_buffers.textures.is_empty() {
            self.binding_data.add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.textures,
                BindingInfo {
                    group_index: 1,
                    binding_index: 2,
                    stage: ShaderStage::Fragment,

                    ..Default::default()
                },
            );
        }
        if !render_context.render_buffers.meshlets.is_empty() {
            self.binding_data.add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.meshlets,
                BindingInfo {
                    group_index: 1,
                    binding_index: 3,
                    stage: ShaderStage::Vertex,

                    ..Default::default()
                },
            );
        }
        if !render_context.render_buffers.meshes_aabb.is_empty() {
            self.binding_data.add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.meshes_aabb,
                BindingInfo {
                    group_index: 1,
                    binding_index: 4,
                    stage: ShaderStage::Vertex,

                    ..Default::default()
                },
            );
        }

        self.binding_data
            .add_sampler_and_textures(
                &render_context.texture_handler,
                render_textures,
                depth_texture,
                BindingInfo {
                    group_index: 2,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .set_vertex_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                0,
                &mut render_context.render_buffers.vertices,
            )
            .set_index_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.indices,
            );
        self.binding_data
            .send_to_gpu(render_context, GBUFFER_PASS_NAME);

        let vertex_layout = DrawVertex::descriptor(0);
        pass.init(
            render_context,
            &self.binding_data,
            Some(vertex_layout),
            None,
        );
    }
    fn update(&self, render_context: &mut RenderContext, command_buffer: &mut CommandBuffer) {
        inox_profiler::scoped_profile!("gbuffer_pass::update");

        let pass = self.render_pass.get();
        let buffers = render_context.buffers();
        let pipeline = pass.pipeline().get();
        if !pipeline.is_initialized() {
            return;
        }

        let render_pass = pass.begin(
            render_context,
            &self.binding_data,
            &buffers,
            &pipeline,
            command_buffer,
        );
        pass.indirect_indexed_draw(
            render_context,
            &buffers,
            self.draw_command_type(),
            render_pass,
        );
    }
}

impl GBufferPass {
    pub fn render_pass(&self) -> &Resource<RenderPass> {
        &self.render_pass
    }
}
