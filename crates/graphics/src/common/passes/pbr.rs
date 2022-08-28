use std::path::PathBuf;

use crate::{
    AsBinding, BindingData, BindingInfo, CommandBuffer, DrawCommandType, GpuBuffer, MeshFlags,
    Pass, RenderContext, RenderCoreContext, RenderPass, RenderPassData, RenderTarget, ShaderStage,
    StoreOperation, TextureId,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::{generate_random_uid, INVALID_UID};

pub const PBR_PIPELINE: &str = "pipelines/PBR.render_pipeline";
pub const PBR_PASS_NAME: &str = "PBRPass";

#[derive(Default, Debug, Clone, Copy)]
struct Data {
    pub gbuffer1_texture_index: u32,
    pub gbuffer2_texture_index: u32,
    pub gbuffer3_texture_index: u32,
    pub gbuffer4_texture_index: u32,
}

#[derive(Default, Clone, Copy)]
pub struct PBRPassData {
    is_dirty: bool,
    data: Data,
}

impl AsBinding for PBRPassData {
    fn is_dirty(&self) -> bool {
        self.is_dirty
    }
    fn set_dirty(&mut self, is_dirty: bool) {
        self.is_dirty = is_dirty;
    }
    fn size(&self) -> u64 {
        std::mem::size_of_val(&self.data) as u64
    }
    fn fill_buffer(&self, render_core_context: &RenderCoreContext, buffer: &mut GpuBuffer) {
        buffer.add_to_gpu_buffer(render_core_context, &[self.data]);
    }
}

pub struct PBRPass {
    render_pass: Resource<RenderPass>,
    binding_data: BindingData,
    data: PBRPassData,
    textures: Vec<TextureId>,
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
    fn is_active(&self, render_context: &mut RenderContext) -> bool {
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
            data: PBRPassData::default(),
            textures: Vec::new(),
            depth_texture: INVALID_UID,
        }
    }
    fn init(&mut self, render_context: &mut RenderContext) {
        inox_profiler::scoped_profile!("pbr_pass::init");

        if self.textures.iter().any(|t| t.is_nil())
            || self.textures.is_empty()
            || self.depth_texture.is_nil()
            || render_context.render_buffers.textures.is_empty()
            || render_context.render_buffers.meshes.is_empty()
            || render_context.render_buffers.meshlets.is_empty()
            || render_context.render_buffers.lights.is_empty()
        {
            return;
        }

        self.fill_data_from_texture_ids(render_context);

        let mut pass = self.render_pass.get_mut();
        let render_textures = pass.render_textures_id();

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
            .add_uniform_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut self.data,
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
                &mut render_context.render_buffers.meshes,
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
                &mut render_context.render_buffers.meshlets,
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
                &mut render_context.render_buffers.materials,
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
                &mut render_context.render_buffers.textures,
                BindingInfo {
                    group_index: 0,
                    binding_index: 5,
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
                    binding_index: 6,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_depth_texture(
                &render_context.texture_handler,
                &self.depth_texture,
                BindingInfo {
                    group_index: 1,
                    binding_index: 0,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_sampler_and_textures(
                &render_context.texture_handler,
                render_textures,
                None,
                BindingInfo {
                    group_index: 2,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            );
        self.binding_data.send_to_gpu(render_context, PBR_PASS_NAME);

        pass.init(render_context, &self.binding_data, None, None);
    }
    fn update(&self, render_context: &mut RenderContext, command_buffer: &mut CommandBuffer) {
        inox_profiler::scoped_profile!("pbr_pass::update");

        if self.textures.iter().any(|t| t.is_nil())
            || self.textures.is_empty()
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
        self.textures = textures.iter().map(|&id| *id).collect();
        self
    }
    pub fn set_depth_texture(&mut self, texture_id: &TextureId) -> &mut Self {
        self.depth_texture = *texture_id;
        self
    }

    fn fill_data_from_texture_ids(&mut self, render_context: &RenderContext) -> &mut Self {
        let gbuffer1_texture_index = render_context
            .render_buffers
            .textures
            .index_of(self.textures.get(0).unwrap_or(&INVALID_UID))
            .unwrap_or_default();
        let gbuffer2_texture_index = render_context
            .render_buffers
            .textures
            .index_of(self.textures.get(1).unwrap_or(&INVALID_UID))
            .unwrap_or_default();
        let gbuffer3_texture_index = render_context
            .render_buffers
            .textures
            .index_of(self.textures.get(2).unwrap_or(&INVALID_UID))
            .unwrap_or_default();
        let gbuffer4_buffer_index = render_context
            .render_buffers
            .textures
            .index_of(self.textures.get(3).unwrap_or(&INVALID_UID))
            .unwrap_or_default();

        if self.data.data.gbuffer1_texture_index != gbuffer1_texture_index as u32
            || self.data.data.gbuffer2_texture_index != gbuffer2_texture_index as u32
            || self.data.data.gbuffer3_texture_index != gbuffer3_texture_index as u32
            || self.data.data.gbuffer4_texture_index != gbuffer4_buffer_index as u32
        {
            self.data.data.gbuffer1_texture_index = gbuffer1_texture_index as u32;
            self.data.data.gbuffer2_texture_index = gbuffer2_texture_index as u32;
            self.data.data.gbuffer3_texture_index = gbuffer3_texture_index as u32;
            self.data.data.gbuffer4_texture_index = gbuffer4_buffer_index as u32;
            self.data.set_dirty(true);
        }

        self
    }
}
