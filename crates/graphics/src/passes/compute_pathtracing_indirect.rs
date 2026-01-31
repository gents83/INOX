use std::path::PathBuf;

use inox_render::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData,
    ConstantDataRw, Pass, RenderContext, RenderContextRc, ShaderStage, TextureView,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;

use crate::{
    passes::pathtracing_common::{DispatchIndirectArgs, PathTracingCounters, DISPATCH_INDIRECT_BUFFER_UID},
};

pub const COMPUTE_PATHTRACING_INDIRECT_PIPELINE: &str =
    "pipelines/ComputePathtracingIndirect.compute_pipeline";
pub const COMPUTE_PATHTRACING_INDIRECT_NAME: &str = "ComputePathTracingIndirectPass";

pub struct ComputePathTracingIndirectPass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    counters: inox_render::GPUBuffer<PathTracingCounters>,
    dispatch_buffer: inox_render::GPUVector<DispatchIndirectArgs>,
}

unsafe impl Send for ComputePathTracingIndirectPass {}
unsafe impl Sync for ComputePathTracingIndirectPass {}

impl Pass for ComputePathTracingIndirectPass {
    fn name(&self) -> &str {
        COMPUTE_PATHTRACING_INDIRECT_NAME
    }
    fn static_name() -> &'static str {
        COMPUTE_PATHTRACING_INDIRECT_NAME
    }
    fn is_active(&self, _render_context: &RenderContext) -> bool {
        true
    }
    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self
    where
        Self: Sized,
    {
        let data = ComputePassData {
            name: COMPUTE_PATHTRACING_INDIRECT_NAME.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_PATHTRACING_INDIRECT_PIPELINE)],
        };

        let counters = render_context
            .global_buffers()
            .buffer::<PathTracingCounters>();
        let dispatch_buffer = render_context
            .global_buffers()
            .vector_with_id::<DispatchIndirectArgs>(DISPATCH_INDIRECT_BUFFER_UID);

        // Initialize dispatch buffer with at least 1 element
        dispatch_buffer.write().unwrap().resize(1, DispatchIndirectArgs::default());

        Self {
            compute_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &data,
                None,
            ),
            constant_data: render_context.global_buffers().constant_data.clone(),
            counters,
            dispatch_buffer,
            binding_data: BindingData::new(render_context, COMPUTE_PATHTRACING_INDIRECT_NAME),
        }
    }
    fn init(&mut self, _render_context: &RenderContext) {
        self.binding_data
            .add_buffer(
                &mut *self.constant_data.write().unwrap(),
                Some("ConstantData"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Uniform | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.counters.write().unwrap(),
                Some("Counters"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.dispatch_buffer.write().unwrap(),
                Some("DispatchBuffer"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                    ..Default::default()
                },
            );

        let mut pass = self.compute_pass.get_mut();
        pass.init(_render_context, &mut self.binding_data, None);
    }

    fn update(
        &mut self,
        render_context: &RenderContext,
        _surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
        inox_profiler::scoped_profile!("pathtracing_indirect_pass::update");

        let pass = self.compute_pass.get();
        pass.dispatch(
            render_context,
            &mut self.binding_data,
            command_buffer,
            1,
            1,
            1,
        );
    }
}
