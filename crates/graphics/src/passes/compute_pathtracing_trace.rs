use std::path::PathBuf;

use inox_bvh::GPUBVHNode;
use inox_render::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData, ConstantDataRw, GPUBuffer, GPUInstance, GPUMeshlet, GPUVector, GPUVertexIndices, GPUVertexPosition, INSTANCE_DATA_ID, Pass, RenderContext, RenderContextRc, ShaderStage, TextureView
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::{generate_random_uid, Uid};

use crate::pathtracing_common::*;

pub const COMPUTE_PATHTRACING_TRACE_PIPELINE: &str =
    "pipelines/ComputePathTracingTrace.compute_pipeline";
pub const COMPUTE_PATHTRACING_TRACE_NAME: &str = "ComputePathTracingTracePass";

pub struct ComputePathTracingTracePass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    bvh: GPUBuffer<GPUBVHNode>,
    indices: GPUBuffer<GPUVertexIndices>,
    vertices_positions: GPUBuffer<GPUVertexPosition>,
    instances: GPUVector<GPUInstance>,
    meshlets: GPUBuffer<GPUMeshlet>,
    rays: GPUVector<RayPackedData>,
    hits: GPUVector<HitPackedData>,
    dimensions: (u32, u32),
    rays_uid: Uid,
    hits_uid: Uid,
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
        Self::create_with_uids(context, render_context, RAYS_UID, HITS_UID)
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("pathtracing_trace_pass::init");

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
                &mut *self.bvh.write().unwrap(),
                Some("BVH"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.indices.write().unwrap(),
                Some("Indices"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
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
                    binding_index: 3,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read | BindingFlags::Vertex,
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
                &mut *self.meshlets.write().unwrap(),
                Some("Meshlets"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 5,
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
                    flags: BindingFlags::Read | BindingFlags::Storage,
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
                    flags: BindingFlags::ReadWrite | BindingFlags::Storage,
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

        let x_pixels_managed_in_shader = 8;
        let y_pixels_managed_in_shader = 8;
        let x = self.dimensions.0.div_ceil(x_pixels_managed_in_shader);
        let y = self.dimensions.1.div_ceil(y_pixels_managed_in_shader);

        let rays_len = self.rays.read().unwrap().len();
        let hits_len = self.hits.read().unwrap().len();
        let required_len = (self.dimensions.0 * self.dimensions.1) as usize * SIZE_OF_DATA_BUFFER_ELEMENT;

        if rays_len < required_len {
            self.rays.write().unwrap().resize(required_len, RayPackedData(0.));
            self.rays.write().unwrap().mark_as_dirty(render_context);
        }
        if hits_len < required_len {
            self.hits.write().unwrap().resize(required_len, HitPackedData(0.));
            self.hits.write().unwrap().mark_as_dirty(render_context);
        }

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
    pub fn create_with_uids(
        context: &ContextRc,
        render_context: &RenderContextRc,
        rays_uid: Uid,
        hits_uid: Uid,
    ) -> Self {
        let data = ComputePassData {
            name: COMPUTE_PATHTRACING_TRACE_NAME.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_PATHTRACING_TRACE_PIPELINE)],
        };

        Self {
            compute_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &data,
                None,
            ),
            constant_data: render_context.global_buffers().constant_data.clone(),
            bvh: render_context.global_buffers().buffer::<GPUBVHNode>(),
            indices: render_context.global_buffers().buffer::<GPUVertexIndices>(),
            vertices_positions: render_context
                .global_buffers()
                .buffer::<GPUVertexPosition>(),
            instances: render_context
                .global_buffers()
                .vector_with_id::<GPUInstance>(INSTANCE_DATA_ID),
            meshlets: render_context.global_buffers().buffer::<GPUMeshlet>(),
            rays: render_context
                .global_buffers()
                .vector_with_id::<RayPackedData>(rays_uid),
            hits: render_context
                .global_buffers()
                .vector_with_id::<HitPackedData>(hits_uid),
            binding_data: BindingData::new(render_context, COMPUTE_PATHTRACING_TRACE_NAME),
            dimensions: (0, 0),
            rays_uid,
            hits_uid,
        }
    }
    pub fn set_dimensions(&mut self, dimensions: (u32, u32)) -> &mut Self {
        self.dimensions = dimensions;
        self
    }
}
