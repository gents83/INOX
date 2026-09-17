use std::path::PathBuf;

use inox_core::ContextRc;
use inox_render::{
    AsBinding, BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData,
    ConstantDataRw, GPUVector, Pass, RenderContext, RenderContextRc, ShaderStage, TextureView,
};
use inox_resources::{DataTypeResource, Resource};
use inox_uid::{generate_random_uid, generate_static_uid_from_string, Uid};

use crate::{
    wavefront_auxiliary_bindings, AccumulationRecord, AoRecord, RayPackedData, ReservoirRecord,
    WAVEFRONT_SCENE_ID,
};

pub const COMPUTE_WAVEFRONT_DIRECT_CANDIDATES_PIPELINE: &str =
    "pipelines/ComputeWavefrontDirectCandidates.compute_pipeline";
pub const COMPUTE_WAVEFRONT_DIRECT_CANDIDATES_NAME: &str = "ComputeWavefrontDirectCandidatesPass";
pub const WAVEFRONT_RESERVOIR_ID: Uid =
    generate_static_uid_from_string("WAVEFRONT_DIRECT_RESERVOIR");
pub const WAVEFRONT_AO_ID: Uid = generate_static_uid_from_string("WAVEFRONT_AO");
pub const WAVEFRONT_DIRECT_ACCUMULATOR_ID: Uid =
    generate_static_uid_from_string("WAVEFRONT_DIRECT_ACCUMULATOR");

pub struct ComputeWavefrontDirectCandidatesPass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    scene_data: GPUVector<u32>,
    gbuffer_world: GPUVector<RayPackedData>,
    reservoirs: GPUVector<ReservoirRecord>,
    ao_state: GPUVector<AoRecord>,
    direct_accumulator: GPUVector<AccumulationRecord>,
    dimensions: (u32, u32),
}

unsafe impl Send for ComputeWavefrontDirectCandidatesPass {}
unsafe impl Sync for ComputeWavefrontDirectCandidatesPass {}

impl Pass for ComputeWavefrontDirectCandidatesPass {
    fn name(&self) -> &str {
        COMPUTE_WAVEFRONT_DIRECT_CANDIDATES_NAME
    }

    fn static_name() -> &'static str {
        COMPUTE_WAVEFRONT_DIRECT_CANDIDATES_NAME
    }

    fn is_active(&self, _render_context: &RenderContext) -> bool {
        !self.scene_data.read().unwrap().is_empty()
            && !self.gbuffer_world.read().unwrap().is_empty()
    }

    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self {
        let data = ComputePassData {
            name: COMPUTE_WAVEFRONT_DIRECT_CANDIDATES_NAME.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_WAVEFRONT_DIRECT_CANDIDATES_PIPELINE)],
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
                COMPUTE_WAVEFRONT_DIRECT_CANDIDATES_NAME,
            ),
            constant_data: render_context.global_buffers().constant_data.clone(),
            scene_data: render_context
                .global_buffers()
                .vector_with_id(WAVEFRONT_SCENE_ID),
            gbuffer_world: render_context.global_buffers().vector::<RayPackedData>(),
            reservoirs: render_context
                .global_buffers()
                .vector_with_id(WAVEFRONT_RESERVOIR_ID),
            ao_state: render_context
                .global_buffers()
                .vector_with_id(wavefront_auxiliary_bindings().ao_state),
            direct_accumulator: render_context
                .global_buffers()
                .vector_with_id(WAVEFRONT_DIRECT_ACCUMULATOR_ID),
            dimensions: (0, 0),
        }
    }

    fn init(&mut self, render_context: &RenderContext) {
        let packed_scene = render_context.global_buffers().packed_scene_data();
        if let Err(error) = packed_scene.validate() {
            eprintln!("Skipping invalid packed scene upload: {error}");
            return;
        }
        if packed_scene.words().len() > std::mem::size_of::<inox_render::PackedSceneHeader>() / 4 {
            let mut scene_data = self.scene_data.write().unwrap();
            if scene_data.as_slice() != packed_scene.words() {
                *scene_data = packed_scene.words().to_vec();
                render_context.mark_as_dirty(scene_data.buffer_id());
            }
        }
        if !self.is_active(render_context) || self.reservoirs.read().unwrap().is_empty() {
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
            &mut *self.scene_data.write().unwrap(),
            Some("Packed Scene"),
            BindingInfo {
                group_index: 0,
                binding_index: 1,
                stage: ShaderStage::Compute,
                flags: BindingFlags::Storage | BindingFlags::Read,
                ..Default::default()
            },
        );
        self.binding_data.add_buffer(
            &mut *self.gbuffer_world.write().unwrap(),
            Some("GBuffer World"),
            BindingInfo {
                group_index: 1,
                binding_index: 0,
                stage: ShaderStage::Compute,
                flags: BindingFlags::Storage | BindingFlags::Read,
                ..Default::default()
            },
        );
        self.binding_data.add_buffer(
            &mut *self.reservoirs.write().unwrap(),
            Some("Direct Reservoirs"),
            BindingInfo {
                group_index: 1,
                binding_index: 1,
                stage: ShaderStage::Compute,
                flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                ..Default::default()
            },
        );
        self.binding_data.add_buffer(
            &mut *self.ao_state.write().unwrap(),
            Some("AO State"),
            BindingInfo {
                group_index: 1,
                binding_index: 2,
                stage: ShaderStage::Compute,
                flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                ..Default::default()
            },
        );
        self.binding_data.add_buffer(
            &mut *self.direct_accumulator.write().unwrap(),
            Some("Direct Accumulator"),
            BindingInfo {
                group_index: 1,
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
        if self.reservoirs.read().unwrap().is_empty() {
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

impl ComputeWavefrontDirectCandidatesPass {
    pub fn set_dimensions(&mut self, dimensions: (u32, u32)) -> &mut Self {
        self.dimensions = dimensions;
        let count = dimensions.0 as usize * dimensions.1 as usize;
        self.reservoirs
            .write()
            .unwrap()
            .resize(count, ReservoirRecord::default());
        self.ao_state
            .write()
            .unwrap()
            .resize(count, AoRecord::default());
        self.direct_accumulator
            .write()
            .unwrap()
            .resize(count, AccumulationRecord::default());
        self
    }
}
