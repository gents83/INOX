use std::path::PathBuf;

use inox_messenger::Listener;
use inox_render::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData,
    ConstantDataRw, Pass, RenderContext, RenderContextRc, ShaderStage, Texture, TextureView,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Handle, Resource};
use inox_uid::generate_random_uid;

use crate::CullingEvent;

pub const DEPTH_FIRST_PIPELINE: &str = "pipelines/ComputeDepthFirst.compute_pipeline";
pub const DEPTH_FIRST_PASS_NAME: &str = "DepthFirstPass";

pub struct DepthFirstPass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    depth_texture: Handle<Texture>,
    hzb_texture: Handle<Texture>,
    listener: Listener,
    update_pyramid: bool,
}
unsafe impl Send for DepthFirstPass {}
unsafe impl Sync for DepthFirstPass {}

impl Pass for DepthFirstPass {
    fn name(&self) -> &str {
        DEPTH_FIRST_PASS_NAME
    }
    fn static_name() -> &'static str {
        DEPTH_FIRST_PASS_NAME
    }
    fn is_active(&self, _render_context: &RenderContext) -> bool {
        true
    }
    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self
    where
        Self: Sized,
    {
        let compute_data = ComputePassData {
            name: DEPTH_FIRST_PASS_NAME.to_string(),
            pipelines: vec![PathBuf::from(DEPTH_FIRST_PIPELINE)],
        };
        let listener = Listener::new(context.message_hub());
        listener.register::<CullingEvent>();
        Self {
            compute_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &compute_data,
                None,
            ),
            constant_data: render_context.global_buffers().constant_data.clone(),
            binding_data: BindingData::new(render_context, DEPTH_FIRST_PASS_NAME),
            depth_texture: None,
            hzb_texture: None,
            listener,
            update_pyramid: true,
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("compute_depth_first_pass::init");

        if self.depth_texture.is_none() || self.hzb_texture.is_none() {
            return;
        }
        if let Some(depth_texture) = self.depth_texture.as_ref() {
            if depth_texture.get().texture_index() < 0 {
                return;
            }
        }

        self.process_messages();

        if !self.update_pyramid {
            return;
        }

        self.binding_data
            .add_buffer(
                &mut *self.constant_data.write().unwrap(),
                Some("ConstantData"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Uniform | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_texture(
                self.depth_texture.as_ref().unwrap().id(),
                0,
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_texture(
                self.hzb_texture.as_ref().unwrap().id(),
                0,
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                    ..Default::default()
                },
            );

        let mut pass = self.compute_pass.get_mut();
        pass.init(render_context, &mut self.binding_data, Some("main"));
    }

    fn update(
        &mut self,
        render_context: &RenderContext,
        _surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
        inox_profiler::scoped_profile!("compute_depth_first_pass::update");

        if self.depth_texture.is_none() || self.hzb_texture.is_none() {
            return;
        }
        if let Some(depth_texture) = self.depth_texture.as_ref() {
            if depth_texture.get().texture_index() < 0 {
                return;
            }
        }

        if !self.update_pyramid {
            return;
        }

        let dimensions = self.depth_texture.as_ref().unwrap().get().dimensions();

        let work_group_count_x = (dimensions.0).div_ceil(8);
        let work_group_count_y = (dimensions.1).div_ceil(8);

        let pass = self.compute_pass.get();
        pass.dispatch(
            render_context,
            &mut self.binding_data,
            command_buffer,
            work_group_count_x,
            work_group_count_y,
            1,
        );
    }
}

impl DepthFirstPass {
    pub fn set_depth_texture(&mut self, texture: Resource<Texture>) -> &mut Self {
        self.depth_texture = Some(texture);
        self
    }
    pub fn set_hzb_texture(&mut self, texture: Resource<Texture>) -> &mut Self {
        self.hzb_texture = Some(texture);
        self
    }
    fn process_messages(&mut self) {
        self.listener
            .process_messages(|event: &CullingEvent| match event {
                CullingEvent::FreezeCamera => {
                    self.update_pyramid = false;
                }
                CullingEvent::UnfreezeCamera => {
                    self.update_pyramid = true;
                }
            });
    }
}
