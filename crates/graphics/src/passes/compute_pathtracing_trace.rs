use std::path::PathBuf;

use inox_render::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData, ConstantDataRw, DEFAULT_HEIGHT, DEFAULT_WIDTH, GPUVector, Pass, RenderContext, RenderContextRc, ShaderStage, TextureView,
};
use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;

use crate::passes::pathtracing_common::{PathTracingCounters, Ray, RayHit, GEOMETRY_BUFFER_UID, SCENE_BUFFER_UID, DISPATCH_INDIRECT_BUFFER_UID, DispatchIndirectArgs};

pub const COMPUTE_PATHTRACING_TRACE_PIPELINE: &str =
    "pipelines/ComputePathtracingTrace.compute_pipeline";
pub const COMPUTE_PATHTRACING_TRACE_NAME: &str = "ComputePathTracingTracePass";

pub struct ComputePathTracingTracePass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    rays: GPUVector<Ray>,
    hits: GPUVector<RayHit>,
    counters: inox_render::GPUBuffer<PathTracingCounters>,
    dispatch_buffer: inox_render::GPUVector<DispatchIndirectArgs>,
}
unsafe impl Send for ComputePathTracingTracePass {}
unsafe impl Sync for ComputePathTracingTracePass {}

impl Pass for ComputePathTracingTracePass {
    fn name(&self) -> &str {
        COMPUTE_PATHTRACING_TRACE_NAME
    }
    fn static_name() -> &'static str {
        COMPUTE_PATHTRACING_TRACE_NAME
    }
    fn is_active(&self, _render_context: &RenderContext) -> bool {
        true
    }
    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self
    where
        Self: Sized,
    {
        let data = ComputePassData {
            name: COMPUTE_PATHTRACING_TRACE_NAME.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_PATHTRACING_TRACE_PIPELINE)],
        };

        let rays = render_context.global_buffers().vector::<Ray>();
        let hits = render_context.global_buffers().vector::<RayHit>();
        let counters = render_context.global_buffers().buffer::<PathTracingCounters>();
        let dispatch_buffer = render_context
            .global_buffers()
            .vector_with_id::<DispatchIndirectArgs>(DISPATCH_INDIRECT_BUFFER_UID);

        // Ensure hits buffer is large enough
        hits.write().unwrap().resize(
            (DEFAULT_WIDTH * DEFAULT_HEIGHT) as usize,
            RayHit::default(),
        );

        Self {
            compute_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &data,
                None,
            ),
            constant_data: render_context.global_buffers().constant_data.clone(),
            rays,
            hits,
            counters,
            dispatch_buffer,
            binding_data: BindingData::new(render_context, COMPUTE_PATHTRACING_TRACE_NAME),
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("pathtracing_trace_pass::init");

        let geometry_buffer = render_context
            .global_buffers()
            .vector_with_id::<u32>(GEOMETRY_BUFFER_UID);
        let scene_buffer = render_context
            .global_buffers()
            .vector_with_id::<u32>(SCENE_BUFFER_UID);

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
                &mut *geometry_buffer.write().unwrap(),
                Some("GeometryBuffer"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *scene_buffer.write().unwrap(),
                Some("SceneBuffer"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.rays.write().unwrap(),
                Some("Rays"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.hits.write().unwrap(),
                Some("Hits"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.counters.write().unwrap(),
                Some("Counters"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                    ..Default::default()
                },
            );

        let mut pass = self.compute_pass.get_mut();
        pass.init(render_context, &mut self.binding_data, None);
    }

    fn update(
        &mut self,
        render_context: &RenderContext,
        _surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
        inox_profiler::scoped_profile!("pathtracing_trace_pass::update");

        let pass = self.compute_pass.get();
        let dispatch_buffer = self.dispatch_buffer.read().unwrap();

        pass.dispatch_workgroups_indirect(
            render_context,
            &mut self.binding_data,
            command_buffer,
            dispatch_buffer.gpu_buffer().unwrap(),
            0,
        );
    }
}

impl ComputePathTracingTracePass {
    pub fn set_rays_buffer(&mut self, rays: &GPUVector<Ray>) {
        self.rays = rays.clone();
        self.binding_data.add_buffer(
            &mut *self.rays.write().unwrap(),
            Some("Rays"),
            BindingInfo {
                group_index: 1,
                binding_index: 0,
                stage: ShaderStage::Compute,
                flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                ..Default::default()
            },
        );
    }
}
