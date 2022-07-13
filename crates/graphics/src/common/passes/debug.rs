use std::path::PathBuf;

use crate::{
    BindingData, BindingInfo, CommandBuffer, LoadOperation, Pass, RenderContext, RenderPass,
    RenderPassData, RenderTarget, ShaderStage, StoreOperation,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;

pub const DEBUG_PIPELINE: &str = "pipelines/Debug.render_pipeline";
pub const DEBUG_PASS_NAME: &str = "DebugPass";

pub struct DebugPass {
    render_pass: Resource<RenderPass>,
    binding_data: BindingData,
}
unsafe impl Send for DebugPass {}
unsafe impl Sync for DebugPass {}

impl Pass for DebugPass {
    fn name(&self) -> &str {
        DEBUG_PASS_NAME
    }
    fn static_name() -> &'static str {
        DEBUG_PASS_NAME
    }
    fn create(context: &ContextRc) -> Self
    where
        Self: Sized,
    {
        inox_profiler::scoped_profile!("debug_pass::create");

        let data = RenderPassData {
            name: DEBUG_PASS_NAME.to_string(),
            load_color: LoadOperation::Load,
            load_depth: LoadOperation::DontCare,
            store_color: StoreOperation::Store,
            store_depth: StoreOperation::DontCare,
            render_target: RenderTarget::Screen,
            pipeline: PathBuf::from(DEBUG_PIPELINE),
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
        inox_profiler::scoped_profile!("debug_pass::init");

        if render_context.render_buffers.meshes.is_empty()
            || render_context.render_buffers.meshlets.is_empty()
            || render_context.render_buffers.matrix.is_empty()
        {
            return;
        }

        self.binding_data
            .add_uniform_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.constant_data,
                BindingInfo {
                    group_index: 0,
                    binding_index: 0,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.meshes,
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.meshlets,
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.matrix,
                BindingInfo {
                    group_index: 0,
                    binding_index: 3,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            );
        self.binding_data
            .send_to_gpu(render_context, DEBUG_PASS_NAME);

        let mut pass = self.render_pass.get_mut();
        pass.init(render_context, &self.binding_data, None, None);
    }
    fn update(&self, render_context: &mut RenderContext, command_buffer: &mut CommandBuffer) {
        inox_profiler::scoped_profile!("debug_pass::update");

        if render_context.render_buffers.meshes.is_empty()
            || render_context.render_buffers.meshlets.is_empty()
            || render_context.render_buffers.matrix.is_empty()
        {
            return;
        }

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
        pass.draw(render_pass, 0..3, 0..1);
    }
}

impl DebugPass {
    pub fn render_pass(&self) -> &Resource<RenderPass> {
        &self.render_pass
    }
}
