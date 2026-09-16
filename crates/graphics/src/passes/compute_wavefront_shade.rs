use std::path::PathBuf;

use inox_core::ContextRc;
use inox_render::{
    AsBinding, BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData,
    ConstantDataRw, GPUBuffer, GPUTexture, GPUVector, Pass, RenderContext, RenderContextRc,
    SamplerType, ShaderStage, TextureView,
};
use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;

use crate::{
    wavefront_primary_shade_bindings, AccumulationRecord, DispatchArgs, HitRecord, QueueHeader,
    RayQueueRecord, WavefrontQueueBindingIds, WAVEFRONT_INDIRECT_ACCUMULATOR_ID,
    WAVEFRONT_SCENE_ID,
};

pub const COMPUTE_WAVEFRONT_SHADE_PIPELINE: &str =
    "pipelines/ComputeWavefrontShade.compute_pipeline";
pub const COMPUTE_WAVEFRONT_SHADE_NAME: &str = "ComputeWavefrontShadePass";

pub struct ComputeWavefrontShadePass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    scene_data: GPUVector<u32>,
    textures: GPUBuffer<GPUTexture>,
    ray_queue: GPUVector<RayQueueRecord>,
    ray_queue_header: GPUVector<QueueHeader>,
    hit_queue: GPUVector<HitRecord>,
    hit_queue_header: GPUVector<QueueHeader>,
    indirect_accumulator: GPUVector<AccumulationRecord>,
    dispatch_args: GPUVector<DispatchArgs>,
    pass_name: String,
}

unsafe impl Send for ComputeWavefrontShadePass {}
unsafe impl Sync for ComputeWavefrontShadePass {}

impl Pass for ComputeWavefrontShadePass {
    fn name(&self) -> &str {
        &self.pass_name
    }

    fn static_name() -> &'static str {
        COMPUTE_WAVEFRONT_SHADE_NAME
    }

    fn is_active(&self, _render_context: &RenderContext) -> bool {
        !self.scene_data.read().unwrap().is_empty()
    }

    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self {
        Self::create_for_queues(
            context,
            render_context,
            wavefront_primary_shade_bindings(),
            COMPUTE_WAVEFRONT_SHADE_NAME,
        )
    }

    fn init(&mut self, render_context: &RenderContext) {
        if self.scene_data.read().unwrap().is_empty()
            || self.hit_queue.read().unwrap().is_empty()
            || self.indirect_accumulator.read().unwrap().is_empty()
        {
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
            .add_buffer(
                &mut *self.scene_data.write().unwrap(),
                Some("Packed Scene"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.ray_queue_header.write().unwrap(),
                Some("Wavefront Ray Queue Header"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.ray_queue.write().unwrap(),
                Some("Wavefront Ray Queue"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.hit_queue_header.write().unwrap(),
                Some("Wavefront Hit Queue Header"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.hit_queue.write().unwrap(),
                Some("Wavefront Hit Queue"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 3,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.indirect_accumulator.write().unwrap(),
                Some("Indirect Accumulator"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 4,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.textures.write().unwrap(),
                Some("Textures"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 5,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Uniform | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_default_sampler(
                BindingInfo {
                    group_index: 2,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
                SamplerType::Unfiltered,
            )
            .add_material_textures(BindingInfo {
                group_index: 2,
                binding_index: 1,
                stage: ShaderStage::Compute,
                ..Default::default()
            });
        self.compute_pass
            .get_mut()
            .init(render_context, &mut self.binding_data, None);
    }

    fn update(
        &mut self,
        render_context: &RenderContext,
        _surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
        if self.scene_data.read().unwrap().is_empty()
            || self
                .hit_queue_header
                .read()
                .unwrap()
                .first()
                .is_none_or(|header| header.count == 0)
        {
            return;
        }
        let buffers = render_context.buffers();
        let dispatch_args_id = self.dispatch_args.read().unwrap().buffer_id();
        self.compute_pass.get().dispatch_indirect(
            render_context,
            &mut self.binding_data,
            command_buffer,
            &buffers,
            dispatch_args_id,
        );
    }
}

impl ComputeWavefrontShadePass {
    pub fn create_for_queues(
        context: &ContextRc,
        render_context: &RenderContextRc,
        ids: WavefrontQueueBindingIds,
        pass_name: &str,
    ) -> Self {
        let data = ComputePassData {
            name: pass_name.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_WAVEFRONT_SHADE_PIPELINE)],
        };
        Self {
            compute_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &data,
                None,
            ),
            binding_data: BindingData::new(render_context, pass_name),
            constant_data: render_context.global_buffers().constant_data.clone(),
            scene_data: render_context
                .global_buffers()
                .vector_with_id(WAVEFRONT_SCENE_ID),
            textures: render_context.global_buffers().buffer::<GPUTexture>(),
            ray_queue: render_context
                .global_buffers()
                .vector_with_id(ids.ray_queue),
            ray_queue_header: render_context
                .global_buffers()
                .vector_with_id(ids.ray_header),
            hit_queue: render_context
                .global_buffers()
                .vector_with_id(ids.hit_queue),
            hit_queue_header: render_context
                .global_buffers()
                .vector_with_id(ids.hit_header),
            indirect_accumulator: render_context
                .global_buffers()
                .vector_with_id(WAVEFRONT_INDIRECT_ACCUMULATOR_ID),
            dispatch_args: render_context
                .global_buffers()
                .vector_with_id(ids.dispatch_args),
            pass_name: pass_name.to_string(),
        }
    }

    pub fn set_dimensions(&mut self, dimensions: (u32, u32)) -> &mut Self {
        self.indirect_accumulator.write().unwrap().resize(
            dimensions.0 as usize * dimensions.1 as usize,
            AccumulationRecord::default(),
        );
        self
    }
}
