use std::path::PathBuf;

use inox_bvh::GPUBVHNode;
use inox_render::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData,
    ConstantDataRw, GPUBuffer, GPULight, GPUMaterial, GPUMesh, GPUMeshlet, GPUTexture, GPUVector,
    GPUVertexAttributes, GPUVertexIndices, GPUVertexPosition, Pass, RenderContext, RenderContextRc,
    SamplerType, ShaderStage, TextureView, DEFAULT_HEIGHT, DEFAULT_WIDTH,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;

use crate::{RadiancePackedData, RayPackedData, ThroughputPackedData, SIZE_OF_DATA_BUFFER_ELEMENT};

pub const COMPUTE_PATHTRACING_INDIRECT_PIPELINE: &str =
    "pipelines/ComputePathTracingIndirect.compute_pipeline";
pub const COMPUTE_PATHTRACING_INDIRECT_NAME: &str = "ComputePathTracingIndirectPass";

#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct DebugPackedData(pub f32);

pub struct ComputePathTracingIndirectPass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    meshes: GPUBuffer<GPUMesh>,
    meshlets: GPUBuffer<GPUMeshlet>,
    bvh: GPUBuffer<GPUBVHNode>,
    indices: GPUBuffer<GPUVertexIndices>,
    vertices_position: GPUBuffer<GPUVertexPosition>,
    vertices_attributes: GPUBuffer<GPUVertexAttributes>,
    textures: GPUBuffer<GPUTexture>,
    materials: GPUBuffer<GPUMaterial>,
    lights: GPUBuffer<GPULight>,
    data_buffer_0: GPUVector<RayPackedData>,
    data_buffer_1: GPUVector<RadiancePackedData>,
    data_buffer_2: GPUVector<ThroughputPackedData>,
    data_buffer_debug: GPUVector<DebugPackedData>,
    dimensions: (u32, u32),
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

        let data_buffer_debug = render_context.global_buffers().vector::<DebugPackedData>();
        data_buffer_debug.write().unwrap().resize(
            SIZE_OF_DATA_BUFFER_ELEMENT * (DEFAULT_WIDTH * DEFAULT_HEIGHT) as usize,
            DebugPackedData(0.),
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
            meshes: render_context.global_buffers().buffer::<GPUMesh>(),
            meshlets: render_context.global_buffers().buffer::<GPUMeshlet>(),
            bvh: render_context.global_buffers().buffer::<GPUBVHNode>(),
            indices: render_context.global_buffers().buffer::<GPUVertexIndices>(),
            vertices_position: render_context
                .global_buffers()
                .buffer::<GPUVertexPosition>(),
            vertices_attributes: render_context
                .global_buffers()
                .buffer::<GPUVertexAttributes>(),
            textures: render_context.global_buffers().buffer::<GPUTexture>(),
            materials: render_context.global_buffers().buffer::<GPUMaterial>(),
            lights: render_context.global_buffers().buffer::<GPULight>(),
            data_buffer_0: render_context.global_buffers().vector::<RayPackedData>(),
            data_buffer_1: render_context
                .global_buffers()
                .vector::<RadiancePackedData>(),
            data_buffer_2: render_context
                .global_buffers()
                .vector::<ThroughputPackedData>(),
            data_buffer_debug,
            binding_data: BindingData::new(render_context, COMPUTE_PATHTRACING_INDIRECT_NAME),
            dimensions: (0, 0),
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("pathtracing_indirect_pass::init");

        if self.meshlets.read().unwrap().is_empty() {
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
                &mut *self.vertices_attributes.write().unwrap(),
                Some("Vertices Attributes"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
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
                    binding_index: 3,
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
                    binding_index: 4,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.materials.write().unwrap(),
                Some("Materials"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 5,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.textures.write().unwrap(),
                Some("Textures"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 6,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.lights.write().unwrap(),
                Some("Lights"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 7,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Uniform | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.vertices_position.write().unwrap(),
                Some("Vertices Position"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read | BindingFlags::Vertex,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.bvh.write().unwrap(),
                Some("BVH"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.data_buffer_0.write().unwrap(),
                Some("DataBuffer_0"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 2,
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
                    binding_index: 3,
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
                    binding_index: 4,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::ReadWrite | BindingFlags::Storage,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.data_buffer_debug.write().unwrap(),
                Some("DataBuffer_Debug"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 5,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::ReadWrite | BindingFlags::Storage,
                    ..Default::default()
                },
            );

        self.binding_data
            .add_default_sampler(
                BindingInfo {
                    group_index: 2,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
                SamplerType::Unfiltered,
            )
            .add_material_textures(BindingInfo {
                group_index: 2,
                binding_index: 1,
                stage: ShaderStage::Compute,
                ..Default::default()
            });

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

        inox_profiler::scoped_profile!("pathtracing_indirect_pass::update");

        let pass = self.compute_pass.get();

        let x_pixels_managed_in_shader = 8;
        let y_pixels_managed_in_shader = 8;
        let x = self.dimensions.0.div_ceil(x_pixels_managed_in_shader);
        let y = self.dimensions.1.div_ceil(y_pixels_managed_in_shader);

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

impl ComputePathTracingIndirectPass {
    pub fn set_dimensions(&mut self, dimensions: (u32, u32)) -> &mut Self {
        self.dimensions = dimensions;
        self
    }
}
