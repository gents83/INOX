use std::path::PathBuf;

use inox_render::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData, ConstantDataRw, DEFAULT_HEIGHT, DEFAULT_WIDTH, GPUBuffer, GPUInstance, GPULight, GPUMaterial, GPUMesh, GPUMeshlet, GPUTexture, GPUVector, GPUVertexAttributes, GPUVertexIndices, GPUVertexPosition, GPUTransform, INSTANCE_DATA_ID, Pass, RenderContext, RenderContextRc, ShaderStage, TextureView,
};
use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;

use crate::passes::pathtracing_common::{PathTracingCounters, Ray, RayHit, ShadowRay, NEXT_RAYS_UID};

pub const COMPUTE_PATHTRACING_SHADE_PIPELINE: &str =
    "pipelines/ComputePathtracingShade.compute_pipeline";
pub const COMPUTE_PATHTRACING_SHADE_NAME: &str = "ComputePathTracingShadePass";

pub struct ComputePathTracingShadePass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    indices: GPUBuffer<GPUVertexIndices>,
    vertices_positions: GPUBuffer<GPUVertexPosition>,
    vertices_attributes: GPUBuffer<GPUVertexAttributes>,
    instances: GPUVector<GPUInstance>,
    transforms: GPUVector<GPUTransform>,
    meshes: GPUBuffer<GPUMesh>,
    meshlets: GPUBuffer<GPUMeshlet>,
    materials: GPUBuffer<GPUMaterial>,
    textures: GPUBuffer<GPUTexture>,
    lights: GPUBuffer<GPULight>,
    rays: GPUVector<Ray>,
    next_rays: GPUVector<Ray>,
    hits: GPUVector<RayHit>,
    shadow_rays: GPUVector<ShadowRay>,
    counters: GPUBuffer<PathTracingCounters>,
    binding_names: Vec<String>,
}
unsafe impl Send for ComputePathTracingShadePass {}
unsafe impl Sync for ComputePathTracingShadePass {}

impl Pass for ComputePathTracingShadePass {
    fn name(&self) -> &str {
        COMPUTE_PATHTRACING_SHADE_NAME
    }
    fn static_name() -> &'static str {
        COMPUTE_PATHTRACING_SHADE_NAME
    }
    fn is_active(&self, _render_context: &RenderContext) -> bool {
        true
    }
    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self
    where
        Self: Sized,
    {
        let data = ComputePassData {
            name: COMPUTE_PATHTRACING_SHADE_NAME.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_PATHTRACING_SHADE_PIPELINE)],
        };

        let rays = render_context.global_buffers().vector::<Ray>();
        let next_rays = render_context.global_buffers().vector_with_id::<Ray>(NEXT_RAYS_UID);
        let hits = render_context.global_buffers().vector::<RayHit>();
        let shadow_rays = render_context.global_buffers().vector::<ShadowRay>();
        let counters = render_context.global_buffers().buffer::<PathTracingCounters>();

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
            indices: render_context.global_buffers().buffer::<GPUVertexIndices>(),
            vertices_positions: render_context
                .global_buffers()
                .buffer::<GPUVertexPosition>(),
            vertices_attributes: render_context
                .global_buffers()
                .buffer::<GPUVertexAttributes>(),
            instances: render_context
                .global_buffers()
                .vector_with_id::<GPUInstance>(INSTANCE_DATA_ID),
            transforms: render_context.global_buffers().vector::<GPUTransform>(),
            meshes: render_context.global_buffers().buffer::<GPUMesh>(),
            meshlets: render_context.global_buffers().buffer::<GPUMeshlet>(),
            materials: render_context.global_buffers().buffer::<GPUMaterial>(),
            textures: render_context.global_buffers().buffer::<GPUTexture>(),
            lights: render_context.global_buffers().buffer::<GPULight>(),
            rays,
            next_rays,
            hits,
            shadow_rays,
            counters,
            binding_data: BindingData::new(render_context, COMPUTE_PATHTRACING_SHADE_NAME),
            binding_names: Vec::new(),
        }
    }
    fn init(&mut self, _render_context: &RenderContext) {
        inox_profiler::scoped_profile!("pathtracing_shade_pass::init");

        // Bind common buffers
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
                &mut *self.indices.write().unwrap(),
                Some("Indices"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read | BindingFlags::Index,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.vertices_positions.write().unwrap(),
                Some("Vertices Positions"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read | BindingFlags::Vertex,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.vertices_attributes.write().unwrap(),
                Some("Vertices Attributes"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 3,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.instances.write().unwrap(),
                Some("Instances"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 4,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.transforms.write().unwrap(),
                Some("Transforms"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 5,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.meshes.write().unwrap(),
                Some("Meshes"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 6,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.meshlets.write().unwrap(),
                Some("Meshlets"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 7,
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
                &mut *self.hits.write().unwrap(),
                Some("Hits"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 3,
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
                    group_index: 1,
                    binding_index: 5,
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
                binding_index: 6,
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
                binding_index: 7,
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
        inox_profiler::scoped_profile!("pathtracing_shade_pass::update");

        let pass = self.compute_pass.get();
        let width = render_context.global_buffers().constant_data.read().unwrap().screen_width;
        let height = render_context.global_buffers().constant_data.read().unwrap().screen_height;

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

impl ComputePathTracingShadePass {
    pub fn set_ray_buffers(&mut self, rays: &GPUVector<Ray>, next_rays: &GPUVector<Ray>) {
        self.rays = rays.clone();
        self.next_rays = next_rays.clone();

        self.binding_data.add_buffer(
            &mut *self.rays.write().unwrap(),
            Some("Rays"),
            BindingInfo {
                group_index: 1,
                binding_index: 6,
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
                binding_index: 7,
                stage: ShaderStage::Compute,
                flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                ..Default::default()
            },
        );
    }
}
