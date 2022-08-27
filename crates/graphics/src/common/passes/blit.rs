use std::path::PathBuf;

use crate::{
    AsBinding, BindingData, BindingInfo, CommandBuffer, DrawCommandType, GpuBuffer, MeshFlags,
    Pass, RenderContext, RenderCoreContext, RenderPass, RenderPassData, RenderTarget, ShaderStage,
    StoreOperation, TextureId,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::{generate_random_uid, INVALID_UID};

pub const BLIT_PIPELINE: &str = "pipelines/Blit.render_pipeline";
pub const BLIT_PASS_NAME: &str = "BlitPass";

#[repr(C, align(16))]
#[derive(Default, Debug, Clone, Copy)]
struct Data {
    pub texture_index: usize,
}

#[derive(Default, Clone, Copy)]
pub struct BlitPassData {
    is_dirty: bool,
    data: Data,
}

impl AsBinding for BlitPassData {
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
    fn is_active(&self, _render_context: &mut RenderContext) -> bool {
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
                data,
                None,
            ),
            binding_data: BindingData::default(),
            data: BlitPassData::default(),
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
            .render_buffers
            .textures
            .index_of(&self.source_texture_id)
        {
            if self.data.data.texture_index != texture_index {
                self.data.data.texture_index += texture_index;
                self.data.set_dirty(true);
            }
        } else {
            return;
        }

        self.binding_data
            .add_uniform_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut self.data,
                BindingInfo {
                    group_index: 0,
                    binding_index: 0,
                    stage: ShaderStage::Vertex,
                    ..Default::default()
                },
            )
            .add_storage_buffer(
                &render_context.core,
                &render_context.binding_data_buffer,
                &mut render_context.render_buffers.textures,
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_textures(
                &render_context.texture_handler,
                render_textures,
                None,
                BindingInfo {
                    group_index: 1,
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

        if self.source_texture_id.is_nil() || render_context.render_buffers.textures.is_empty() {
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

impl BlitPass {
    pub fn render_pass(&self) -> &Resource<RenderPass> {
        &self.render_pass
    }
    pub fn set_source(&mut self, id: &TextureId) -> &mut Self {
        self.source_texture_id = *id;
        self
    }
}
