use std::path::PathBuf;

use inox_render::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ConstantDataRw, Pass, RenderContext,
    RenderContextRc, RenderPass, RenderPassBeginData, RenderPassData, RenderTarget, ShaderStage,
    StoreOperation, Texture, TextureView,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Handle, Resource, ResourceTrait};
use inox_uid::generate_random_uid;

pub const FINALIZE_PIPELINE: &str = "pipelines/Finalize.render_pipeline";
pub const FINALIZE_NAME: &str = "FinalizePass";
pub const NUM_FRAMES_OF_HISTORY: usize = 2;

pub struct FinalizePass {
    render_pass: Resource<RenderPass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    direct_texture: Handle<Texture>,
    indirect_diffuse_texture: Handle<Texture>,
    indirect_specular_texture: Handle<Texture>,
    shadow_texture: Handle<Texture>,
    ao_texture: Handle<Texture>,
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
    fn is_active(&self, _render_context: &RenderContext) -> bool {
        true
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
            direct_texture: None,
            indirect_diffuse_texture: None,
            indirect_specular_texture: None,
            shadow_texture: None,
            ao_texture: None,
            frame_textures: [NONE_TEXTURE_VALUE; NUM_FRAMES_OF_HISTORY],
            frame_index: 0,
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        if self.frame_textures.iter().any(|h| h.is_none())
            || self.direct_texture.is_none()
            || self.indirect_diffuse_texture.is_none()
            || self.indirect_specular_texture.is_none()
            || self.shadow_texture.is_none()
            || self.ao_texture.is_none()
        {
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
            .add_buffer(
                &mut *self.constant_data.write().unwrap(),
                Some("ConstantData"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 0,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Uniform | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_texture(
                self.direct_texture.as_ref().unwrap().id(),
                0,
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_texture(
                self.indirect_diffuse_texture.as_ref().unwrap().id(),
                0,
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_texture(
                self.indirect_specular_texture.as_ref().unwrap().id(),
                0,
                BindingInfo {
                    group_index: 0,
                    binding_index: 3,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_texture(
                self.frame_textures[previous_frame_index]
                    .as_ref()
                    .unwrap()
                    .id(),
                0,
                BindingInfo {
                    group_index: 0,
                    binding_index: 4,
                    stage: ShaderStage::Fragment,
                    ..Default::default()
                },
            )
            .add_texture(
                self.shadow_texture.as_ref().unwrap().id(),
                0,
                BindingInfo {
                    group_index: 0,
                    binding_index: 5,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_texture(
                self.ao_texture.as_ref().unwrap().id(),
                0,
                BindingInfo {
                    group_index: 0,
                    binding_index: 6,
                    stage: ShaderStage::Fragment,
                    flags: BindingFlags::Read,
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
        if self.frame_textures.iter().any(|h| h.is_none())
            || self.direct_texture.is_none()
            || self.indirect_diffuse_texture.is_none()
            || self.indirect_specular_texture.is_none()
            || self.shadow_texture.is_none()
            || self.ao_texture.is_none()
        {
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
        #[allow(unused_mut)]
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
    pub fn set_direct_texture(&mut self, texture: &Resource<Texture>) -> &mut Self {
        self.direct_texture = Some(texture.clone());
        self
    }
    pub fn set_indirect_diffuse_texture(&mut self, texture: &Resource<Texture>) -> &mut Self {
        self.indirect_diffuse_texture = Some(texture.clone());
        self
    }
    pub fn set_indirect_specular_texture(&mut self, texture: &Resource<Texture>) -> &mut Self {
        self.indirect_specular_texture = Some(texture.clone());
        self
    }
    pub fn set_shadow_texture(&mut self, texture: &Resource<Texture>) -> &mut Self {
        self.shadow_texture = Some(texture.clone());
        self
    }
    pub fn set_ao_texture(&mut self, texture: &Resource<Texture>) -> &mut Self {
        self.ao_texture = Some(texture.clone());
        self
    }
}
