use std::path::PathBuf;

use inox_render::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData, ConstantDataRw, GPUBuffer, GPUMesh, GPUMeshlet, GPUVector, GPUVertexPosition, Pass, RenderContext, RenderContextRc, ShaderStage, TextureId, TextureView,
};
use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::{generate_random_uid, INVALID_UID};

use crate::passes::pathtracing_common::{PathTracingCounters, ShadowRay};
use inox_bvh::GPUBVHNode;
use crate::RadiancePackedData;

pub const COMPUTE_PATHTRACING_SHADOW_PIPELINE: &str =
    "pipelines/ComputePathtracingShadow.compute_pipeline";
pub const COMPUTE_PATHTRACING_SHADOW_NAME: &str = "ComputePathTracingShadowPass";

pub struct ComputePathTracingShadowPass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    meshes: GPUBuffer<GPUMesh>,
    meshlets: GPUBuffer<GPUMeshlet>,
    bvh: GPUBuffer<GPUBVHNode>,
    vertices_positions: GPUBuffer<GPUVertexPosition>,
    shadow_rays: GPUVector<ShadowRay>,
    counters: GPUBuffer<PathTracingCounters>,
    data_buffer_1: GPUVector<RadiancePackedData>,
    render_targets: [TextureId; 4], // Diffuse, Specular, Shadow, AO
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
            meshes: render_context.global_buffers().buffer::<GPUMesh>(),
            meshlets: render_context.global_buffers().buffer::<GPUMeshlet>(),
            bvh: render_context.global_buffers().buffer::<GPUBVHNode>(),
            vertices_positions: render_context.global_buffers().buffer::<GPUVertexPosition>(),
            shadow_rays,
            counters,
            data_buffer_1,
            binding_data: BindingData::new(render_context, COMPUTE_PATHTRACING_SHADOW_NAME),
            render_targets: [INVALID_UID; 4],
        }
    }
    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("pathtracing_shadow_pass::init");

         if self.render_targets[0].is_nil() {
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
                &mut *self.meshes.write().unwrap(),
                Some("Meshes"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 1,
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
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.bvh.write().unwrap(),
                Some("BVH"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 3,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.vertices_positions.write().unwrap(),
                Some("VerticesPositions"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 4,
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
                &mut *self.counters.write().unwrap(),
                Some("Counters"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 1,
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
                    binding_index: 6,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::ReadWrite,
                    ..Default::default()
                },
            );

        // Add Render Targets (Storage Textures)
        for (i, texture_id) in self.render_targets.iter().enumerate() {
            self.binding_data.add_texture(
                texture_id,
                (BindingFlags::ReadWrite | BindingFlags::StorageBinding).into(),
                BindingInfo {
                    group_index: 1,
                    binding_index: 2 + i as u32,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
            );
        }

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

impl ComputePathTracingShadowPass {
    pub fn set_render_targets(&mut self, diffuse: &TextureId, specular: &TextureId, shadow: &TextureId, ao: &TextureId) {
        self.render_targets = [*diffuse, *specular, *shadow, *ao];
    }
}
