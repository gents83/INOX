use std::path::PathBuf;

use inox_core::ContextRc;
use inox_render::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData,
    ConstantDataRw, GPUVector, Pass, RenderContext, RenderContextRc, ShaderStage, TextureView,
};
use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;

use crate::{
    wavefront_auxiliary_bindings, AccumulationRecord, AoRecord, WAVEFRONT_DIRECT_ACCUMULATOR_ID,
    WAVEFRONT_FINAL_ACCUMULATOR_ID, WAVEFRONT_INDIRECT_ACCUMULATOR_ID,
};

pub const COMPUTE_WAVEFRONT_COMPOSITE_PIPELINE: &str =
    "pipelines/ComputeWavefrontComposite.compute_pipeline";
pub const COMPUTE_WAVEFRONT_COMPOSITE_NAME: &str = "ComputeWavefrontCompositePass";

pub struct ComputeWavefrontCompositePass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    final_accumulator: GPUVector<AccumulationRecord>,
    direct_accumulator: GPUVector<AccumulationRecord>,
    indirect_accumulator: GPUVector<AccumulationRecord>,
    ao_state: GPUVector<AoRecord>,
    dimensions: (u32, u32),
}

unsafe impl Send for ComputeWavefrontCompositePass {}
unsafe impl Sync for ComputeWavefrontCompositePass {}

impl Pass for ComputeWavefrontCompositePass {
    fn name(&self) -> &str {
        COMPUTE_WAVEFRONT_COMPOSITE_NAME
    }

    fn static_name() -> &'static str {
        COMPUTE_WAVEFRONT_COMPOSITE_NAME
    }

    fn is_active(&self, _render_context: &RenderContext) -> bool {
        !self.final_accumulator.read().unwrap().is_empty()
            && !self.direct_accumulator.read().unwrap().is_empty()
            && !self.indirect_accumulator.read().unwrap().is_empty()
    }

    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self {
        let data = ComputePassData {
            name: COMPUTE_WAVEFRONT_COMPOSITE_NAME.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_WAVEFRONT_COMPOSITE_PIPELINE)],
        };
        Self {
            compute_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &data,
                None,
            ),
            binding_data: BindingData::new(render_context, COMPUTE_WAVEFRONT_COMPOSITE_NAME),
            constant_data: render_context.global_buffers().constant_data.clone(),
            final_accumulator: render_context
                .global_buffers()
                .vector_with_id(WAVEFRONT_FINAL_ACCUMULATOR_ID),
            direct_accumulator: render_context
                .global_buffers()
                .vector_with_id(WAVEFRONT_DIRECT_ACCUMULATOR_ID),
            indirect_accumulator: render_context
                .global_buffers()
                .vector_with_id(WAVEFRONT_INDIRECT_ACCUMULATOR_ID),
            ao_state: render_context
                .global_buffers()
                .vector_with_id(wavefront_auxiliary_bindings().ao_state),
            dimensions: (0, 0),
        }
    }

    fn init(&mut self, render_context: &RenderContext) {
        if !self.is_active(render_context) || self.ao_state.read().unwrap().is_empty() {
            return;
        }
        self.binding_data.add_buffer(
            &mut *self.constant_data.write().unwrap(),
            Some("ConstantData"),
            BindingInfo {
                group_index: 0,
                binding_index: 0,
                stage: ShaderStage::Compute,
                flags: BindingFlags::Uniform | BindingFlags::Read,
                ..Default::default()
            },
        );
        self.binding_data.add_buffer(
            &mut *self.final_accumulator.write().unwrap(),
            Some("Final Accumulator"),
            BindingInfo {
                group_index: 0,
                binding_index: 1,
                stage: ShaderStage::Compute,
                flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                ..Default::default()
            },
        );
        self.binding_data.add_buffer(
            &mut *self.direct_accumulator.write().unwrap(),
            Some("Direct Accumulator"),
            BindingInfo {
                group_index: 0,
                binding_index: 2,
                stage: ShaderStage::Compute,
                flags: BindingFlags::Storage | BindingFlags::Read,
                ..Default::default()
            },
        );
        self.binding_data.add_buffer(
            &mut *self.ao_state.write().unwrap(),
            Some("AO State"),
            BindingInfo {
                group_index: 0,
                binding_index: 3,
                stage: ShaderStage::Compute,
                flags: BindingFlags::Storage | BindingFlags::Read,
                ..Default::default()
            },
        );
        self.binding_data.add_buffer(
            &mut *self.indirect_accumulator.write().unwrap(),
            Some("Indirect Accumulator"),
            BindingInfo {
                group_index: 0,
                binding_index: 4,
                stage: ShaderStage::Compute,
                flags: BindingFlags::Storage | BindingFlags::Read,
                ..Default::default()
            },
        );
        self.compute_pass
            .get_mut()
            .init(render_context, &mut self.binding_data, None);
    }

    fn resize(&mut self, dimensions: inox_render::FrameDimensions) {
        self.set_dimensions(dimensions.render_dimensions());
    }

    fn update(
        &mut self,
        render_context: &RenderContext,
        _surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
        if self.final_accumulator.read().unwrap().is_empty() {
            return;
        }
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

impl ComputeWavefrontCompositePass {
    pub fn set_dimensions(&mut self, dimensions: (u32, u32)) -> &mut Self {
        self.dimensions = dimensions;
        self.final_accumulator.write().unwrap().resize(
            dimensions.0 as usize * dimensions.1 as usize,
            AccumulationRecord::default(),
        );
        self
    }
}
