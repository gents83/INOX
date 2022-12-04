use std::path::PathBuf;

use crate::{
    BindingData, BindingInfo, CommandBuffer, ConstantDataRw, DrawCommandType, LightsBuffer,
    MaterialsBuffer, MeshFlags, MeshesBuffer, MeshletsBuffer, OutputRenderPass, Pass,
    RenderContext, RenderPass, RenderPassBeginData, RenderPassData, RenderTarget, ShaderStage,
    StoreOperation, TextureId, TextureView, TexturesBuffer,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource, ResourceTrait};
use inox_uid::{generate_random_uid, INVALID_UID};

pub const PBR_PIPELINE: &str = "pipelines/PBR.render_pipeline";
pub const PBR_PASS_NAME: &str = "PBRPass";

pub struct PBRPass {
    render_pass: Resource<RenderPass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    textures: TexturesBuffer,
    materials: MaterialsBuffer,
    lights: LightsBuffer,
    meshes: MeshesBuffer,
    meshlets: MeshletsBuffer,
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
                &data,
                None,
            ),
            constant_data: render_context.constant_data.clone(),
            textures: render_context.render_buffers.textures.clone(),
            materials: render_context.render_buffers.materials.clone(),
            lights: render_context.render_buffers.lights.clone(),
            meshes: render_context.render_buffers.meshes.clone(),
            meshlets: render_context.render_buffers.meshlets.clone(),
            binding_data: BindingData::new(render_context, PBR_PASS_NAME),
            gbuffer_textures: Vec::new(),
            depth_texture: INVALID_UID,
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("pbr_pass::init");

        if self.gbuffer_textures.iter().any(|t| t.is_nil())
            || self.gbuffer_textures.is_empty()
            || self.depth_texture.is_nil()
            || self.textures.read().unwrap().is_empty()
            || self.meshes.read().unwrap().is_empty()
            || self.meshlets.read().unwrap().is_empty()
            || self.lights.read().unwrap().is_empty()
        {
            return;
        }

        let mut pass = self.render_pass.get_mut();

        self.binding_data
            .add_uniform_buffer(
                &mut *self.constant_data.write().unwrap(),
                Some("ConstantData"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 0,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.meshes.write().unwrap(),
                Some("Meshes"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.meshlets.write().unwrap(),
                Some("Meshlets"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.materials.write().unwrap(),
                Some("Materials"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 3,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.textures.write().unwrap(),
                Some("Textures"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 4,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &mut *self.lights.write().unwrap(),
                Some("Lights"),
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
            .add_material_textures(BindingInfo {
                group_index: 2,
                binding_index: 1,
                stage: ShaderStage::Fragment,
                ..Default::default()
            });

        pass.init(render_context, &mut self.binding_data, None, None);
    }
    fn update(
        &mut self,
        render_context: &RenderContext,
        surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
        inox_profiler::scoped_profile!("pbr_pass::update");

        if self.gbuffer_textures.iter().any(|t| t.is_nil())
            || self.gbuffer_textures.is_empty()
            || self.depth_texture.is_nil()
            || self.textures.read().unwrap().is_empty()
            || self.meshes.read().unwrap().is_empty()
            || self.meshlets.read().unwrap().is_empty()
        {
            return;
        }

        let pass = self.render_pass.get();
        let pipeline = pass.pipeline().get();
        if !pipeline.is_initialized() {
            return;
        }
        let buffers = render_context.buffers();
        let render_targets = render_context.texture_handler.render_targets();

        let render_pass_begin_data = RenderPassBeginData {
            render_core_context: &render_context.core,
            buffers: &buffers,
            render_targets: render_targets.as_slice(),
            surface_view,
            command_buffer,
        };
        let mut render_pass = pass.begin(&mut self.binding_data, &pipeline, render_pass_begin_data);
        {
            inox_profiler::gpu_scoped_profile!(
                &mut render_pass,
                &render_context.core.device,
                "pbr_pass",
            );
            pass.draw(render_context, render_pass, 0..3, 0..1);
        }
    }
}

impl OutputRenderPass for PBRPass {
    fn render_pass(&self) -> &Resource<RenderPass> {
        &self.render_pass
    }
}

impl PBRPass {
    pub fn set_gbuffers_textures(&mut self, textures: &[&TextureId]) -> &mut Self {
        self.gbuffer_textures = textures.iter().map(|&id| *id).collect();
        self
    }
    pub fn set_depth_texture(&mut self, texture_id: &TextureId) -> &mut Self {
        self.depth_texture = *texture_id;
        self
    }
}
