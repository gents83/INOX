use std::path::PathBuf;

use inox_bvh::GPUBVHNode;
use inox_render::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData, ConstantDataRw, DEFAULT_HEIGHT, DEFAULT_WIDTH, GPUBuffer, GPUInstance, GPUMeshlet, GPUVector, GPUVertexIndices, GPUVertexPosition, INSTANCE_DATA_ID, Pass, RenderContext, RenderContextRc, SamplerType, ShaderStage, TextureId, TextureView
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::{generate_random_uid, INVALID_UID};

use crate::pathtracing_common::*;

pub const COMPUTE_PATHTRACING_SHADOW_PIPELINE: &str =
    "pipelines/ComputePathTracingShadow.compute_pipeline";
pub const COMPUTE_PATHTRACING_SHADOW_NAME: &str = "ComputePathTracingShadowPass";

pub struct ComputePathTracingShadowPass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    bvh: GPUBuffer<GPUBVHNode>,
    indices: GPUBuffer<GPUVertexIndices>,
    vertices_positions: GPUBuffer<GPUVertexPosition>,
    instances: GPUVector<GPUInstance>,
    meshlets: GPUBuffer<GPUMeshlet>,
    shadow_data: GPUVector<ShadowDataPackedData>,
    radiance: GPUVector<RadiancePackedData>,
    dimensions: (u32, u32),
}
unsafe impl Send for ComputePathTracingShadowPass {}
unsafe impl Sync for ComputePathTracingShadowPass {}

impl Pass for ComputePathTracingShadowPass {
    fn name(&self) -> &str {
        COMPUTE_PATHTRACING_SHADOW_NAME
    }
    fn static_name() -> &'static str {
        COMPUTE_PATHTRACING_SHADOW_NAME
    }
    fn is_active(&self, _render_context: &RenderContext) -> bool {
        true
    }
    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self
    where
        Self: Sized,
    {
        let data = ComputePassData {
            name: COMPUTE_PATHTRACING_SHADOW_NAME.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_PATHTRACING_SHADOW_PIPELINE)],
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
            shadow_data: render_context.global_buffers().vector_with_id::<ShadowDataPackedData>(SHADOW_DATA_UID),
            radiance: render_context.global_buffers().vector_with_id::<RadiancePackedData>(RADIANCE_UID),
            binding_data: BindingData::new(render_context, COMPUTE_PATHTRACING_SHADOW_NAME),
            dimensions: (0, 0),
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("pathtracing_shadow_pass::init");

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
                &mut *self.shadow_data.write().unwrap(),
                Some("ShadowData"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.radiance.write().unwrap(),
                Some("Radiance"),
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
        if self.meshlets.read().unwrap().is_empty()
            || self.instances.read().unwrap().is_empty()
        {
            return;
        }

        inox_profiler::scoped_profile!("pathtracing_shadow_pass::update");

        let pass = self.compute_pass.get();

        let x_pixels_managed_in_shader = 8;
        let y_pixels_managed_in_shader = 8;
        let x = self.dimensions.0.div_ceil(x_pixels_managed_in_shader);
        let y = self.dimensions.1.div_ceil(y_pixels_managed_in_shader);

        let shadow_data_len = self.shadow_data.read().unwrap().len();
        let required_len = (self.dimensions.0 * self.dimensions.1) as usize * SIZE_OF_DATA_BUFFER_ELEMENT * 3; // Using same factor as initialized

        if shadow_data_len < required_len {
            self.shadow_data.write().unwrap().resize(required_len, ShadowDataPackedData(0.));
            self.shadow_data.write().unwrap().mark_as_dirty(render_context);
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

impl ComputePathTracingShadowPass {
    pub fn set_dimensions(&mut self, dimensions: (u32, u32)) -> &mut Self {
        self.dimensions = dimensions;
        self
    }
}
