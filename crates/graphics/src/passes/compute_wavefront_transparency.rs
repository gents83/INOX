use std::path::PathBuf;

use inox_core::ContextRc;
use inox_render::{
    AsBinding, BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData,
    ConstantDataRw, GPUVector, Pass, RenderContext, RenderContextRc, ShaderStage, TextureView,
};
use inox_resources::{DataTypeResource, Resource};
use inox_uid::{generate_random_uid, generate_static_uid_from_string, Uid};

use crate::{
    queue_capacity_u32, wavefront_auxiliary_bindings, AccumulationRecord, QueueHeader,
    RadiancePackedData, TransparentHitRecord, WAVEFRONT_SCENE_ID,
};

pub const COMPUTE_WAVEFRONT_TRANSPARENCY_PIPELINE: &str =
    "pipelines/ComputeWavefrontTransparency.compute_pipeline";
pub const COMPUTE_WAVEFRONT_TRANSPARENCY_NAME: &str = "ComputeWavefrontTransparencyPass";
pub const COMPUTE_WAVEFRONT_TRANSPARENCY_RESOLVE_PIPELINE: &str =
    "pipelines/ComputeWavefrontTransparencyResolve.compute_pipeline";
pub const COMPUTE_WAVEFRONT_TRANSPARENCY_RESOLVE_NAME: &str =
    "ComputeWavefrontTransparencyResolvePass";
pub const WAVEFRONT_TRANSPARENT_QUEUE_ID: Uid =
    generate_static_uid_from_string("WAVEFRONT_TRANSPARENT_QUEUE");
pub const WAVEFRONT_TRANSPARENT_HEADER_ID: Uid =
    generate_static_uid_from_string("WAVEFRONT_TRANSPARENT_HEADER");
const MAX_TRANSPARENT_QUEUE_CAPACITY: usize = 1_048_576;

pub struct ComputeWavefrontTransparencyPass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    scene_data: GPUVector<u32>,
    gbuffer_material: GPUVector<RadiancePackedData>,
    queue: GPUVector<TransparentHitRecord>,
    header: GPUVector<QueueHeader>,
    dimensions: (u32, u32),
}

pub struct ComputeWavefrontTransparencyResolvePass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    queue: GPUVector<TransparentHitRecord>,
    header: GPUVector<QueueHeader>,
    final_accumulator: GPUVector<AccumulationRecord>,
    dimensions: (u32, u32),
}

unsafe impl Send for ComputeWavefrontTransparencyPass {}
unsafe impl Sync for ComputeWavefrontTransparencyPass {}
unsafe impl Send for ComputeWavefrontTransparencyResolvePass {}
unsafe impl Sync for ComputeWavefrontTransparencyResolvePass {}

impl Pass for ComputeWavefrontTransparencyPass {
    fn name(&self) -> &str {
        COMPUTE_WAVEFRONT_TRANSPARENCY_NAME
    }
    fn static_name() -> &'static str {
        COMPUTE_WAVEFRONT_TRANSPARENCY_NAME
    }
    fn is_active(&self, _render_context: &RenderContext) -> bool {
        !self.scene_data.read().unwrap().is_empty() && !self.queue.read().unwrap().is_empty()
    }
    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self {
        let data = ComputePassData {
            name: COMPUTE_WAVEFRONT_TRANSPARENCY_NAME.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_WAVEFRONT_TRANSPARENCY_PIPELINE)],
        };
        let header = render_context
            .global_buffers()
            .vector_with_id::<QueueHeader>(wavefront_auxiliary_bindings().transparent_header);
        header.write().unwrap().resize(1, QueueHeader::default());
        Self {
            compute_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &data,
                None,
            ),
            binding_data: BindingData::new(render_context, COMPUTE_WAVEFRONT_TRANSPARENCY_NAME),
            constant_data: render_context.global_buffers().constant_data.clone(),
            scene_data: render_context
                .global_buffers()
                .vector_with_id(WAVEFRONT_SCENE_ID),
            gbuffer_material: render_context
                .global_buffers()
                .vector::<RadiancePackedData>(),
            queue: render_context
                .global_buffers()
                .vector_with_id(wavefront_auxiliary_bindings().transparent_queue),
            header,
            dimensions: (0, 0),
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        if !self.is_active(render_context) {
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
                &mut *self.gbuffer_material.write().unwrap(),
                Some("GBuffer Material"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.header.write().unwrap(),
                Some("Transparent Queue Header"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.queue.write().unwrap(),
                Some("Transparent Queue"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                    ..Default::default()
                },
            );
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
        if self.queue.read().unwrap().is_empty() {
            return;
        }
        let mut header = self.header.write().unwrap();
        header[0].count = 0;
        header[0].overflowed = 0;
        header[0].capacity = queue_capacity_u32(self.queue.read().unwrap().len());
        render_context.mark_as_dirty(header.buffer_id());
        self.compute_pass.get().dispatch(
            render_context,
            &mut self.binding_data,
            command_buffer,
            self.dimensions.0.div_ceil(8),
            self.dimensions.1.div_ceil(8),
            1,
        );
    }
}

impl ComputeWavefrontTransparencyPass {
    pub fn set_dimensions(&mut self, dimensions: (u32, u32)) -> &mut Self {
        self.dimensions = dimensions;
        let capacity =
            (dimensions.0 as usize * dimensions.1 as usize * 4).min(MAX_TRANSPARENT_QUEUE_CAPACITY);
        self.queue
            .write()
            .unwrap()
            .resize(capacity, TransparentHitRecord::default());
        self.header.write().unwrap()[0].capacity = queue_capacity_u32(capacity);
        self
    }
}

impl Pass for ComputeWavefrontTransparencyResolvePass {
    fn name(&self) -> &str {
        COMPUTE_WAVEFRONT_TRANSPARENCY_RESOLVE_NAME
    }
    fn static_name() -> &'static str {
        COMPUTE_WAVEFRONT_TRANSPARENCY_RESOLVE_NAME
    }
    fn is_active(&self, _render_context: &RenderContext) -> bool {
        !self.final_accumulator.read().unwrap().is_empty() && !self.queue.read().unwrap().is_empty()
    }
    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self {
        let data = ComputePassData {
            name: COMPUTE_WAVEFRONT_TRANSPARENCY_RESOLVE_NAME.to_string(),
            pipelines: vec![PathBuf::from(
                COMPUTE_WAVEFRONT_TRANSPARENCY_RESOLVE_PIPELINE,
            )],
        };
        Self {
            compute_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &data,
                None,
            ),
            binding_data: BindingData::new(
                render_context,
                COMPUTE_WAVEFRONT_TRANSPARENCY_RESOLVE_NAME,
            ),
            constant_data: render_context.global_buffers().constant_data.clone(),
            queue: render_context
                .global_buffers()
                .vector_with_id(wavefront_auxiliary_bindings().transparent_queue),
            header: render_context
                .global_buffers()
                .vector_with_id(wavefront_auxiliary_bindings().transparent_header),
            final_accumulator: render_context
                .global_buffers()
                .vector_with_id(crate::WAVEFRONT_FINAL_ACCUMULATOR_ID),
            dimensions: (0, 0),
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        if !self.is_active(render_context) {
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
                &mut *self.header.write().unwrap(),
                Some("Transparent Queue Header"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.queue.write().unwrap(),
                Some("Transparent Queue"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.final_accumulator.write().unwrap(),
                Some("Final Accumulator"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 3,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                    ..Default::default()
                },
            );
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
        if self.queue.read().unwrap().is_empty() {
            return;
        }
        self.compute_pass.get().dispatch(
            render_context,
            &mut self.binding_data,
            command_buffer,
            self.queue.read().unwrap().len().div_ceil(64) as u32,
            1,
            1,
        );
    }
}

impl ComputeWavefrontTransparencyResolvePass {
    pub fn set_dimensions(&mut self, dimensions: (u32, u32)) -> &mut Self {
        self.dimensions = dimensions;
        self
    }
}
