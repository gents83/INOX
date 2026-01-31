use std::path::PathBuf;

use inox_render::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData, ConstantDataRw, DEFAULT_HEIGHT, DEFAULT_WIDTH, GPUBuffer, GPUInstance, GPUMesh, GPUMeshlet, GPUVector, GPUVertexAttributes, GPUVertexIndices, GPUVertexPosition, GPUTransform, INSTANCE_DATA_ID, Pass, RenderContext, RenderContextRc, ShaderStage, TextureView,
};
use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;

use crate::passes::pathtracing_common::{PathTracingCounters, Ray, RayHit};
use inox_bvh::GPUBVHNode;

pub const COMPUTE_PATHTRACING_TRACE_PIPELINE: &str =
    "pipelines/ComputePathtracingTrace.compute_pipeline";
pub const COMPUTE_PATHTRACING_TRACE_NAME: &str = "ComputePathTracingTracePass";

pub struct ComputePathTracingTracePass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    indices: GPUBuffer<GPUVertexIndices>,
    vertices_positions: GPUBuffer<GPUVertexPosition>,
    vertices_attributes: GPUBuffer<GPUVertexAttributes>,
    instances: GPUVector<GPUInstance>,
    transforms: GPUVector<GPUTransform>,
    meshes: GPUBuffer<GPUMesh>,
    meshlets: GPUBuffer<GPUMeshlet>,
    bvh: GPUBuffer<GPUBVHNode>,
    rays: GPUVector<Ray>,
    hits: GPUVector<RayHit>,
    counters: GPUBuffer<PathTracingCounters>,
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
            indices: render_context.global_buffers().buffer::<GPUVertexIndices>(),
            vertices_positions: render_context
                .global_buffers()
                .buffer::<GPUVertexPosition>(),
            vertices_attributes: render_context
                .global_buffers()
                .buffer::<GPUVertexAttributes>(),
            instances: render_context
                .global_buffers()
                .vector_with_id::<GPUInstance>(INSTANCE_DATA_ID),
            transforms: render_context.global_buffers().vector::<GPUTransform>(),
            meshes: render_context.global_buffers().buffer::<GPUMesh>(),
            meshlets: render_context.global_buffers().buffer::<GPUMeshlet>(),
            bvh: render_context.global_buffers().buffer::<GPUBVHNode>(),
            rays,
            hits,
            counters,
            binding_data: BindingData::new(render_context, COMPUTE_PATHTRACING_TRACE_NAME),
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("pathtracing_trace_pass::init");

        if self.meshlets.read().unwrap().is_empty()
            || self.instances.read().unwrap().is_empty()
        {
            return;
        }

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
                &mut *self.indices.write().unwrap(),
                Some("Indices"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read | BindingFlags::Index,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.vertices_positions.write().unwrap(),
                Some("Vertices Positions"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read | BindingFlags::Vertex,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.vertices_attributes.write().unwrap(),
                Some("Vertices Attributes"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 3,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.instances.write().unwrap(),
                Some("Instances"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 4,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.transforms.write().unwrap(),
                Some("Transforms"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 5,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.meshes.write().unwrap(),
                Some("Meshes"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 6,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.meshlets.write().unwrap(),
                Some("Meshlets"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 7,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.bvh.write().unwrap(),
                Some("BVH"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 8,
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
                    flags: BindingFlags::Storage | BindingFlags::ReadWrite,
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
        if self.meshlets.read().unwrap().is_empty()
            || self.instances.read().unwrap().is_empty()
        {
            return;
        }

        inox_profiler::scoped_profile!("pathtracing_trace_pass::update");

        let pass = self.compute_pass.get();
        // Assuming dispatch size based on max potential rays (screen size)

        let width = self.constant_data.read().unwrap().screen_size()[0] as u32;
        let height = self.constant_data.read().unwrap().screen_size()[1] as u32;

        let x_pixels_managed_in_shader = 8;
        let y_pixels_managed_in_shader = 8;
        let x = width.div_ceil(x_pixels_managed_in_shader);
        let y = height.div_ceil(y_pixels_managed_in_shader);

        pass.dispatch(
            render_context,
            &mut self.binding_data,
            command_buffer,
            x,
            y,
            1,
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
