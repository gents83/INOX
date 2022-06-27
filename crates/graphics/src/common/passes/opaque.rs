use std::path::PathBuf;

use crate::{
    BindingData, BindingInfo, DrawInstance, DrawVertex, MeshFlags, Pass, RenderContext, RenderPass,
    RenderPassData, RenderTarget, ShaderStage, StoreOperation,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;

pub const DEFAULT_PIPELINE: &str = "pipelines/Default.render_pipeline";
pub const OPAQUE_PASS_NAME: &str = "OpaquePass";

pub struct OpaquePass {
    render_pass: Resource<RenderPass>,
    binding_data: BindingData,
}
unsafe impl Send for OpaquePass {}
unsafe impl Sync for OpaquePass {}

impl Pass for OpaquePass {
    fn name(&self) -> &str {
        OPAQUE_PASS_NAME
    }
    fn static_name() -> &'static str {
        OPAQUE_PASS_NAME
    }
    fn create(context: &ContextRc) -> Self
    where
        Self: Sized,
    {
        inox_profiler::scoped_profile!("opaque_pass::create");

        let data = RenderPassData {
            name: OPAQUE_PASS_NAME.to_string(),
            store_color: StoreOperation::Store,
            store_depth: StoreOperation::Store,
            render_target: RenderTarget::Screen,
            pipeline: PathBuf::from(DEFAULT_PIPELINE),
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
        inox_profiler::scoped_profile!("opaque_pass::init");

        let mesh_flags = MeshFlags::Visible | MeshFlags::Opaque;

        if !render_context.has_instances(mesh_flags)
            || render_context
                .render_buffers
                .vertex_positions_and_colors
                .is_empty()
            || render_context.render_buffers.matrix.is_empty()
        {
            return;
        }

        let mut pass = self.render_pass.get_mut();
        let instances = render_context
            .render_buffers
            .instances
            .get_mut(&mesh_flags)
            .unwrap();

        self.binding_data
            .add_uniform_data(
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
            .add_storage_data(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.vertex_positions_and_colors,
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Vertex,
                    read_only: true,
                    ..Default::default()
                },
            )
            .add_storage_data(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.matrix,
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
                    stage: ShaderStage::Vertex,
                    read_only: true,
                    ..Default::default()
                },
            )
            .set_vertex_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                0,
                &mut render_context.render_buffers.vertices,
            )
            .set_vertex_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                1,
                instances,
            )
            .set_index_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.indices,
            );
        self.binding_data.send_to_gpu(render_context);

        let vertex_layout = DrawVertex::descriptor(0);
        let instance_layout = DrawInstance::descriptor(vertex_layout.location());
        pass.init_pipeline(
            render_context,
            &self.binding_data,
            vertex_layout,
            instance_layout,
        );
    }
    fn update(&mut self, render_context: &mut RenderContext) {
        inox_profiler::scoped_profile!("opaque_pass::update");

        if !render_context.has_instances(MeshFlags::Visible | MeshFlags::Opaque) {
            return;
        }

        let pass = self.render_pass.get();
        let mut encoder = render_context.core.new_encoder();
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
            &mut encoder,
        );
        pass.draw_meshlets(render_context, render_pass);

        render_context.core.submit(encoder);
    }
}

impl OpaquePass {
    pub fn render_pass(&self) -> &Resource<RenderPass> {
        &self.render_pass
    }
}
