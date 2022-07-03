use std::path::PathBuf;

use crate::{
    AsBinding, BindingData, BindingInfo, CommandBuffer, GpuBuffer, Pass, RenderContext,
    RenderCoreContext, RenderPass, RenderPassData, RenderTarget, ShaderStage, StoreOperation,
    TextureId,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::{generate_random_uid, INVALID_UID};

pub const BLIT_PIPELINE: &str = "pipelines/Blit.render_pipeline";
pub const BLIT_PASS_NAME: &str = "BlitPass";

#[repr(C, align(16))]
#[derive(Default, Clone, Copy, PartialEq)]
pub struct BlitPassData {
    pub texture_index: usize,
    is_dirty: bool,
}

impl AsBinding for BlitPassData {
    fn is_dirty(&self) -> bool {
        self.is_dirty
    }
    fn set_dirty(&mut self, dirty: bool) {
        self.is_dirty = dirty;
    }
    fn size(&self) -> u64 {
        std::mem::size_of::<usize>() as _
    }
    fn fill_buffer(&self, render_core_context: &RenderCoreContext, buffer: &mut GpuBuffer) {
        buffer.add_to_gpu_buffer(render_core_context, &[self.texture_index]);
    }
}

pub struct BlitPass {
    render_pass: Resource<RenderPass>,
    binding_data: BindingData,
    data: BlitPassData,
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
                data,
                None,
            ),
            binding_data: BindingData::default(),
            data: BlitPassData {
                texture_index: 0,
                is_dirty: true,
            },
            source_texture_id: INVALID_UID,
        }
    }
    fn init(&mut self, render_context: &mut RenderContext) {
        inox_profiler::scoped_profile!("blit_pass::init");

        if self.source_texture_id.is_nil() || render_context.render_buffers.textures.is_empty() {
            return;
        }

        let mut pass = self.render_pass.get_mut();
        let render_textures = pass.render_textures_id();

        if let Some(texture_index) = render_context
            .texture_handler
            .texture_index(&self.source_texture_id)
        {
            if self.data.texture_index != texture_index {
                self.data.texture_index.clone_from(&texture_index);
                self.data.set_dirty(true);
            }
        }

        self.binding_data
            .add_storage_data(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut self.data,
                BindingInfo {
                    group_index: 0,
                    binding_index: 0,
                    stage: ShaderStage::Vertex,
                    read_only: true,
                    ..Default::default()
                },
            )
            .add_storage_data(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.textures,
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Fragment,
                    read_only: true,
                    ..Default::default()
                },
            )
            .add_textures_data(
                &render_context.texture_handler,
                render_textures,
                None,
                BindingInfo {
                    group_index: 1,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            );
        self.binding_data.send_to_gpu(render_context);

        pass.init_pipeline(render_context, &self.binding_data, None, None);
    }
    fn update(&mut self, render_context: &mut RenderContext, command_buffer: &mut CommandBuffer) {
        inox_profiler::scoped_profile!("blit_pass::update");

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

impl BlitPass {
    pub fn render_pass(&self) -> &Resource<RenderPass> {
        &self.render_pass
    }
    pub fn set_source(&mut self, id: &TextureId) -> &mut Self {
        self.source_texture_id = *id;
        self
    }
}
