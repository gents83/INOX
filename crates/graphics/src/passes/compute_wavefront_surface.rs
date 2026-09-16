use std::path::PathBuf;

use inox_core::ContextRc;
use inox_render::{
    AsBinding, BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData,
    ConstantDataRw, GPUVector, Pass, RenderContext, RenderContextRc, ShaderStage, TextureId,
    TextureView,
};
use inox_resources::{DataTypeResource, Resource};
use inox_uid::{generate_random_uid, INVALID_UID};

use crate::{RadiancePackedData, RayPackedData, ThroughputPackedData, WAVEFRONT_SCENE_ID};

pub const COMPUTE_WAVEFRONT_SURFACE_PIPELINE: &str =
    "pipelines/ComputeWavefrontSurface.compute_pipeline";
pub const COMPUTE_WAVEFRONT_SURFACE_NAME: &str = "ComputeWavefrontSurfacePass";

pub struct ComputeWavefrontSurfacePass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    scene_data: GPUVector<u32>,
    data_buffer_0: GPUVector<RayPackedData>,
    data_buffer_1: GPUVector<RadiancePackedData>,
    data_buffer_2: GPUVector<ThroughputPackedData>,
    visibility_texture: TextureId,
    depth_texture: TextureId,
    dimensions: (u32, u32),
}

unsafe impl Send for ComputeWavefrontSurfacePass {}
unsafe impl Sync for ComputeWavefrontSurfacePass {}

impl Pass for ComputeWavefrontSurfacePass {
    fn name(&self) -> &str {
        COMPUTE_WAVEFRONT_SURFACE_NAME
    }
    fn static_name() -> &'static str {
        COMPUTE_WAVEFRONT_SURFACE_NAME
    }
    fn is_active(&self, _render_context: &RenderContext) -> bool {
        !self.visibility_texture.is_nil() && !self.depth_texture.is_nil()
    }
    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self {
        let data = ComputePassData {
            name: COMPUTE_WAVEFRONT_SURFACE_NAME.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_WAVEFRONT_SURFACE_PIPELINE)],
        };
        Self {
            compute_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &data,
                None,
            ),
            binding_data: BindingData::new(render_context, COMPUTE_WAVEFRONT_SURFACE_NAME),
            constant_data: render_context.global_buffers().constant_data.clone(),
            scene_data: render_context
                .global_buffers()
                .vector_with_id(WAVEFRONT_SCENE_ID),
            data_buffer_0: render_context.global_buffers().vector::<RayPackedData>(),
            data_buffer_1: render_context
                .global_buffers()
                .vector::<RadiancePackedData>(),
            data_buffer_2: render_context
                .global_buffers()
                .vector::<ThroughputPackedData>(),
            visibility_texture: INVALID_UID,
            depth_texture: INVALID_UID,
            dimensions: (0, 0),
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        if !self.is_active(render_context) {
            return;
        }
        let packed = render_context.global_buffers().packed_scene_data();
        if let Err(error) = packed.validate() {
            eprintln!("Skipping invalid packed scene upload: {error}");
            return;
        }
        if packed.words().len() <= std::mem::size_of::<inox_render::PackedSceneHeader>() / 4 {
            return;
        }
        {
            let mut scene = self.scene_data.write().unwrap();
            if scene.as_slice() != packed.words() {
                *scene = packed.words().to_vec();
                render_context.mark_as_dirty(scene.buffer_id());
            }
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
            .add_texture(
                &self.visibility_texture,
                0,
                BindingInfo {
                    group_index: 1,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_texture(
                &self.depth_texture,
                0,
                BindingInfo {
                    group_index: 1,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.data_buffer_0.write().unwrap(),
                Some("Surface State 0"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.data_buffer_1.write().unwrap(),
                Some("Surface State 1"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 3,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.data_buffer_2.write().unwrap(),
                Some("Surface State 2"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 4,
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
        if !self.is_active(render_context) {
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

impl ComputeWavefrontSurfacePass {
    pub fn set_visibility_texture(&mut self, id: &TextureId, dimensions: (u32, u32)) -> &mut Self {
        self.visibility_texture = *id;
        self.dimensions = dimensions;
        self
    }
    pub fn set_depth_texture(&mut self, id: &TextureId) -> &mut Self {
        self.depth_texture = *id;
        self
    }
}
