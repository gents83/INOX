use std::path::PathBuf;

use inox_render::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData, ConstantDataRw, GPUBuffer, GPUVector, Pass, RenderContext, RenderContextRc, ShaderStage, TextureView,
};
use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;

use crate::passes::pathtracing_common::{PathTracingCounters, ShadowRay, GEOMETRY_BUFFER_UID, SCENE_BUFFER_UID};
use crate::RadiancePackedData;

pub const COMPUTE_PATHTRACING_SHADOW_PIPELINE: &str =
    "pipelines/ComputePathtracingShadow.compute_pipeline";
pub const COMPUTE_PATHTRACING_SHADOW_NAME: &str = "ComputePathTracingShadowPass";

pub struct ComputePathTracingShadowPass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    shadow_rays: GPUVector<ShadowRay>,
    counters: GPUBuffer<PathTracingCounters>,
    data_buffer_1: GPUVector<RadiancePackedData>,
}
unsafe impl Send for ComputePathTracingShadowPass {}
unsafe impl Sync for ComputePathTracingShadowPass {}

impl Pass for ComputePathTracingShadowPass {
    fn name(&self) -> &str {
        COMPUTE_PATHTRACING_SHADOW_NAME
    }
    fn static_name() -> &'static str {
        COMPUTE_PATHTRACING_SHADOW_NAME
    }
    fn is_active(&self, _render_context: &RenderContext) -> bool {
        true
    }
    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self
    where
        Self: Sized,
    {
        let data = ComputePassData {
            name: COMPUTE_PATHTRACING_SHADOW_NAME.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_PATHTRACING_SHADOW_PIPELINE)],
        };

        let shadow_rays = render_context.global_buffers().vector::<ShadowRay>();
        let counters = render_context.global_buffers().buffer::<PathTracingCounters>();
        let data_buffer_1 = render_context.global_buffers().vector::<RadiancePackedData>();

        Self {
            compute_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &data,
                None,
            ),
            constant_data: render_context.global_buffers().constant_data.clone(),
            shadow_rays,
            counters,
            data_buffer_1,
            binding_data: BindingData::new(render_context, COMPUTE_PATHTRACING_SHADOW_NAME),
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("pathtracing_shadow_pass::init");

        let geometry_buffer = render_context
            .global_buffers()
            .vector_with_id::<u32>(GEOMETRY_BUFFER_UID);
        let scene_buffer = render_context
            .global_buffers()
            .vector_with_id::<u32>(SCENE_BUFFER_UID);

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
                &mut *geometry_buffer.write().unwrap(),
                Some("GeometryBuffer"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *scene_buffer.write().unwrap(),
                Some("SceneBuffer"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.shadow_rays.write().unwrap(),
                Some("ShadowRays"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.data_buffer_1.write().unwrap(),
                Some("DataBuffer1"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.counters.write().unwrap(),
                Some("Counters"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            );

        let mut pass = self.compute_pass.get_mut();
        pass.init(render_context, &mut self.binding_data, None);
    }

    fn update(
        &mut self,
        render_context: &RenderContext,
        _surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
         if self.render_targets[0].is_nil() {
            return;
        }

        inox_profiler::scoped_profile!("pathtracing_shadow_pass::update");

        let pass = self.compute_pass.get();
        let width = self.constant_data.read().unwrap().screen_size()[0] as u32;
        let height = self.constant_data.read().unwrap().screen_size()[1] as u32;

        let x = width.div_ceil(8);
        let y = height.div_ceil(8);

        pass.dispatch(
            render_context,
            &mut self.binding_data,
            command_buffer,
            x,
            y,
            1,
        );
    }
}

impl ComputePathTracingShadowPass {
    pub fn set_render_targets(&mut self, diffuse: &TextureId, specular: &TextureId, shadow: &TextureId, ao: &TextureId) {
        self.render_targets = [*diffuse, *specular, *shadow, *ao];
    }
}
