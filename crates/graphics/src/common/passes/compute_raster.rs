use std::path::PathBuf;

use crate::{
    BindingData, BindingInfo, CommandBuffer, ComputePass, ComputePassData, DrawCommandType,
    MeshFlags, Pass, RenderContext, ShaderStage, Texture, TextureFormat, TextureUsage,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Handle, Resource};
use inox_uid::generate_random_uid;

pub const COMPUTE_RASTER_PIPELINE: &str = "pipelines/ComputeRaster.compute_pipeline";
pub const COMPUTE_RASTER_PASS_NAME: &str = "ComputeRasterPass";
const COMPUTE_PBR_TEXTURE_FORMAT: TextureFormat = TextureFormat::R32Sint;

pub struct ComputeRasterPass {
    context: ContextRc,
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    render_target: Handle<Texture>,
}
unsafe impl Send for ComputeRasterPass {}
unsafe impl Sync for ComputeRasterPass {}

impl Pass for ComputeRasterPass {
    fn name(&self) -> &str {
        COMPUTE_RASTER_PASS_NAME
    }
    fn static_name() -> &'static str {
        COMPUTE_RASTER_PASS_NAME
    }
    fn is_active(&self) -> bool {
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
            name: COMPUTE_RASTER_PASS_NAME.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_RASTER_PIPELINE)],
        };
        Self {
            context: context.clone(),
            compute_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                data,
                None,
            ),
            binding_data: BindingData::default(),
            render_target: None,
        }
    }
    fn init(&mut self, render_context: &mut RenderContext) {
        inox_profiler::scoped_profile!("pbr_pass::init");

        if render_context.render_buffers.meshlets.is_empty() {
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
            .add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.indices,
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
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
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    is_vertex: true,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.vertex_positions_and_colors,
                BindingInfo {
                    group_index: 0,
                    binding_index: 3,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.meshes,
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
                &mut render_context.render_buffers.meshlets,
                BindingInfo {
                    group_index: 0,
                    binding_index: 5,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_storage_textures(
                render_context,
                if self.render_target.is_some() {
                    vec![self.render_target.as_ref().unwrap()]
                } else {
                    Vec::new()
                },
                BindingInfo {
                    group_index: 1,
                    stage: ShaderStage::Compute,
                    read_only: false,
                    ..Default::default()
                },
            )
            .send_to_gpu(render_context, COMPUTE_RASTER_PASS_NAME);

        let mut pass = self.compute_pass.get_mut();
        pass.init(render_context, &self.binding_data);
    }

    fn update(&self, render_context: &mut RenderContext, command_buffer: &mut CommandBuffer) {
        if render_context.render_buffers.meshlets.is_empty() {
            return;
        }
        let pass = self.compute_pass.get();

        let compute_pass = pass.begin(&self.binding_data, command_buffer);
        let max_per_group = 16;
        let count = (render_context.render_buffers.meshlets.item_count() as u32 + max_per_group
            - 1)
            / max_per_group;
        pass.dispatch(compute_pass, count, 1, 1);
    }
}

impl ComputeRasterPass {
    pub fn resolution(&mut self, width: u32, height: u32) -> &mut Self {
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
        self
    }
}
