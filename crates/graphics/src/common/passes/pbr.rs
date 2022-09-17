use std::path::PathBuf;

use crate::{
    BindingData, BindingInfo, CommandBuffer, DrawCommandType, MeshFlags, Pass, RenderContext,
    RenderPass, RenderPassData, RenderTarget, ShaderStage, StoreOperation, TextureId,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::{generate_random_uid, INVALID_UID};

pub const PBR_PIPELINE: &str = "pipelines/PBR.render_pipeline";
pub const PBR_PASS_NAME: &str = "PBRPass";

pub struct PBRPass {
    render_pass: Resource<RenderPass>,
    binding_data: BindingData,
    gbuffer_textures: Vec<TextureId>,
    depth_texture: TextureId,
}
unsafe impl Send for PBRPass {}
unsafe impl Sync for PBRPass {}

impl Pass for PBRPass {
    fn name(&self) -> &str {
        PBR_PASS_NAME
    }
    fn static_name() -> &'static str {
        PBR_PASS_NAME
    }
    fn is_active(&self, render_context: &RenderContext) -> bool {
        render_context.has_commands(&self.draw_command_type(), &self.mesh_flags())
    }
    fn mesh_flags(&self) -> MeshFlags {
        MeshFlags::Visible | MeshFlags::Opaque
    }
    fn draw_command_type(&self) -> DrawCommandType {
        DrawCommandType::PerMeshlet
    }
    fn create(context: &ContextRc) -> Self
    where
        Self: Sized,
    {
        inox_profiler::scoped_profile!("pbr_pass::create");

        let data = RenderPassData {
            name: PBR_PASS_NAME.to_string(),
            store_color: StoreOperation::Store,
            store_depth: StoreOperation::Store,
            render_target: RenderTarget::Screen,
            pipeline: PathBuf::from(PBR_PIPELINE),
            ..Default::default()
        };

        Self {
            render_pass: RenderPass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                data,
                None,
            ),
            binding_data: BindingData::default(),
            gbuffer_textures: Vec::new(),
            depth_texture: INVALID_UID,
        }
    }
    fn init(&mut self, render_context: &mut RenderContext) {
        inox_profiler::scoped_profile!("pbr_pass::init");

        if self.gbuffer_textures.iter().any(|t| t.is_nil())
            || self.gbuffer_textures.is_empty()
            || self.depth_texture.is_nil()
            || render_context.render_buffers.textures.is_empty()
            || render_context.render_buffers.meshes.is_empty()
            || render_context.render_buffers.meshlets.is_empty()
            || render_context.render_buffers.lights.is_empty()
        {
            return;
        }

        let mut pass = self.render_pass.get_mut();

        self.binding_data
            .add_uniform_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.constant_data,
                BindingInfo {
                    group_index: 0,
                    binding_index: 0,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.meshes,
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.meshlets,
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.materials,
                BindingInfo {
                    group_index: 0,
                    binding_index: 3,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.textures,
                BindingInfo {
                    group_index: 0,
                    binding_index: 4,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.lights,
                BindingInfo {
                    group_index: 0,
                    binding_index: 5,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            );

        self.gbuffer_textures
            .iter()
            .enumerate()
            .for_each(|(i, id)| {
                self.binding_data.add_texture(
                    &render_context.texture_handler,
                    id,
                    BindingInfo {
                        group_index: 1,
                        binding_index: i,
                        stage: ShaderStage::Fragment,
                        ..Default::default()
                    },
                );
            });
        self.binding_data.add_texture(
            &render_context.texture_handler,
            &self.depth_texture,
            BindingInfo {
                group_index: 1,
                binding_index: self.gbuffer_textures.len(),
                stage: ShaderStage::Fragment,
                ..Default::default()
            },
        );

        self.binding_data
            .add_default_sampler(BindingInfo {
                group_index: 2,
                binding_index: 0,
                stage: ShaderStage::Fragment,
                ..Default::default()
            })
            .add_material_textures(
                &render_context.texture_handler,
                BindingInfo {
                    group_index: 2,
                    binding_index: 1,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            );
        self.binding_data.send_to_gpu(render_context, PBR_PASS_NAME);

        pass.init(render_context, &self.binding_data, None, None);
    }
    fn update(&self, render_context: &mut RenderContext, command_buffer: &mut CommandBuffer) {
        inox_profiler::scoped_profile!("pbr_pass::update");

        if self.gbuffer_textures.iter().any(|t| t.is_nil())
            || self.gbuffer_textures.is_empty()
            || self.depth_texture.is_nil()
            || render_context.render_buffers.textures.is_empty()
            || render_context.render_buffers.meshes.is_empty()
            || render_context.render_buffers.meshlets.is_empty()
        {
            return;
        }

        let pass = self.render_pass.get();
        let buffers = render_context.buffers();
        let pipeline = pass.pipeline().get();
        if !pipeline.is_initialized() {
            return;
        }

        let render_pass = pass.begin(
            render_context,
            &self.binding_data,
            &buffers,
            &pipeline,
            command_buffer,
        );
        pass.draw(render_pass, 0..3, 0..1);
    }
}

impl PBRPass {
    pub fn render_pass(&self) -> &Resource<RenderPass> {
        &self.render_pass
    }
    pub fn set_gbuffers_textures(&mut self, textures: &[&TextureId]) -> &mut Self {
        self.gbuffer_textures = textures.iter().map(|&id| *id).collect();
        self
    }
    pub fn set_depth_texture(&mut self, texture_id: &TextureId) -> &mut Self {
        self.depth_texture = *texture_id;
        self
    }
}
