use std::path::PathBuf;

use inox_render::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData, ConstantDataRw, DEFAULT_HEIGHT, DEFAULT_WIDTH, GPUBuffer, GPULight, GPUMaterial, GPUTexture, GPUVector, Pass, RenderContext, RenderContextRc, SamplerType, ShaderStage, TextureView
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::{generate_random_uid, Uid};

use crate::pathtracing_common::*;

pub const COMPUTE_PATHTRACING_LIGHTING_PIPELINE: &str =
    "pipelines/ComputePathTracingLighting.compute_pipeline";
pub const COMPUTE_PATHTRACING_LIGHTING_NAME: &str = "ComputePathTracingLightingPass";

pub struct ComputePathTracingLightingPass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    materials: GPUBuffer<GPUMaterial>,
    textures: GPUBuffer<GPUTexture>,
    lights: GPUBuffer<GPULight>,
    surface: GPUVector<SurfacePackedData>,
    rays: GPUVector<RayPackedData>,
    radiance: GPUVector<RadiancePackedData>,
    throughput: GPUVector<ThroughputPackedData>,
    shadow_data: GPUVector<ShadowDataPackedData>,
    next_rays: GPUVector<RayPackedData>,
    dimensions: (u32, u32),
    rays_uid: Uid,
    next_rays_uid: Uid,
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
        Self::create_with_uids(
            context,
            render_context,
            RAYS_UID,
            NEXT_RAYS_UID,
        )
    }
    fn init(&mut self, render_context: &RenderContext) {
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
                &mut *self.materials.write().unwrap(),
                Some("Materials"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Uniform | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.textures.write().unwrap(),
                Some("Textures"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Uniform | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.lights.write().unwrap(),
                Some("Lights"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 3,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Uniform | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.surface.write().unwrap(),
                Some("Surface"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.rays.write().unwrap(),
                Some("Rays"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.radiance.write().unwrap(),
                Some("Radiance"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.throughput.write().unwrap(),
                Some("Throughput"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 3,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.shadow_data.write().unwrap(),
                Some("ShadowData"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 4,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.next_rays.write().unwrap(),
                Some("NextRays"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 5,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                    ..Default::default()
                },
            );

        self.binding_data
            .add_default_sampler(
                BindingInfo {
                    group_index: 2,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
                SamplerType::Default,
            )
            .add_material_textures(BindingInfo {
                group_index: 2,
                binding_index: 1,
                stage: ShaderStage::Compute,
                ..Default::default()
            });

        let mut pass = self.compute_pass.get_mut();
        pass.init(render_context, &mut self.binding_data, None);
    }

    fn update(
        &mut self,
        render_context: &RenderContext,
        _surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
        inox_profiler::scoped_profile!("pathtracing_lighting_pass::update");

        let pass = self.compute_pass.get();

        let x_pixels_managed_in_shader = 8;
        let y_pixels_managed_in_shader = 8;
        let x = self.dimensions.0.div_ceil(x_pixels_managed_in_shader);
        let y = self.dimensions.1.div_ceil(y_pixels_managed_in_shader);

        let rays_len = self.rays.read().unwrap().len();
        let next_rays_len = self.next_rays.read().unwrap().len();
        let surface_len = self.surface.read().unwrap().len();
        let required_len = (self.dimensions.0 * self.dimensions.1) as usize * SIZE_OF_DATA_BUFFER_ELEMENT;
        let required_surface_len = required_len * 4; // 16 floats vs 4

        if rays_len < required_len {
            self.rays.write().unwrap().resize(required_len, RayPackedData(0.));
            self.rays.write().unwrap().mark_as_dirty(render_context);
        }
        if next_rays_len < required_len {
            self.next_rays.write().unwrap().resize(required_len, RayPackedData(0.));
            self.next_rays.write().unwrap().mark_as_dirty(render_context);
        }
        if surface_len < required_surface_len {
            self.surface.write().unwrap().resize(required_surface_len, SurfacePackedData(0.));
            self.surface.write().unwrap().mark_as_dirty(render_context);
        }

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
    pub fn create_with_uids(
        context: &ContextRc,
        render_context: &RenderContextRc,
        rays_uid: Uid,
        next_rays_uid: Uid,
    ) -> Self {
        let data = ComputePassData {
            name: COMPUTE_PATHTRACING_LIGHTING_NAME.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_PATHTRACING_LIGHTING_PIPELINE)],
        };

        let shadow_data = render_context
            .global_buffers()
            .vector_with_id::<ShadowDataPackedData>(SHADOW_DATA_UID);
        let next_rays = render_context
            .global_buffers()
            .vector_with_id::<RayPackedData>(next_rays_uid);

        let len = (DEFAULT_WIDTH * DEFAULT_HEIGHT) as usize * SIZE_OF_DATA_BUFFER_ELEMENT;
        shadow_data.write().unwrap().resize(
            len * 3, // Shadow needs less data? Just placeholder for now.
            ShadowDataPackedData(0.),
        );
        next_rays.write().unwrap().resize(
            len,
            RayPackedData(0.),
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
            surface: render_context
                .global_buffers()
                .vector_with_id::<SurfacePackedData>(SURFACE_UID),
            rays: render_context
                .global_buffers()
                .vector_with_id::<RayPackedData>(rays_uid),
            radiance: render_context
                .global_buffers()
                .vector_with_id::<RadiancePackedData>(RADIANCE_UID),
            throughput: render_context
                .global_buffers()
                .vector_with_id::<ThroughputPackedData>(THROUGHPUT_UID),
            shadow_data,
            next_rays,
            binding_data: BindingData::new(render_context, COMPUTE_PATHTRACING_LIGHTING_NAME),
            dimensions: (0, 0),
            rays_uid,
            next_rays_uid,
        }
    }
    pub fn set_dimensions(&mut self, dimensions: (u32, u32)) -> &mut Self {
        self.dimensions = dimensions;
        self
    }
}
