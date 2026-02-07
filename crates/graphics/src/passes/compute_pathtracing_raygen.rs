use std::path::PathBuf;

use inox_render::{
    AsBinding, BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData, ConstantDataRw, DEFAULT_HEIGHT, DEFAULT_WIDTH, GPUVector, Pass, RenderContext, RenderContextRc, ShaderStage, TextureId, TextureView
};
use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::{generate_random_uid, INVALID_UID};

use crate::passes::pathtracing_common::{PathTracingCounters, Ray, RayHit, GEOMETRY_BUFFER_UID, SCENE_BUFFER_UID};
use crate::RadiancePackedData;

pub const COMPUTE_PATHTRACING_RAYGEN_PIPELINE: &str =
    "pipelines/ComputePathtracingRayGen.compute_pipeline";
pub const COMPUTE_PATHTRACING_RAYGEN_NAME: &str = "ComputePathTracingRayGenPass";

pub struct ComputePathTracingRayGenPass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    hits: GPUVector<RayHit>,
    rays: GPUVector<Ray>,
    counters: inox_render::GPUBuffer<PathTracingCounters>,
    data_buffer_1: GPUVector<RadiancePackedData>,
    visibility_texture: TextureId,
    depth_texture: TextureId,
    dimensions: (u32, u32),
}
unsafe impl Send for ComputePathTracingRayGenPass {}
unsafe impl Sync for ComputePathTracingRayGenPass {}

impl Pass for ComputePathTracingRayGenPass {
    fn name(&self) -> &str {
        COMPUTE_PATHTRACING_RAYGEN_NAME
    }
    fn static_name() -> &'static str {
        COMPUTE_PATHTRACING_RAYGEN_NAME
    }
    fn is_active(&self, _render_context: &RenderContext) -> bool {
        true
    }
    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self
    where
        Self: Sized,
    {
        let data = ComputePassData {
            name: COMPUTE_PATHTRACING_RAYGEN_NAME.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_PATHTRACING_RAYGEN_PIPELINE)],
        };

        let hits = render_context.global_buffers().vector::<RayHit>();
        let rays = render_context.global_buffers().vector::<Ray>();
        let counters = render_context.global_buffers().buffer::<PathTracingCounters>();
        let data_buffer_1 = render_context.global_buffers().vector::<RadiancePackedData>();

        // Pre-allocate hits and rays buffer
        hits.write().unwrap().resize(
            (DEFAULT_WIDTH * DEFAULT_HEIGHT) as usize,
            RayHit::default(),
        );
        rays.write().unwrap().resize(
            (DEFAULT_WIDTH * DEFAULT_HEIGHT) as usize,
            Ray::default(),
        );
        data_buffer_1.write().unwrap().resize(
            4 * (DEFAULT_WIDTH * DEFAULT_HEIGHT) as usize,
            RadiancePackedData(0.),
        );

        Self {
            compute_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &data,
                None,
            ),
            constant_data: render_context.global_buffers().constant_data.clone(),
            hits,
            rays,
            counters,
            data_buffer_1,
            binding_data: BindingData::new(render_context, COMPUTE_PATHTRACING_RAYGEN_NAME),
            visibility_texture: INVALID_UID,
            depth_texture: INVALID_UID,
            dimensions: (0, 0),
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("pathtracing_raygen_pass::init");

        if self.visibility_texture.is_nil() || self.depth_texture.is_nil() {
            return;
        }

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
                &mut *self.hits.write().unwrap(),
                Some("Hits"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 3,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.rays.write().unwrap(),
                Some("Rays"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 4,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.counters.write().unwrap(),
                Some("Counters"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 5,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                    ..Default::default()
                },
            )
            .add_texture(
                &self.visibility_texture,
                0,
                BindingInfo {
                    group_index: 0,
                    binding_index: 6,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_texture(
                &self.depth_texture,
                0,
                BindingInfo {
                    group_index: 0,
                    binding_index: 7,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.data_buffer_1.write().unwrap(),
                Some("DataBuffer1"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 8,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::ReadWrite,
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
        if self.visibility_texture.is_nil() {
            return;
        }

        inox_profiler::scoped_profile!("pathtracing_raygen_pass::update");

        {
            let mut counters = self.counters.write().unwrap();
            counters.data_mut()[0] = PathTracingCounters::default();
            counters.mark_as_dirty(render_context);
        }

        let pass = self.compute_pass.get();

        let x_pixels_managed_in_shader = 8;
        let y_pixels_managed_in_shader = 8;
        let x = self.dimensions.0.div_ceil(x_pixels_managed_in_shader);
        let y = self.dimensions.1.div_ceil(y_pixels_managed_in_shader);

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

impl ComputePathTracingRayGenPass {
    pub fn set_visibility_texture(
        &mut self,
        texture_id: &TextureId,
        dimensions: (u32, u32),
    ) -> &mut Self {
        self.dimensions = dimensions;
        self.visibility_texture = *texture_id;
        self
    }
    pub fn set_depth_texture(&mut self, texture_id: &TextureId) -> &mut Self {
        self.depth_texture = *texture_id;
        self
    }
}
