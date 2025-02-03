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

pub const DEPTH_PYRAMID_PIPELINE: &str = "pipelines/ComputeDepthPyramid.compute_pipeline";
pub const DEPTH_PYRAMID_PASS_NAME: &str = "DepthPyramidPass";

pub struct DepthPyramidPass {
    compute_pass: Resource<ComputePass>,
    render_context: RenderContextRc,
    binding_data: Vec<BindingData>,
    constant_data: ConstantDataRw,
    dimensions: (u32, u32),
    hzb_texture: Handle<Texture>,
    mip_levels: Vec<u32>,
    listener: Listener,
    update_pyramid: bool,
}
unsafe impl Send for DepthPyramidPass {}
unsafe impl Sync for DepthPyramidPass {}

impl Pass for DepthPyramidPass {
    fn name(&self) -> &str {
        DEPTH_PYRAMID_PASS_NAME
    }
    fn static_name() -> &'static str {
        DEPTH_PYRAMID_PASS_NAME
    }
    fn is_active(&self, _render_context: &RenderContext) -> bool {
        true
    }
    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self
    where
        Self: Sized,
    {
        let compute_data = ComputePassData {
            name: DEPTH_PYRAMID_PASS_NAME.to_string(),
            pipelines: vec![PathBuf::from(DEPTH_PYRAMID_PIPELINE)],
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
            render_context: render_context.clone(),
            constant_data: render_context.global_buffers().constant_data.clone(),
            binding_data: Vec::new(),
            mip_levels: Vec::new(),
            dimensions: (0, 0),
            hzb_texture: None,
            listener,
            update_pyramid: true,
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("compute_depth_pyramid_pass::init");

        if self.hzb_texture.is_none() {
            return;
        }

        self.process_messages();

        if !self.update_pyramid {
            return;
        }

        self.binding_data
            .iter_mut()
            .enumerate()
            .for_each(|(i, binding_data)| {
                binding_data
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
                    .add_buffer(
                        &mut self.mip_levels[i],
                        Some("MipLevel"),
                        BindingInfo {
                            group_index: 0,
                            binding_index: 1,
                            stage: ShaderStage::Compute,
                            flags: BindingFlags::Uniform | BindingFlags::Read,
                            ..Default::default()
                        },
                    )
                    .add_texture(
                        self.hzb_texture.as_ref().unwrap().id(),
                        i as u32,
                        BindingInfo {
                            group_index: 0,
                            binding_index: 2,
                            stage: ShaderStage::Compute,
                            flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                            ..Default::default()
                        },
                    )
                    .add_texture(
                        self.hzb_texture.as_ref().unwrap().id(),
                        (i + 1) as u32,
                        BindingInfo {
                            group_index: 0,
                            binding_index: 3,
                            stage: ShaderStage::Compute,
                            flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                            ..Default::default()
                        },
                    );

                let mut pass = self.compute_pass.get_mut();
                pass.init(render_context, binding_data, Some("main"));
            });
    }

    fn update(
        &mut self,
        render_context: &RenderContext,
        _surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
        inox_profiler::scoped_profile!("compute_depth_pyramid_pass::update");

        if self.hzb_texture.is_none() {
            return;
        }

        if !self.update_pyramid {
            return;
        }

        self.binding_data
            .iter_mut()
            .enumerate()
            .for_each(|(mip_level, binding_data)| {
                let work_group_count_x = ((self.dimensions.0 / (1 << (mip_level + 1))) + 7) / 8;
                let work_group_count_y = ((self.dimensions.1 / (1 << (mip_level + 1))) + 7) / 8;

                let pass = self.compute_pass.get();
                pass.dispatch(
                    render_context,
                    binding_data,
                    command_buffer,
                    work_group_count_x,
                    work_group_count_y,
                    1,
                );
            });
    }
}

impl DepthPyramidPass {
    pub fn set_depth_texture(&mut self, texture: Resource<Texture>) -> &mut Self {
        self.dimensions = texture.get().dimensions();
        let hzb_size = self.dimensions.0.max(self.dimensions.1).next_power_of_two();
        let mip_count = (f32::log2(hzb_size as f32) as u32).max(1);
        self.binding_data.clear();
        self.mip_levels.clear();
        for i in 1..mip_count {
            let binding_data = BindingData::new(
                &self.render_context,
                format!("{DEPTH_PYRAMID_PASS_NAME}_mip{i}").as_str(),
            );
            self.binding_data.push(binding_data);
            self.mip_levels.push(i);
        }
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
