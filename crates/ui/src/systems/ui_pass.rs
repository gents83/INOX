use std::path::PathBuf;

use inox_core::ContextRc;
use inox_graphics::{
    AsBinding, BindingData, BindingInfo, GpuBuffer, Pass, RenderContext, RenderCoreContext,
    RenderPass, RenderPassData, RenderTarget, ShaderStage, StoreOperation,
};
use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;

const UI_PIPELINE: &str = "pipelines/UI.render_pipeline";
pub const UI_PASS_NAME: &str = "UIPass";

#[repr(C, align(16))]
#[derive(Default, Clone, Copy, PartialEq)]
pub struct UIPassData {
    pub ui_scale: f32,
    is_dirty: bool,
}

impl AsBinding for UIPassData {
    fn is_dirty(&self) -> bool {
        self.is_dirty
    }
    fn set_dirty(&mut self, dirty: bool) {
        self.is_dirty = dirty;
    }
    fn size(&self) -> u64 {
        std::mem::size_of::<Self>() as _
    }
    fn fill_buffer(&self, render_core_context: &RenderCoreContext, buffer: &mut GpuBuffer) {
        buffer.add_to_gpu_buffer(render_core_context, &[self.ui_scale]);
    }
}

pub struct UIPass {
    render_pass: Resource<RenderPass>,
    binding_data: BindingData,
    custom_data: UIPassData,
}
unsafe impl Send for UIPass {}
unsafe impl Sync for UIPass {}

impl Pass for UIPass {
    fn name(&self) -> &str {
        UI_PASS_NAME
    }
    fn static_name() -> &'static str {
        UI_PASS_NAME
    }
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
                None,
            ),
            binding_data: BindingData::default(),
            custom_data: UIPassData::default(),
        }
    }
    fn init(&mut self, render_context: &mut RenderContext) {
        let mut pass = self.render_pass.get_mut();
        let render_texture = pass.render_texture_id();
        let depth_texture = pass.depth_texture_id();

        self.binding_data
            .add_uniform_data(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.constant_data,
                BindingInfo {
                    group_index: 0,
                    binding_index: 0,
                    stage: ShaderStage::VertexAndFragment,
                    ..Default::default()
                },
            )
            .add_storage_data(
                &render_context.core,
                &render_context.binding_data_buffer,
                render_context.render_buffers.materials_mut(),
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::VertexAndFragment,
                    read_only: true,
                    ..Default::default()
                },
            )
            .add_textures_data(
                &render_context.texture_handler,
                render_texture,
                depth_texture,
                BindingInfo {
                    group_index: 1,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_storage_data(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut self.custom_data,
                BindingInfo {
                    group_index: 2,
                    binding_index: 0,
                    stage: ShaderStage::VertexAndFragment,
                    read_only: true,
                    ..Default::default()
                },
            );
        self.binding_data.send_to_gpu(render_context);

        pass.init_pipelines(render_context, &self.binding_data);
    }
    fn update(&mut self, render_context: &RenderContext) {
        let pass = self.render_pass.get();

        let mut encoder = render_context.core.new_encoder();

        let render_pass = pass.begin(render_context, &self.binding_data, &mut encoder);
        pass.draw(render_context, render_pass);

        render_context.core.submit(encoder);
    }
}

impl UIPass {
    pub fn render_pass(&self) -> &Resource<RenderPass> {
        &self.render_pass
    }
    pub fn set_ui_scale(&mut self, ui_scale: f32) {
        if self.custom_data.ui_scale != ui_scale {
            self.custom_data.ui_scale = ui_scale;
            self.custom_data.is_dirty = true;
        }
    }
}
