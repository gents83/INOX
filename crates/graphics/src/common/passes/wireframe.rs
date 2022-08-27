use std::path::PathBuf;

use crate::{
    BindingData, BindingInfo, CommandBuffer, DrawCommandType, DrawVertex, LoadOperation, MeshFlags,
    Pass, RenderContext, RenderPass, RenderPassData, RenderTarget, ShaderStage, StoreOperation,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;

pub const WIREFRAME_PIPELINE: &str = "pipelines/Wireframe.render_pipeline";
pub const WIREFRAME_PASS_NAME: &str = "WireframePass";

pub struct WireframePass {
    render_pass: Resource<RenderPass>,
    binding_data: BindingData,
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
    fn is_active(&self, render_context: &mut RenderContext) -> bool {
        render_context.has_commands(&self.draw_command_type(), &self.mesh_flags())
    }
    fn mesh_flags(&self) -> MeshFlags {
        MeshFlags::Visible | MeshFlags::Wireframe
    }
    fn draw_command_type(&self) -> DrawCommandType {
        DrawCommandType::PerMeshlet
    }
    fn create(context: &ContextRc) -> Self
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
                data,
                None,
            ),
            binding_data: BindingData::default(),
        }
    }
    fn init(&mut self, render_context: &mut RenderContext) {
        inox_profiler::scoped_profile!("wireframe_pass::init");

        let mut pass = self.render_pass.get_mut();

        self.binding_data
            .add_uniform_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.constant_data,
                BindingInfo {
                    group_index: 0,
                    binding_index: 0,
                    stage: ShaderStage::Vertex,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.vertex_positions,
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Vertex,

                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.vertex_colors,
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
                    stage: ShaderStage::Vertex,

                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.meshes,
                BindingInfo {
                    group_index: 0,
                    binding_index: 3,
                    stage: ShaderStage::Vertex,

                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.meshes_aabb,
                BindingInfo {
                    group_index: 0,
                    binding_index: 4,
                    stage: ShaderStage::Vertex,

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
            .send_to_gpu(render_context, WIREFRAME_PASS_NAME);

        let vertex_layout = DrawVertex::descriptor(0);
        pass.init(
            render_context,
            &self.binding_data,
            Some(vertex_layout),
            None,
        );
    }
    fn update(&self, render_context: &mut RenderContext, command_buffer: &mut CommandBuffer) {
        inox_profiler::scoped_profile!("wireframe_pass::update");

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

impl WireframePass {
    pub fn render_pass(&self) -> &Resource<RenderPass> {
        &self.render_pass
    }
}
