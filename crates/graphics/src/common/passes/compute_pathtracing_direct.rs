use std::path::PathBuf;

use crate::{
    AtomicCounters, BVHBuffer, BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass,
    ComputePassData, ConstantDataRw, DrawCommandType, IndicesBuffer, LightsBuffer, MaterialsBuffer,
    MeshFlags, MeshesBuffer, MeshletsBuffer, Pass, RadianceDataBuffer, RenderContext,
    RuntimeVerticesBuffer, SamplerType, ShaderStage, TextureId, TextureView, TexturesBuffer,
    VertexAttributesBuffer,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::{generate_random_uid, INVALID_UID};

pub const COMPUTE_PATHTRACING_DIRECT_PIPELINE: &str =
    "pipelines/ComputePathtracingDirect.compute_pipeline";
pub const COMPUTE_PATHTRACING_DIRECT_NAME: &str = "ComputePathTracingDirectPass";

pub struct ComputePathTracingDirectPass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    meshes: MeshesBuffer,
    meshlets: MeshletsBuffer,
    radiance_data_buffer: RadianceDataBuffer,
    atomic_counters: AtomicCounters,
    bhv: BVHBuffer,
    indices: IndicesBuffer,
    runtime_vertices: RuntimeVerticesBuffer,
    vertices_attributes: VertexAttributesBuffer,
    textures: TexturesBuffer,
    materials: MaterialsBuffer,
    lights: LightsBuffer,
    visibility_texture: TextureId,
    depth_texture: TextureId,
    binding_texture: TextureId,
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
    fn create(context: &ContextRc, render_context: &RenderContext) -> Self
    where
        Self: Sized,
    {
        let data = ComputePassData {
            name: COMPUTE_PATHTRACING_DIRECT_NAME.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_PATHTRACING_DIRECT_PIPELINE)],
        };

        Self {
            compute_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &data,
                None,
            ),
            constant_data: render_context.constant_data.clone(),
            meshes: render_context.global_buffers.meshes.clone(),
            meshlets: render_context.global_buffers.meshlets.clone(),
            radiance_data_buffer: render_context.global_buffers.radiance_data_buffer.clone(),
            atomic_counters: render_context.global_buffers.atomic_counters.clone(),
            bhv: render_context.global_buffers.bvh.clone(),
            indices: render_context.global_buffers.indices.clone(),
            runtime_vertices: render_context.global_buffers.runtime_vertices.clone(),
            vertices_attributes: render_context.global_buffers.vertex_attributes.clone(),
            textures: render_context.global_buffers.textures.clone(),
            materials: render_context.global_buffers.materials.clone(),
            lights: render_context.global_buffers.lights.clone(),
            binding_data: BindingData::new(render_context, COMPUTE_PATHTRACING_DIRECT_NAME),
            visibility_texture: INVALID_UID,
            binding_texture: INVALID_UID,
            depth_texture: INVALID_UID,
            dimensions: (0, 0),
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("pathtracing_direct_pass::init");

        if self.visibility_texture.is_nil()
            || self.meshlets.read().unwrap().is_empty()
            || self.radiance_data_buffer.read().unwrap().data().is_empty()
        {
            return;
        }

        self.atomic_counters.write().unwrap().set(vec![0u32; 2]);

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
            .add_storage_buffer(
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
                &mut *self.runtime_vertices.write().unwrap(),
                Some("Runtime Vertices"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Read | BindingFlags::Vertex,
                },
            )
            .add_storage_buffer(
                &mut *self.radiance_data_buffer.write().unwrap(),
                Some("Radiance Data Buffer"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::ReadWrite | BindingFlags::Storage,
                },
            )
            .add_storage_buffer(
                &mut *self.atomic_counters.write().unwrap(),
                Some("Atomic Counters"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::ReadWrite | BindingFlags::Storage,
                },
            )
            .add_storage_buffer(
                &mut *self.bhv.write().unwrap(),
                Some("BHV"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 3,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_texture(
                &self.visibility_texture,
                BindingInfo {
                    group_index: 1,
                    binding_index: 4,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::ReadWrite | BindingFlags::Storage,
                },
            )
            .add_texture(
                &self.depth_texture,
                BindingInfo {
                    group_index: 1,
                    binding_index: 5,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_texture(
                &self.binding_texture,
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
                SamplerType::Unfiltered,
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
        if self.visibility_texture.is_nil()
            || self.meshlets.read().unwrap().is_empty()
            || self.radiance_data_buffer.read().unwrap().data().is_empty()
        {
            return;
        }

        inox_profiler::scoped_profile!("pathtracing_direct_pass::update");

        let pass = self.compute_pass.get();

        let x_pixels_managed_in_shader = 8;
        let y_pixels_managed_in_shader = 8;
        let x = (x_pixels_managed_in_shader
            * ((self.dimensions.0 + x_pixels_managed_in_shader - 1) / x_pixels_managed_in_shader))
            / x_pixels_managed_in_shader;
        let y = (y_pixels_managed_in_shader
            * ((self.dimensions.1 + y_pixels_managed_in_shader - 1) / y_pixels_managed_in_shader))
            / y_pixels_managed_in_shader;

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
    pub fn set_visibility_texture(&mut self, texture_id: &TextureId) -> &mut Self {
        self.visibility_texture = *texture_id;
        self
    }
    pub fn set_depth_texture(&mut self, texture_id: &TextureId) -> &mut Self {
        self.depth_texture = *texture_id;
        self
    }
    pub fn set_binding_texture(&mut self, texture_id: &TextureId) -> &mut Self {
        self.binding_texture = *texture_id;
        self
    }
    pub fn set_radiance_texture_size(&mut self, dimensions: (u32, u32)) -> &mut Self {
        self.dimensions = dimensions;
        //origin(3f), direction(3f), radiance(3f), throughput_weight(3f), seed(2u) = 14
        const RADIANCE_DATA_STRIDE: u32 = std::mem::size_of::<u32>() as u32 * 14;
        self.radiance_data_buffer.write().unwrap().set(vec![
            0u32;
            (dimensions.0 * dimensions.1 * RADIANCE_DATA_STRIDE)
                as usize
        ]);
        self
    }
}
