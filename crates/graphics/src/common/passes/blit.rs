use std::path::PathBuf;

use crate::{
    BindingData, BindingInfo, CommandBuffer, DrawCommandType, MeshFlags, Pass, RenderContext,
    RenderPass, RenderPassData, RenderTarget, ShaderStage, StoreOperation, TextureId,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource, ResourceTrait};
use inox_uid::{generate_random_uid, INVALID_UID};

pub const BLIT_PIPELINE: &str = "pipelines/Blit.render_pipeline";
pub const BLIT_PASS_NAME: &str = "BlitPass";

pub struct BlitPass {
    render_pass: Resource<RenderPass>,
    binding_data: BindingData,
    source_texture_id: TextureId,
}
unsafe impl Send for BlitPass {}
unsafe impl Sync for BlitPass {}

impl Pass for BlitPass {
    fn name(&self) -> &str {
        BLIT_PASS_NAME
    }
    fn static_name() -> &'static str {
        BLIT_PASS_NAME
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
        inox_profiler::scoped_profile!("blit_pass::create");

        let data = RenderPassData {
            name: BLIT_PASS_NAME.to_string(),
            store_color: StoreOperation::Store,
            store_depth: StoreOperation::Store,
            render_target: RenderTarget::Screen,
            pipeline: PathBuf::from(BLIT_PIPELINE),
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
            binding_data: BindingData::default(),
            source_texture_id: INVALID_UID,
        }
    }
    fn init(&mut self, render_context: &mut RenderContext) {
        inox_profiler::scoped_profile!("blit_pass::init");

        if self.source_texture_id.is_nil() || render_context.render_buffers.textures.is_empty() {
            return;
        }

        let mut pass = self.render_pass.get_mut();

        self.binding_data.add_texture(
            &render_context.texture_handler,
            &self.source_texture_id,
            BindingInfo {
                group_index: 0,
                binding_index: 0,
                stage: ShaderStage::Fragment,
                ..Default::default()
            },
        );
        self.binding_data
            .send_to_gpu(render_context, BLIT_PASS_NAME);

        pass.init(render_context, &self.binding_data, None, None);
    }
    fn update(&self, render_context: &mut RenderContext, command_buffer: &mut CommandBuffer) {
        inox_profiler::scoped_profile!("blit_pass::update");

        if self.source_texture_id.is_nil() {
            return;
        }

        let pass = self.render_pass.get();
        let buffers = render_context.buffers();
        let pipeline = pass.pipeline().get();
        if !pipeline.is_initialized() {
            return;
        }
        let mut render_pass = pass.begin(
            render_context,
            &self.binding_data,
            &buffers,
            &pipeline,
            command_buffer,
        );
        {
            inox_profiler::gpu_scoped_profile!(
                &mut render_pass,
                &render_context.core.device,
                "blit_pass",
            );
            pass.draw(render_context, render_pass, 0..3, 0..1);
        }
    }
}

impl BlitPass {
    pub fn render_pass(&self) -> &Resource<RenderPass> {
        &self.render_pass
    }
    pub fn set_source(&mut self, id: &TextureId) -> &mut Self {
        self.source_texture_id = *id;
        self
    }
}
