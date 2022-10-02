use std::path::PathBuf;

use crate::{
    AsBinding, BindingData, BindingInfo, CommandBuffer, ComputePass, ComputePassData,
    DrawCommandType, GpuBuffer, MeshFlags, Pass, RenderContext, RenderCoreContext, ShaderStage,
    Texture, TextureFormat, TextureId, TextureUsage, DEFAULT_HEIGHT, DEFAULT_WIDTH,
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
    fn draw_command_type(&self) -> DrawCommandType {
        DrawCommandType::PerMeshlet
    }
    fn create(context: &ContextRc) -> Self
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
            binding_data: BindingData::default(),
            visibility_buffer_id: INVALID_UID,
            data: ComputePbrPassData {
                dimensions: [DEFAULT_WIDTH, DEFAULT_HEIGHT],
                ..Default::default()
            },
            render_target: None,
        }
    }
    fn init(&mut self, render_context: &mut RenderContext) {
        inox_profiler::scoped_profile!("compute_pbr_pass::init");

        if self.render_target.is_none()
            && (self.data.dimensions[0] != render_context.core.config.width
                || self.data.dimensions[1] != render_context.core.config.height)
        {
            self.data.dimensions = [
                render_context.core.config.width,
                render_context.core.config.height,
            ];
            self.data.set_dirty(true);
        }

        if self.visibility_buffer_id.is_nil()
            || render_context.render_buffers.textures.is_empty()
            || render_context.render_buffers.meshes.is_empty()
            || render_context.render_buffers.meshlets.is_empty()
            || render_context.render_buffers.lights.is_empty()
        {
            return;
        }

        self.binding_data
            .add_uniform_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.constant_data,
                BindingInfo {
                    group_index: 0,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_uniform_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut self.data,
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.indices,
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    is_index: true,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.vertices,
                BindingInfo {
                    group_index: 0,
                    binding_index: 3,
                    stage: ShaderStage::Compute,
                    is_vertex: true,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.vertex_positions,
                BindingInfo {
                    group_index: 0,
                    binding_index: 4,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.vertex_colors,
                BindingInfo {
                    group_index: 0,
                    binding_index: 5,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.vertex_normals,
                BindingInfo {
                    group_index: 0,
                    binding_index: 6,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.vertex_uvs,
                BindingInfo {
                    group_index: 0,
                    binding_index: 7,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.meshes,
                BindingInfo {
                    group_index: 1,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.meshlets,
                BindingInfo {
                    group_index: 1,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.materials,
                BindingInfo {
                    group_index: 1,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.textures,
                BindingInfo {
                    group_index: 1,
                    binding_index: 3,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.lights,
                BindingInfo {
                    group_index: 1,
                    binding_index: 4,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.meshes_aabb,
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
            .add_material_textures(
                &render_context.texture_handler,
                BindingInfo {
                    group_index: 2,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_texture(
                &render_context.texture_handler,
                &self.visibility_buffer_id,
                BindingInfo {
                    group_index: 3,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_texture(
                &render_context.texture_handler,
                self.render_target.as_ref().unwrap().id(),
                BindingInfo {
                    group_index: 3,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    is_storage: true,
                    read_only: false,
                    ..Default::default()
                },
            );
        self.binding_data
            .send_to_gpu(render_context, COMPUTE_PBR_PASS_NAME);

        let mut pass = self.compute_pass.get_mut();
        pass.init(render_context, &self.binding_data);
    }

    fn update(&self, render_context: &mut RenderContext, command_buffer: &mut CommandBuffer) {
        inox_profiler::scoped_profile!("compute_pbr_pass::update");

        if self.visibility_buffer_id.is_nil()
            || render_context.render_buffers.textures.is_empty()
            || render_context.render_buffers.meshes.is_empty()
            || render_context.render_buffers.meshlets.is_empty()
        {
            return;
        }

        let pass = self.compute_pass.get();

        let max_cluster_size = 32;
        let x_pixels_managed_in_shader = 4 * 8;
        let y_pixels_managed_in_shader = 4 * 4;
        let x = max_cluster_size
            * ((self.data.dimensions[0] + max_cluster_size - 1) / max_cluster_size)
            / x_pixels_managed_in_shader;
        let y = max_cluster_size
            * ((self.data.dimensions[1] + max_cluster_size - 1) / max_cluster_size)
            / y_pixels_managed_in_shader;

        let mut compute_pass = pass.begin(render_context, &self.binding_data, command_buffer);
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
    pub fn render_target_id(&self) -> &TextureId {
        self.render_target.as_ref().unwrap().id()
    }
}
