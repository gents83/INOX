use std::path::PathBuf;

use crate::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData,
    ConstantDataRw, DrawCommandType, MeshFlags, OutputPass, Pass, RenderContext, ShaderStage,
    Texture, TextureFormat, TextureId, TextureUsage, TextureView,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Handle, Resource};
use inox_uid::{generate_random_uid, INVALID_UID};

pub const COMPUTE_FINALIZE_PIPELINE: &str = "pipelines/ComputeFinalize.compute_pipeline";
pub const COMPUTE_FINALIZE_NAME: &str = "ComputeFinalizePass";

pub struct ComputeFinalizePass {
    context: ContextRc,
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    render_target: Handle<Texture>,
    radiance_texture: TextureId,
}
unsafe impl Send for ComputeFinalizePass {}
unsafe impl Sync for ComputeFinalizePass {}

impl Pass for ComputeFinalizePass {
    fn name(&self) -> &str {
        COMPUTE_FINALIZE_NAME
    }
    fn static_name() -> &'static str {
        COMPUTE_FINALIZE_NAME
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
        let data = ComputePassData {
            name: COMPUTE_FINALIZE_NAME.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_FINALIZE_PIPELINE)],
        };

        Self {
            context: context.clone(),
            compute_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &data,
                None,
            ),
            constant_data: render_context.constant_data.clone(),
            binding_data: BindingData::new(render_context, COMPUTE_FINALIZE_NAME),
            render_target: None,
            radiance_texture: INVALID_UID,
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("finalize_pass::init");

        if self.render_target.is_none() || self.radiance_texture.is_nil() {
            return;
        }

        self.binding_data
            .add_uniform_buffer(
                &mut *self.constant_data.write().unwrap(),
                Some("ConstantData"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_texture(
                self.render_target.as_ref().unwrap().id(),
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::ReadWrite | BindingFlags::Storage,
                },
            )
            .add_texture(
                &self.radiance_texture,
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            );

        let mut pass = self.compute_pass.get_mut();
        pass.init(render_context, &mut self.binding_data);
    }

    fn update(
        &mut self,
        render_context: &RenderContext,
        _surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
        if self.render_target.is_none() || self.radiance_texture.is_nil() {
            return;
        }

        inox_profiler::scoped_profile!("finalize_pass::update");

        let pass = self.compute_pass.get();

        let x_pixels_managed_in_shader = 16;
        let y_pixels_managed_in_shader = 16;
        let dimensions = self.render_target.as_ref().unwrap().get().dimensions();
        let x = (x_pixels_managed_in_shader
            * ((dimensions.0 + x_pixels_managed_in_shader - 1) / x_pixels_managed_in_shader))
            / x_pixels_managed_in_shader;
        let y = (y_pixels_managed_in_shader
            * ((dimensions.1 + y_pixels_managed_in_shader - 1) / y_pixels_managed_in_shader))
            / y_pixels_managed_in_shader;

        pass.dispatch(
            render_context,
            &mut self.binding_data,
            command_buffer,
            x,
            y,
            1,
        );
    }
}

impl OutputPass for ComputeFinalizePass {
    fn render_targets_id(&self) -> Option<Vec<TextureId>> {
        Some([*self.render_target.as_ref().unwrap().id()].to_vec())
    }
    fn depth_target_id(&self) -> Option<TextureId> {
        None
    }
}

impl ComputeFinalizePass {
    pub fn set_radiance_texture(&mut self, texture_id: &TextureId) -> &mut Self {
        self.radiance_texture = *texture_id;
        self
    }
    pub fn add_render_target_with_resolution(
        &mut self,
        width: u32,
        height: u32,
        render_format: TextureFormat,
    ) -> &mut Self {
        self.render_target = Some(Texture::create_from_format(
            self.context.shared_data(),
            self.context.message_hub(),
            width,
            height,
            render_format,
            TextureUsage::TextureBinding
                | TextureUsage::CopySrc
                | TextureUsage::CopyDst
                | TextureUsage::RenderAttachment
                | TextureUsage::StorageBinding,
        ));
        self
    }
}
