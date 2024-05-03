use std::path::PathBuf;

use inox_bvh::GPUBVHNode;
use inox_render::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData,
    ConstantDataRw, DrawCommandType, GPUBuffer, GPULight, GPUMaterial, GPUMesh, GPUMeshlet,
    GPUTexture, GPUVector, GPUVertexAttributes, GPUVertexIndices, GPUVertexPosition, MeshFlags,
    Pass, RenderContext, RenderContextRc, SamplerType, ShaderStage, TextureId, TextureView,
    DEFAULT_HEIGHT, DEFAULT_WIDTH,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::{generate_random_uid, INVALID_UID};

pub const COMPUTE_PATHTRACING_DIRECT_PIPELINE: &str =
    "pipelines/ComputePathtracingDirect.compute_pipeline";
pub const COMPUTE_PATHTRACING_DIRECT_NAME: &str = "ComputePathTracingDirectPass";

pub const SIZE_OF_DATA_BUFFER_ELEMENT: usize = 4;
#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct RayPackedData(pub f32);
#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct RadiancePackedData(pub f32);
#[repr(C)]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct ThroughputPackedData(pub f32);

pub struct ComputePathTracingDirectPass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    meshes: GPUBuffer<GPUMesh>,
    meshlets: GPUBuffer<GPUMeshlet>,
    bhv: GPUBuffer<GPUBVHNode>,
    indices: GPUBuffer<GPUVertexIndices>,
    vertices_positions: GPUBuffer<GPUVertexPosition>,
    vertices_attributes: GPUBuffer<GPUVertexAttributes>,
    textures: GPUBuffer<GPUTexture>,
    materials: GPUBuffer<GPUMaterial>,
    lights: GPUBuffer<GPULight>,
    visibility_texture: TextureId,
    depth_texture: TextureId,
    data_buffer_0: GPUVector<RayPackedData>,
    data_buffer_1: GPUVector<RadiancePackedData>,
    data_buffer_2: GPUVector<ThroughputPackedData>,
    dimensions: (u32, u32),
}
unsafe impl Send for ComputePathTracingDirectPass {}
unsafe impl Sync for ComputePathTracingDirectPass {}

impl Pass for ComputePathTracingDirectPass {
    fn name(&self) -> &str {
        COMPUTE_PATHTRACING_DIRECT_NAME
    }
    fn static_name() -> &'static str {
        COMPUTE_PATHTRACING_DIRECT_NAME
    }
    fn is_active(&self, render_context: &RenderContext) -> bool {
        render_context.has_commands(&self.draw_commands_type(), &self.mesh_flags())
    }
    fn mesh_flags(&self) -> MeshFlags {
        MeshFlags::Visible | MeshFlags::Opaque
    }
    fn draw_commands_type(&self) -> DrawCommandType {
        DrawCommandType::PerMeshlet
    }
    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self
    where
        Self: Sized,
    {
        let data = ComputePassData {
            name: COMPUTE_PATHTRACING_DIRECT_NAME.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_PATHTRACING_DIRECT_PIPELINE)],
        };

        let data_buffer_0 = render_context.global_buffers().vector::<RayPackedData>();
        let data_buffer_1 = render_context
            .global_buffers()
            .vector::<RadiancePackedData>();
        let data_buffer_2 = render_context
            .global_buffers()
            .vector::<ThroughputPackedData>();

        data_buffer_0.write().unwrap().resize(
            SIZE_OF_DATA_BUFFER_ELEMENT * (DEFAULT_WIDTH * DEFAULT_HEIGHT) as usize,
            RayPackedData(0.),
        );
        data_buffer_1.write().unwrap().resize(
            SIZE_OF_DATA_BUFFER_ELEMENT * (DEFAULT_WIDTH * DEFAULT_HEIGHT) as usize,
            RadiancePackedData(0.),
        );
        data_buffer_2.write().unwrap().resize(
            SIZE_OF_DATA_BUFFER_ELEMENT * (DEFAULT_WIDTH * DEFAULT_HEIGHT) as usize,
            ThroughputPackedData(0.),
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
            bhv: render_context.global_buffers().buffer::<GPUBVHNode>(),
            indices: render_context.global_buffers().buffer::<GPUVertexIndices>(),
            vertices_positions: render_context
                .global_buffers()
                .buffer::<GPUVertexPosition>(),
            vertices_attributes: render_context
                .global_buffers()
                .buffer::<GPUVertexAttributes>(),
            textures: render_context.global_buffers().buffer::<GPUTexture>(),
            materials: render_context.global_buffers().buffer::<GPUMaterial>(),
            lights: render_context.global_buffers().buffer::<GPULight>(),
            data_buffer_0,
            data_buffer_1,
            data_buffer_2,
            binding_data: BindingData::new(render_context, COMPUTE_PATHTRACING_DIRECT_NAME),
            visibility_texture: INVALID_UID,
            depth_texture: INVALID_UID,
            dimensions: (0, 0),
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("pathtracing_direct_pass::init");

        if self.visibility_texture.is_nil() || self.meshlets.read().unwrap().is_empty() {
            return;
        }

        self.binding_data
            .add_uniform_buffer(
                &mut *self.constant_data.write().unwrap(),
                Some("ConstantData"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.indices.write().unwrap(),
                Some("Indices"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Read | BindingFlags::Index,
                },
            )
            .add_storage_buffer(
                &mut *self.vertices_attributes.write().unwrap(),
                Some("Vertices Attributes"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.meshes.write().unwrap(),
                Some("Meshes"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 3,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.meshlets.write().unwrap(),
                Some("Meshlets"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 4,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.materials.write().unwrap(),
                Some("Materials"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 5,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.textures.write().unwrap(),
                Some("Textures"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 6,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_uniform_buffer(
                &mut *self.lights.write().unwrap(),
                Some("Lights"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 7,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.vertices_positions.write().unwrap(),
                Some("Vertices Positions"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Read | BindingFlags::Vertex,
                },
            )
            .add_storage_buffer(
                &mut *self.bhv.write().unwrap(),
                Some("BHV"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_texture(
                &self.visibility_texture,
                BindingInfo {
                    group_index: 1,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_texture(
                &self.depth_texture,
                BindingInfo {
                    group_index: 1,
                    binding_index: 3,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.data_buffer_0.write().unwrap(),
                Some("DataBuffer_0"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 4,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::ReadWrite | BindingFlags::Storage,
                },
            )
            .add_storage_buffer(
                &mut *self.data_buffer_1.write().unwrap(),
                Some("DataBuffer_1"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 5,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::ReadWrite | BindingFlags::Storage,
                },
            )
            .add_storage_buffer(
                &mut *self.data_buffer_2.write().unwrap(),
                Some("DataBuffer_2"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 6,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::ReadWrite | BindingFlags::Storage,
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
                SamplerType::Default,
            )
            .add_material_textures(BindingInfo {
                group_index: 2,
                binding_index: 1,
                stage: ShaderStage::Compute,
                ..Default::default()
            });

        let mut pass = self.compute_pass.get_mut();
        pass.init(render_context, &mut self.binding_data);
    }

    fn update(
        &mut self,
        render_context: &RenderContext,
        _surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
        if self.visibility_texture.is_nil() || self.meshlets.read().unwrap().is_empty() {
            return;
        }

        inox_profiler::scoped_profile!("pathtracing_direct_pass::update");

        let pass = self.compute_pass.get();

        let x_pixels_managed_in_shader = 8;
        let y_pixels_managed_in_shader = 8;
        let x = (self.dimensions.0 + x_pixels_managed_in_shader - 1) / x_pixels_managed_in_shader;
        let y = (self.dimensions.1 + y_pixels_managed_in_shader - 1) / y_pixels_managed_in_shader;

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

impl ComputePathTracingDirectPass {
    pub fn set_visibility_texture(
        &mut self,
        texture_id: &TextureId,
        dimensions: (u32, u32),
    ) -> &mut Self {
        self.dimensions = dimensions;
        self.visibility_texture = *texture_id;
        self
    }
    pub fn set_depth_texture(&mut self, texture_id: &TextureId) -> &mut Self {
        self.depth_texture = *texture_id;
        self
    }
}
