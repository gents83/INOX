use std::path::PathBuf;

use inox_render::{
    BindingData, BindingInfo, CommandBuffer, Pass, RenderContext, RenderContextRc, RenderPass,
    RenderPassBeginData, RenderPassData, RenderTarget, ShaderStage, StoreOperation, TextureId,
    TextureView, NUM_FRAMES_OF_HISTORY,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource, ResourceTrait};
use inox_uid::{generate_random_uid, INVALID_UID};

pub const BLIT_PIPELINE: &str = "pipelines/Blit.render_pipeline";
pub const BLIT_PASS_NAME: &str = "BlitPass";

pub struct BlitPass {
    render_pass: Resource<RenderPass>,
    binding_data: BindingData,
    source_textures: [TextureId; NUM_FRAMES_OF_HISTORY],
    frame_index: usize,
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
    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self
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
            binding_data: BindingData::new(render_context, BLIT_PASS_NAME),
            source_textures: [INVALID_UID; NUM_FRAMES_OF_HISTORY],
            frame_index: 0,
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        if self.source_textures.iter().any(|v| v.is_nil()) {
            return;
        }

        inox_profiler::scoped_profile!("blit_pass::init");

        let mut pass = self.render_pass.get_mut();

        self.binding_data.add_texture(
            &self.source_textures[self.frame_index],
            0,
            BindingInfo {
                group_index: 0,
                binding_index: 0,
                stage: ShaderStage::Fragment,
                ..Default::default()
            },
        );

        pass.init(render_context, &mut self.binding_data, None, None);
    }
    fn update(
        &mut self,
        render_context: &RenderContext,
        surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
        if self.source_textures.iter().any(|v| v.is_nil()) {
            return;
        }

        inox_profiler::scoped_profile!("blit_pass::update");

        let pass = self.render_pass.get();
        let pipeline = pass.pipeline().get();
        if !pipeline.is_initialized() {
            return;
        }
        let buffers = render_context.buffers();
        let render_targets = render_context.texture_handler().render_targets();

        let render_pass_begin_data = RenderPassBeginData {
            render_core_context: &render_context.webgpu,
            buffers: &buffers,
            render_targets: render_targets.as_slice(),
            surface_view,
            command_buffer,
        };
        let mut render_pass = pass.begin(&mut self.binding_data, &pipeline, render_pass_begin_data);
        {
            inox_profiler::gpu_scoped_profile!(
                &mut render_pass,
                &render_context.webgpu.device,
                "blit_pass",
            );
            pass.draw(render_context, render_pass, 0..3, 0..1);

            self.frame_index = (self.frame_index + 1) % NUM_FRAMES_OF_HISTORY;
        }
    }
}

impl BlitPass {
    pub fn set_sources(&mut self, texture_ids: [&TextureId; NUM_FRAMES_OF_HISTORY]) -> &mut Self {
        texture_ids.iter().enumerate().for_each(|(i, &id)| {
            self.source_textures[i] = *id;
        });
        self
    }
}
