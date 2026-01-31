use std::path::PathBuf;

use inox_render::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData, ConstantDataRw, DEFAULT_HEIGHT, DEFAULT_WIDTH, GPUBuffer, GPULight, GPUMaterial, GPUTexture, GPUVector, Pass, RenderContext, RenderContextRc, ShaderStage, TextureView,
};
use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;

use crate::{
    passes::pathtracing_common::{PathTracingCounters, Ray, ShadowRay, SurfaceData, NEXT_RAYS_UID, SURFACE_DATA_UID},
    RadiancePackedData,
};

pub const COMPUTE_PATHTRACING_LIGHTING_PIPELINE: &str =
    "pipelines/ComputePathtracingLighting.compute_pipeline";
pub const COMPUTE_PATHTRACING_LIGHTING_NAME: &str = "ComputePathTracingLightingPass";

pub struct ComputePathTracingLightingPass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    materials: GPUBuffer<GPUMaterial>,
    textures: GPUBuffer<GPUTexture>,
    lights: GPUBuffer<GPULight>,
    rays: GPUVector<Ray>,
    next_rays: GPUVector<Ray>,
    surface_data: GPUVector<SurfaceData>,
    shadow_rays: GPUVector<ShadowRay>,
    counters: GPUBuffer<PathTracingCounters>,
    data_buffer_1: GPUVector<RadiancePackedData>,
}
unsafe impl Send for ComputePathTracingLightingPass {}
unsafe impl Sync for ComputePathTracingLightingPass {}

impl Pass for ComputePathTracingLightingPass {
    fn name(&self) -> &str {
        COMPUTE_PATHTRACING_LIGHTING_NAME
    }
    fn static_name() -> &'static str {
        COMPUTE_PATHTRACING_LIGHTING_NAME
    }
    fn is_active(&self, _render_context: &RenderContext) -> bool {
        true
    }
    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self
    where
        Self: Sized,
    {
        let data = ComputePassData {
            name: COMPUTE_PATHTRACING_LIGHTING_NAME.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_PATHTRACING_LIGHTING_PIPELINE)],
        };

        let rays = render_context.global_buffers().vector::<Ray>();
        let next_rays = render_context.global_buffers().vector_with_id::<Ray>(NEXT_RAYS_UID);
        let shadow_rays = render_context.global_buffers().vector::<ShadowRay>();
        let counters = render_context.global_buffers().buffer::<PathTracingCounters>();
        let surface_data = render_context.global_buffers().vector_with_id::<SurfaceData>(SURFACE_DATA_UID);
        let data_buffer_1 = render_context.global_buffers().vector::<RadiancePackedData>();

        shadow_rays.write().unwrap().resize(
            (DEFAULT_WIDTH * DEFAULT_HEIGHT) as usize,
            ShadowRay::default(),
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
            materials: render_context.global_buffers().buffer::<GPUMaterial>(),
            textures: render_context.global_buffers().buffer::<GPUTexture>(),
            lights: render_context.global_buffers().buffer::<GPULight>(),
            rays,
            next_rays,
            surface_data,
            shadow_rays,
            counters,
            data_buffer_1,
            binding_data: BindingData::new(render_context, COMPUTE_PATHTRACING_LIGHTING_NAME),
        }
    }
    fn init(&mut self, _render_context: &RenderContext) {
        inox_profiler::scoped_profile!("pathtracing_lighting_pass::init");

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
                &mut *self.surface_data.write().unwrap(),
                Some("SurfaceData"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.materials.write().unwrap(),
                Some("Materials"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Uniform | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.textures.write().unwrap(),
                Some("Textures"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Uniform | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.lights.write().unwrap(),
                Some("Lights"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Uniform | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.shadow_rays.write().unwrap(),
                Some("ShadowRays"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 3,
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
                    binding_index: 4,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.data_buffer_1.write().unwrap(),
                Some("DataBuffer1"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 7,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                    ..Default::default()
                },
            );

        // Rays and NextRays
        self.binding_data.add_buffer(
            &mut *self.rays.write().unwrap(),
            Some("Rays"),
            BindingInfo {
                group_index: 1,
                binding_index: 5,
                stage: ShaderStage::Compute,
                flags: BindingFlags::Storage | BindingFlags::Read,
                ..Default::default()
            },
        );
        self.binding_data.add_buffer(
            &mut *self.next_rays.write().unwrap(),
            Some("NextRays"),
            BindingInfo {
                group_index: 1,
                binding_index: 6,
                stage: ShaderStage::Compute,
                flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                ..Default::default()
            },
        );
    }

    fn update(
        &mut self,
        render_context: &RenderContext,
        _surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
        inox_profiler::scoped_profile!("pathtracing_lighting_pass::update");

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

impl ComputePathTracingLightingPass {
    pub fn set_ray_buffers(&mut self, rays: &GPUVector<Ray>, next_rays: &GPUVector<Ray>) {
        self.rays = rays.clone();
        self.next_rays = next_rays.clone();

        self.binding_data.add_buffer(
            &mut *self.rays.write().unwrap(),
            Some("Rays"),
            BindingInfo {
                group_index: 1,
                binding_index: 5,
                stage: ShaderStage::Compute,
                flags: BindingFlags::Storage | BindingFlags::Read,
                ..Default::default()
            },
        );
        self.binding_data.add_buffer(
            &mut *self.next_rays.write().unwrap(),
            Some("NextRays"),
            BindingInfo {
                group_index: 1,
                binding_index: 6,
                stage: ShaderStage::Compute,
                flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                ..Default::default()
            },
        );
    }
}
