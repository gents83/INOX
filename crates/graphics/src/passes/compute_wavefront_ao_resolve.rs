use std::path::PathBuf;

use inox_core::ContextRc;
use inox_render::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData,
    ConstantDataRw, GPUVector, Pass, RenderContext, RenderContextRc, ShaderStage, TextureView,
};
use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;

use crate::{wavefront_auxiliary_bindings, AoRecord, RadiancePackedData};

pub const COMPUTE_WAVEFRONT_AO_RESOLVE_PIPELINE: &str =
    "pipelines/ComputeWavefrontAoResolve.compute_pipeline";
pub const COMPUTE_WAVEFRONT_AO_RESOLVE_NAME: &str = "ComputeWavefrontAoResolvePass";

pub struct ComputeWavefrontAoResolvePass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    gbuffer_material: GPUVector<RadiancePackedData>,
    ao_state: GPUVector<AoRecord>,
    dimensions: (u32, u32),
}

unsafe impl Send for ComputeWavefrontAoResolvePass {}
unsafe impl Sync for ComputeWavefrontAoResolvePass {}

impl Pass for ComputeWavefrontAoResolvePass {
    fn name(&self) -> &str {
        COMPUTE_WAVEFRONT_AO_RESOLVE_NAME
    }
    fn static_name() -> &'static str {
        COMPUTE_WAVEFRONT_AO_RESOLVE_NAME
    }
    fn is_active(&self, _render_context: &RenderContext) -> bool {
        !self.gbuffer_material.read().unwrap().is_empty()
            && !self.ao_state.read().unwrap().is_empty()
    }
    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self {
        let data = ComputePassData {
            name: COMPUTE_WAVEFRONT_AO_RESOLVE_NAME.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_WAVEFRONT_AO_RESOLVE_PIPELINE)],
        };
        Self {
            compute_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &data,
                None,
            ),
            binding_data: BindingData::new(render_context, COMPUTE_WAVEFRONT_AO_RESOLVE_NAME),
            constant_data: render_context.global_buffers().constant_data.clone(),
            gbuffer_material: render_context
                .global_buffers()
                .vector::<RadiancePackedData>(),
            ao_state: render_context
                .global_buffers()
                .vector_with_id(wavefront_auxiliary_bindings().ao_state),
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
                &mut *self.gbuffer_material.write().unwrap(),
                Some("GBuffer Material"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.ao_state.write().unwrap(),
                Some("AO State"),
                BindingInfo {
                    group_index: 0,
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
        if self.ao_state.read().unwrap().is_empty() {
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

impl ComputeWavefrontAoResolvePass {
    pub fn set_dimensions(&mut self, dimensions: (u32, u32)) -> &mut Self {
        self.dimensions = dimensions;
        self
    }
}
