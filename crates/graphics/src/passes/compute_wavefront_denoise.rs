use std::path::PathBuf;

use inox_core::ContextRc;
use inox_render::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData,
    ConstantDataRw, GPUVector, Pass, RenderContext, RenderContextRc, ShaderStage, TextureView,
    DENOISER_BILATERAL, DENOISER_TEMPORAL_VARIANCE,
};
use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;

use crate::{AccumulationRecord, WAVEFRONT_FINAL_ACCUMULATOR_ID};

pub const COMPUTE_WAVEFRONT_DENOISE_TEMPORAL_NAME: &str = "ComputeWavefrontDenoiseTemporalPass";
pub const COMPUTE_WAVEFRONT_DENOISE_BILATERAL_NAME: &str = "ComputeWavefrontDenoiseBilateralPass";
pub const COMPUTE_WAVEFRONT_DENOISE_TEMPORAL_PIPELINE: &str =
    "pipelines/ComputeWavefrontDenoiseTemporal.compute_pipeline";
pub const COMPUTE_WAVEFRONT_DENOISE_BILATERAL_PIPELINE: &str =
    "pipelines/ComputeWavefrontDenoiseBilateral.compute_pipeline";
const WORKGROUP_SIZE: u32 = 8;

#[derive(Clone, Copy)]
enum DenoiserKind {
    Temporal,
    Bilateral,
}

pub struct ComputeWavefrontDenoisePass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    final_accumulator: GPUVector<AccumulationRecord>,
    dimensions: (u32, u32),
    kind: DenoiserKind,
    name: &'static str,
}

unsafe impl Send for ComputeWavefrontDenoisePass {}
unsafe impl Sync for ComputeWavefrontDenoisePass {}

impl Pass for ComputeWavefrontDenoisePass {
    fn name(&self) -> &str {
        self.name
    }

    fn static_name() -> &'static str {
        COMPUTE_WAVEFRONT_DENOISE_TEMPORAL_NAME
    }

    fn is_active(&self, _render_context: &RenderContext) -> bool {
        let mode = self.constant_data.read().unwrap().denoiser_mode();
        matches!(
            (self.kind, mode),
            (DenoiserKind::Temporal, DENOISER_TEMPORAL_VARIANCE)
                | (DenoiserKind::Bilateral, DENOISER_BILATERAL)
        )
    }

    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self {
        Self::create_kind(context, render_context, DenoiserKind::Temporal)
    }

    fn init(&mut self, render_context: &RenderContext) {
        if self.final_accumulator.read().unwrap().is_empty() {
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
        // The final accumulator is full resolution, matching composite and
        // upscale. Render scale affects reconstruction quality, not dispatch
        // dimensions or storage strides.
        let (width, height) = self.dimensions;
        self.compute_pass.get().dispatch(
            render_context,
            &mut self.binding_data,
            command_buffer,
            width.div_ceil(WORKGROUP_SIZE),
            height.div_ceil(WORKGROUP_SIZE),
            1,
        );
    }
}

impl ComputeWavefrontDenoisePass {
    fn create_kind(
        context: &ContextRc,
        render_context: &RenderContextRc,
        kind: DenoiserKind,
    ) -> Self {
        let (name, pipeline) = match kind {
            DenoiserKind::Temporal => (
                COMPUTE_WAVEFRONT_DENOISE_TEMPORAL_NAME,
                COMPUTE_WAVEFRONT_DENOISE_TEMPORAL_PIPELINE,
            ),
            DenoiserKind::Bilateral => (
                COMPUTE_WAVEFRONT_DENOISE_BILATERAL_NAME,
                COMPUTE_WAVEFRONT_DENOISE_BILATERAL_PIPELINE,
            ),
        };
        let data = ComputePassData {
            name: name.to_string(),
            pipelines: vec![PathBuf::from(pipeline)],
        };
        Self {
            compute_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &data,
                None,
            ),
            binding_data: BindingData::new(render_context, name),
            constant_data: render_context.global_buffers().constant_data.clone(),
            final_accumulator: render_context
                .global_buffers()
                .vector_with_id(WAVEFRONT_FINAL_ACCUMULATOR_ID),
            dimensions: (0, 0),
            kind,
            name,
        }
    }

    pub fn create_bilateral(context: &ContextRc, render_context: &RenderContextRc) -> Self {
        Self::create_kind(context, render_context, DenoiserKind::Bilateral)
    }

    pub fn set_dimensions(&mut self, dimensions: (u32, u32)) -> &mut Self {
        self.dimensions = dimensions;
        self
    }
}
