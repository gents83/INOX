use std::path::PathBuf;

use inox_render::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ConstantDataRw, DrawCommandType,
    GPUVector, MeshFlags, Pass, RenderContext, RenderContextRc, RenderPass, RenderPassBeginData,
    RenderPassData, RenderTarget, ShaderStage, StoreOperation, Texture, TextureView,
    NUM_FRAMES_OF_HISTORY,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Handle, Resource, ResourceTrait};
use inox_uid::generate_random_uid;

use crate::RadiancePackedData;

pub const FINALIZE_PIPELINE: &str = "pipelines/Finalize.render_pipeline";
pub const FINALIZE_NAME: &str = "FinalizePass";

pub struct FinalizePass {
    render_pass: Resource<RenderPass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    data_buffer_1: GPUVector<RadiancePackedData>,
    frame_textures: [Handle<Texture>; NUM_FRAMES_OF_HISTORY],
    frame_index: usize,
}
unsafe impl Send for FinalizePass {}
unsafe impl Sync for FinalizePass {}

const NONE_TEXTURE_VALUE: Handle<Texture> = None;

impl Pass for FinalizePass {
    fn name(&self) -> &str {
        FINALIZE_NAME
    }
    fn static_name() -> &'static str {
        FINALIZE_NAME
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
    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self
    where
        Self: Sized,
    {
        let data = RenderPassData {
            name: FINALIZE_NAME.to_string(),
            store_color: StoreOperation::Store,
            store_depth: StoreOperation::Store,
            render_target: RenderTarget::Screen,
            pipeline: PathBuf::from(FINALIZE_PIPELINE),
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
            constant_data: render_context.global_buffers().constant_data.clone(),
            binding_data: BindingData::new(render_context, FINALIZE_NAME),
            data_buffer_1: render_context
                .global_buffers()
                .vector::<RadiancePackedData>(),
            frame_textures: [NONE_TEXTURE_VALUE; NUM_FRAMES_OF_HISTORY],
            frame_index: 0,
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        if self.frame_textures.iter().any(|h| h.is_none()) {
            return;
        }

        inox_profiler::scoped_profile!("finalize_pass::init");

        let previous_frame_index = if self.frame_index == 0 {
            NUM_FRAMES_OF_HISTORY - 1
        } else {
            self.frame_index - 1
        };

        let mut pass = self.render_pass.get_mut();
        pass.remove_all_render_targets()
            .add_render_target(self.frame_textures[self.frame_index].as_ref().unwrap());

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
                &mut *self.data_buffer_1.write().unwrap(),
                Some("DataBuffer_1"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::ReadWrite | BindingFlags::Storage,
                },
            )
            .add_texture(
                self.frame_textures[previous_frame_index]
                    .as_ref()
                    .unwrap()
                    .id(),
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
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
        if self.frame_textures.iter().any(|h| h.is_none()) {
            return;
        }

        inox_profiler::scoped_profile!("finalize_pass::update");

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
                "finalize_pass",
            );
            pass.draw(render_context, render_pass, 0..3, 0..1);

            self.frame_index = (self.frame_index + 1) % NUM_FRAMES_OF_HISTORY;
        }
    }
}

impl FinalizePass {
    pub fn set_frame_textures(
        &mut self,
        textures: [&Resource<Texture>; NUM_FRAMES_OF_HISTORY],
    ) -> &mut Self {
        textures.iter().enumerate().for_each(|(i, &t)| {
            self.frame_textures[i] = Some(t.clone());
        });
        self.render_pass
            .get_mut()
            .remove_all_render_targets()
            .add_render_target(textures[0]);
        self
    }
}
