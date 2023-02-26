use std::path::PathBuf;

use crate::{
    AsBinding, BHVBuffer, BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass,
    ComputePassData, ConstantDataRw, DrawCommandType, GpuBuffer, IndicesBuffer, LightsBuffer,
    MaterialsBuffer, MeshFlags, MeshesBuffer, MeshletsBuffer, OutputPass, Pass, RenderContext,
    RenderCoreContext, ShaderStage, Texture, TextureFormat, TextureId, TextureUsage, TextureView,
    TexturesBuffer, VertexColorsBuffer, VertexNormalsBuffer, VertexPositionsBuffer,
    VertexUVsBuffer, VerticesBuffer, DEFAULT_HEIGHT, DEFAULT_WIDTH,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Handle, Resource};
use inox_uid::{generate_random_uid, INVALID_UID};

pub const COMPUTE_PBR_PIPELINE: &str = "pipelines/ComputePbr.compute_pipeline";
pub const COMPUTE_PBR_PASS_NAME: &str = "ComputePbrPass";
const COMPUTE_PBR_TEXTURE_FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;

#[derive(Default)]
struct ComputePbrPassData {
    dimensions: [u32; 2],
    visibility_buffer_texture_index: u32,
    is_dirty: u32,
}

impl AsBinding for ComputePbrPassData {
    fn is_dirty(&self) -> bool {
        self.is_dirty != 0u32
    }
    fn set_dirty(&mut self, is_dirty: bool) {
        self.is_dirty = is_dirty as _;
    }
    fn size(&self) -> u64 {
        std::mem::size_of_val(&self.dimensions) as u64
            + std::mem::size_of_val(&self.visibility_buffer_texture_index) as u64
            + std::mem::size_of_val(&self.is_dirty) as u64
    }

    fn fill_buffer(&self, render_core_context: &RenderCoreContext, buffer: &mut GpuBuffer) {
        buffer.add_to_gpu_buffer(render_core_context, &[self.dimensions]);
        buffer.add_to_gpu_buffer(render_core_context, &[self.visibility_buffer_texture_index]);
        buffer.add_to_gpu_buffer(render_core_context, &[self.is_dirty]);
    }
}
pub struct ComputePbrPass {
    context: ContextRc,
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    textures: TexturesBuffer,
    lights: LightsBuffer,
    materials: MaterialsBuffer,
    meshes: MeshesBuffer,
    bhv: BHVBuffer,
    meshlets: MeshletsBuffer,
    vertices: VerticesBuffer,
    indices: IndicesBuffer,
    vertex_positions: VertexPositionsBuffer,
    vertex_colors: VertexColorsBuffer,
    vertex_normals: VertexNormalsBuffer,
    vertex_uvs: VertexUVsBuffer,
    data: ComputePbrPassData,
    visibility_buffer_id: TextureId,
    render_target: Handle<Texture>,
}
unsafe impl Send for ComputePbrPass {}
unsafe impl Sync for ComputePbrPass {}

impl Pass for ComputePbrPass {
    fn name(&self) -> &str {
        COMPUTE_PBR_PASS_NAME
    }
    fn static_name() -> &'static str {
        COMPUTE_PBR_PASS_NAME
    }
    fn is_active(&self, _render_context: &RenderContext) -> bool {
        true
    }
    fn mesh_flags(&self) -> MeshFlags {
        MeshFlags::None
    }
    fn draw_commands_type(&self) -> DrawCommandType {
        DrawCommandType::PerMeshlet
    }
    fn create(context: &ContextRc, render_context: &RenderContext) -> Self
    where
        Self: Sized,
    {
        let data = ComputePassData {
            name: COMPUTE_PBR_PASS_NAME.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_PBR_PIPELINE)],
        };
        Self {
            context: context.clone(),
            compute_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &data,
                None,
            ),
            constant_data: render_context.constant_data.clone(),
            textures: render_context.render_buffers.textures.clone(),
            lights: render_context.render_buffers.lights.clone(),
            materials: render_context.render_buffers.materials.clone(),
            meshes: render_context.render_buffers.meshes.clone(),
            bhv: render_context.render_buffers.bhv.clone(),
            meshlets: render_context.render_buffers.meshlets.clone(),
            vertices: render_context.render_buffers.vertices.clone(),
            indices: render_context.render_buffers.indices.clone(),
            vertex_positions: render_context.render_buffers.vertex_positions.clone(),
            vertex_colors: render_context.render_buffers.vertex_colors.clone(),
            vertex_normals: render_context.render_buffers.vertex_normals.clone(),
            vertex_uvs: render_context.render_buffers.vertex_uvs.clone(),
            binding_data: BindingData::new(render_context, COMPUTE_PBR_PASS_NAME),
            visibility_buffer_id: INVALID_UID,
            data: ComputePbrPassData {
                dimensions: [DEFAULT_WIDTH, DEFAULT_HEIGHT],
                ..Default::default()
            },
            render_target: None,
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("compute_pbr_pass::init");

        if self.render_target.is_none() {
            return;
        }

        let resolution = self.render_target.as_ref().unwrap().get().dimensions();
        if self.render_target.is_none()
            && (self.data.dimensions[0] != resolution.0 || self.data.dimensions[1] != resolution.1)
        {
            self.data.dimensions = [resolution.0, resolution.1];
            self.data.set_dirty(true);
        }

        if self.visibility_buffer_id.is_nil()
            || self.textures.read().unwrap().is_empty()
            || self.meshes.read().unwrap().is_empty()
            || self.meshlets.read().unwrap().is_empty()
            || self.lights.read().unwrap().is_empty()
        {
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
            .add_uniform_buffer(
                &mut self.data,
                Some("ComputeData"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.indices.write().unwrap(),
                Some("Indices"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Read | BindingFlags::Index,
                },
            )
            .add_storage_buffer(
                &mut *self.vertices.write().unwrap(),
                Some("Vertices"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 3,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Read | BindingFlags::Vertex,
                },
            )
            .add_storage_buffer(
                &mut *self.vertex_positions.write().unwrap(),
                Some("VertexPositions"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 4,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.vertex_colors.write().unwrap(),
                Some("VertexColors"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 5,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.vertex_normals.write().unwrap(),
                Some("VertexNormals"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 6,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.vertex_uvs.write().unwrap(),
                Some("VertexUVs"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 7,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.meshes.write().unwrap(),
                Some("Meshes"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.meshlets.write().unwrap(),
                Some("Meshlets"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.materials.write().unwrap(),
                Some("Materials"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.textures.write().unwrap(),
                Some("Textures"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 3,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Read | BindingFlags::Storage,
                },
            )
            .add_storage_buffer(
                &mut *self.lights.write().unwrap(),
                Some("Lights"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 4,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.bhv.write().unwrap(),
                Some("BHV"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 5,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_default_sampler(BindingInfo {
                group_index: 2,
                binding_index: 0,
                stage: ShaderStage::Compute,
                ..Default::default()
            })
            .add_material_textures(BindingInfo {
                group_index: 2,
                binding_index: 1,
                stage: ShaderStage::Compute,
                ..Default::default()
            })
            .add_texture(
                &self.visibility_buffer_id,
                BindingInfo {
                    group_index: 3,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_texture(
                self.render_target.as_ref().unwrap().id(),
                BindingInfo {
                    group_index: 3,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Write | BindingFlags::Storage,
                },
            );

        let mut pass = self.compute_pass.get_mut();
        pass.init(render_context, &mut self.binding_data);
    }

    fn update(
        &mut self,
        render_context: &RenderContext,
        _surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
        inox_profiler::scoped_profile!("compute_pbr_pass::update");

        if self.visibility_buffer_id.is_nil()
            || self.textures.read().unwrap().is_empty()
            || self.meshes.read().unwrap().is_empty()
            || self.meshlets.read().unwrap().is_empty()
        {
            return;
        }

        let pass = self.compute_pass.get();

        let x_pixels_managed_in_shader = 16;
        let y_pixels_managed_in_shader = 16;
        let max_cluster_size = x_pixels_managed_in_shader.max(y_pixels_managed_in_shader);
        let x = (max_cluster_size
            * ((self.data.dimensions[0] + max_cluster_size - 1) / max_cluster_size))
            / x_pixels_managed_in_shader;
        let y = (max_cluster_size
            * ((self.data.dimensions[1] + max_cluster_size - 1) / max_cluster_size))
            / y_pixels_managed_in_shader;

        let mut compute_pass = pass.begin(render_context, &mut self.binding_data, command_buffer);
        {
            inox_profiler::gpu_scoped_profile!(
                &mut compute_pass,
                &render_context.core.device,
                "compute_pbr_pass",
            );
            pass.dispatch(render_context, compute_pass, x, y, 1);
        }
    }
}

impl OutputPass for ComputePbrPass {
    fn render_targets_id(&self) -> Vec<TextureId> {
        [*self.render_target.as_ref().unwrap().id()].to_vec()
    }
}

impl ComputePbrPass {
    pub fn add_texture(&mut self, texture_id: &TextureId) -> &mut Self {
        self.visibility_buffer_id = *texture_id;
        self
    }
    pub fn add_render_target_with_resolution(&mut self, width: u32, height: u32) -> &mut Self {
        self.render_target = Some(Texture::create_from_format(
            self.context.shared_data(),
            self.context.message_hub(),
            width,
            height,
            COMPUTE_PBR_TEXTURE_FORMAT,
            TextureUsage::TextureBinding
                | TextureUsage::CopySrc
                | TextureUsage::CopyDst
                | TextureUsage::RenderAttachment
                | TextureUsage::StorageBinding,
        ));
        self.data.dimensions = [width, height];
        self.data.set_dirty(true);
        self
    }
}
