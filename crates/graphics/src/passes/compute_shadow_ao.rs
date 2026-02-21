use std::path::PathBuf;

use inox_bvh::GPUBVHNode;
use inox_render::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData,
    ConstantDataRw, GPUBuffer, GPUInstance, GPULight, GPUMesh, GPUMeshlet, GPUTransform, GPUVector,
    GPUVertexIndices, GPUVertexPosition, Pass, RenderContext, RenderContextRc, ShaderStage,
    TextureView, INSTANCE_DATA_ID,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;

use crate::{RadiancePackedData, RayPackedData, ThroughputPackedData};

pub const COMPUTE_SHADOW_AO_PIPELINE: &str = "pipelines/ComputeShadowAO.compute_pipeline";
pub const COMPUTE_SHADOW_AO_NAME: &str = "ComputeShadowAOPass";

pub struct ComputeShadowAOPass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    indices: GPUBuffer<GPUVertexIndices>,
    vertices_positions: GPUBuffer<GPUVertexPosition>,
    instances: GPUVector<GPUInstance>,
    transforms: GPUVector<GPUTransform>,
    meshes: GPUBuffer<GPUMesh>,
    meshlets: GPUBuffer<GPUMeshlet>,
    bvh: GPUBuffer<GPUBVHNode>,
    lights: GPUBuffer<GPULight>,
    data_buffer_0: GPUVector<RayPackedData>,
    data_buffer_1: GPUVector<RadiancePackedData>,
    data_buffer_2: GPUVector<ThroughputPackedData>,
    dimensions: (u32, u32),
}
unsafe impl Send for ComputeShadowAOPass {}
unsafe impl Sync for ComputeShadowAOPass {}

impl Pass for ComputeShadowAOPass {
    fn name(&self) -> &str {
        COMPUTE_SHADOW_AO_NAME
    }
    fn static_name() -> &'static str {
        COMPUTE_SHADOW_AO_NAME
    }
    fn is_active(&self, _render_context: &RenderContext) -> bool {
        true
    }
    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self {
        let data = ComputePassData {
            name: COMPUTE_SHADOW_AO_NAME.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_SHADOW_AO_PIPELINE)],
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
            indices: render_context.global_buffers().buffer::<GPUVertexIndices>(),
            vertices_positions: render_context
                .global_buffers()
                .buffer::<GPUVertexPosition>(),
            instances: render_context
                .global_buffers()
                .vector_with_id::<GPUInstance>(INSTANCE_DATA_ID),
            transforms: render_context.global_buffers().vector::<GPUTransform>(),
            meshes: render_context.global_buffers().buffer::<GPUMesh>(),
            meshlets: render_context.global_buffers().buffer::<GPUMeshlet>(),
            bvh: render_context.global_buffers().buffer::<GPUBVHNode>(),
            lights: render_context.global_buffers().buffer::<GPULight>(),
            data_buffer_0: render_context.global_buffers().vector::<RayPackedData>(),
            data_buffer_1: render_context
                .global_buffers()
                .vector::<RadiancePackedData>(),
            data_buffer_2: render_context
                .global_buffers()
                .vector::<ThroughputPackedData>(),
            binding_data: BindingData::new(render_context, COMPUTE_SHADOW_AO_NAME),
            dimensions: (0, 0),
        }
    }

    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("shadow_ao_pass::init");

        if self.meshlets.read().unwrap().is_empty() {
            return;
        }

        // Group 0: geometry + BVH
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
                &mut *self.instances.write().unwrap(),
                Some("Instances"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 3,
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
                    binding_index: 4,
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
                    binding_index: 5,
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
                    binding_index: 6,
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
                    binding_index: 7,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            );

        // Group 1: lights + data buffers
        self.binding_data
            .add_buffer(
                &mut *self.lights.write().unwrap(),
                Some("Lights"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Uniform | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.data_buffer_0.write().unwrap(),
                Some("DataBuffer_0"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::ReadWrite | BindingFlags::Storage,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.data_buffer_1.write().unwrap(),
                Some("DataBuffer_1"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::ReadWrite | BindingFlags::Storage,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.data_buffer_2.write().unwrap(),
                Some("DataBuffer_2"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 3,
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
        if self.meshlets.read().unwrap().is_empty() {
            return;
        }

        inox_profiler::scoped_profile!("shadow_ao_pass::update");

        let pass = self.compute_pass.get();
        let x = self.dimensions.0.div_ceil(8);
        let y = self.dimensions.1.div_ceil(8);

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

impl ComputeShadowAOPass {
    pub fn set_dimensions(&mut self, dimensions: (u32, u32)) -> &mut Self {
        self.dimensions = dimensions;
        self
    }
}
