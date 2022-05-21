use std::path::PathBuf;

use inox_core::ContextRc;
use inox_graphics::{
    platform::is_indirect_mode_enabled, AsBufferBinding, BindingData, DataBuffer, Pass,
    RenderContext, RenderMode, RenderPass, RenderPassData, RenderTarget, ShaderStage,
    StoreOperation,
};
use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;

const UI_PIPELINE: &str = "pipelines/UI.pipeline";
pub const UI_PASS_NAME: &str = "UIPass";

#[repr(C, align(16))]
#[derive(Default, Clone, Copy, PartialEq)]
pub struct UIPassData {
    pub ui_scale: f32,
}

impl AsBufferBinding for UIPassData {
    fn size(&self) -> u64 {
        std::mem::size_of::<Self>() as _
    }

    fn fill_buffer(&self, render_context: &RenderContext, buffer: &mut DataBuffer) {
        buffer.add_to_gpu_buffer(render_context, &[*self]);
    }
}

pub struct UIPass {
    render_pass: Resource<RenderPass>,
    binding_data: BindingData,
    custom_data: UIPassData,
    need_to_update_buffer: bool,
}
unsafe impl Send for UIPass {}
unsafe impl Sync for UIPass {}

impl Pass for UIPass {
    fn create(context: &ContextRc) -> Self
    where
        Self: Sized,
    {
        let data = RenderPassData {
            name: UI_PASS_NAME.to_string(),
            store_color: StoreOperation::Store,
            render_target: RenderTarget::Screen,
            pipelines: vec![PathBuf::from(UI_PIPELINE)],
            ..Default::default()
        };
        Self {
            render_pass: RenderPass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                data,
            ),
            binding_data: BindingData::default(),
            custom_data: UIPassData::default(),
            need_to_update_buffer: true,
        }
    }
    fn prepare(&mut self, render_context: &RenderContext) {
        let pass = self.render_pass.get();

        let encoder = render_context.new_encoder();

        let render_texture = pass.render_texture_id();
        let depth_texture = pass.depth_texture_id();

        if self.need_to_update_buffer {
            self.custom_data.mark_as_dirty(render_context);
            self.need_to_update_buffer = false;
        }

        self.binding_data
            .add_uniform_data(
                render_context,
                0,
                0,
                &render_context.constant_data,
                ShaderStage::VertexAndFragment,
            )
            .add_storage_data(
                render_context,
                0,
                1,
                &render_context.dynamic_data,
                ShaderStage::VertexAndFragment,
                true,
            )
            .add_textures_data(
                1,
                &render_context.texture_handler,
                render_texture,
                depth_texture,
                ShaderStage::Fragment,
            )
            .add_storage_data(
                render_context,
                2,
                0,
                &self.custom_data,
                ShaderStage::VertexAndFragment,
                true,
            );
        self.binding_data.send_to_gpu(render_context);

        let pipelines = pass.pipelines();
        pipelines.iter().for_each(|pipeline| {
            if render_context
                .graphics_data
                .get()
                .instance_count(pipeline.id())
                == 0
            {
                return;
            }

            let render_format = render_context.render_format(&pass);
            let depth_format = render_context.depth_format(&pass);

            pipeline.get_mut().init(
                render_context,
                render_format,
                depth_format,
                &self.binding_data,
            );
        });

        render_context.submit(encoder);
    }
    fn update(&mut self, render_context: &RenderContext) {
        let pass = self.render_pass.get();

        pass.pipelines().iter().for_each(|pipeline| {
            if is_indirect_mode_enabled() && pass.data().render_mode == RenderMode::Indirect {
                render_context
                    .graphics_data
                    .get_mut()
                    .fill_command_buffer(render_context, pipeline.id());
            }
        });

        let mut encoder = render_context.new_encoder();

        let render_pass = pass.begin(render_context, &self.binding_data, &mut encoder);
        pass.draw(render_context, render_pass);

        render_context.submit(encoder);
    }
}

impl UIPass {
    pub fn render_pass(&self) -> &Resource<RenderPass> {
        &self.render_pass
    }
    pub fn set_custom_data(&mut self, data: UIPassData) {
        if self.custom_data != data {
            self.need_to_update_buffer = true;
            self.custom_data = data;
        }
    }
}
