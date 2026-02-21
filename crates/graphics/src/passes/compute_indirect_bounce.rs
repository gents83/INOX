use std::path::PathBuf;

use inox_bvh::GPUBVHNode;
use inox_render::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData,
    ConstantDataRw, GPUBuffer, GPUInstance, GPULight, GPUMaterial, GPUMesh, GPUMeshlet, GPUTexture,
    GPUTransform, GPUVector, GPUVertexAttributes, GPUVertexIndices, GPUVertexPosition, Pass,
    RenderContext, RenderContextRc, SamplerType, ShaderStage, TextureView, INSTANCE_DATA_ID,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;

use crate::{RadiancePackedData, RayPackedData, ThroughputPackedData};

pub const COMPUTE_RAY_TRAVERSAL_PIPELINE: &str = "pipelines/ComputeRayTraversal.compute_pipeline";
pub const COMPUTE_BOUNCE_SHADE_PIPELINE: &str = "pipelines/ComputeBounceShade.compute_pipeline";
pub const COMPUTE_INDIRECT_BOUNCE_NAME: &str = "ComputeIndirectBouncePass";

pub struct ComputeIndirectBouncePass {
    // Traversal pipeline + bindings
    traversal_pass: Resource<ComputePass>,
    traversal_binding_data: BindingData,
    // Shade pipeline + bindings
    shade_pass: Resource<ComputePass>,
    shade_binding_data: BindingData,
    // Shared resources
    constant_data: ConstantDataRw,
    indices: GPUBuffer<GPUVertexIndices>,
    vertices_positions: GPUBuffer<GPUVertexPosition>,
    vertices_attributes: GPUBuffer<GPUVertexAttributes>,
    instances: GPUVector<GPUInstance>,
    transforms: GPUVector<GPUTransform>,
    meshes: GPUBuffer<GPUMesh>,
    meshlets: GPUBuffer<GPUMeshlet>,
    bvh: GPUBuffer<GPUBVHNode>,
    materials: GPUBuffer<GPUMaterial>,
    textures: GPUBuffer<GPUTexture>,
    lights: GPUBuffer<GPULight>,
    data_buffer_0: GPUVector<RayPackedData>,
    data_buffer_1: GPUVector<RadiancePackedData>,
    data_buffer_2: GPUVector<ThroughputPackedData>,
    dimensions: (u32, u32),
}
unsafe impl Send for ComputeIndirectBouncePass {}
unsafe impl Sync for ComputeIndirectBouncePass {}

impl Pass for ComputeIndirectBouncePass {
    fn name(&self) -> &str {
        COMPUTE_INDIRECT_BOUNCE_NAME
    }
    fn static_name() -> &'static str {
        COMPUTE_INDIRECT_BOUNCE_NAME
    }
    fn is_active(&self, _render_context: &RenderContext) -> bool {
        true
    }
    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self {
        let traversal_data = ComputePassData {
            name: "ComputeRayTraversalPass".to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_RAY_TRAVERSAL_PIPELINE)],
        };
        let shade_data = ComputePassData {
            name: "ComputeBounceShadePass".to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_BOUNCE_SHADE_PIPELINE)],
        };

        Self {
            traversal_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &traversal_data,
                None,
            ),
            traversal_binding_data: BindingData::new(render_context, "ComputeRayTraversalPass"),
            shade_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &shade_data,
                None,
            ),
            shade_binding_data: BindingData::new(render_context, "ComputeBounceShadePass"),
            constant_data: render_context.global_buffers().constant_data.clone(),
            indices: render_context.global_buffers().buffer::<GPUVertexIndices>(),
            vertices_positions: render_context
                .global_buffers()
                .buffer::<GPUVertexPosition>(),
            vertices_attributes: render_context
                .global_buffers()
                .buffer::<GPUVertexAttributes>(),
            transforms: render_context.global_buffers().vector::<GPUTransform>(),
            instances: render_context
                .global_buffers()
                .vector_with_id::<GPUInstance>(INSTANCE_DATA_ID),
            meshes: render_context.global_buffers().buffer::<GPUMesh>(),
            meshlets: render_context.global_buffers().buffer::<GPUMeshlet>(),
            bvh: render_context.global_buffers().buffer::<GPUBVHNode>(),
            materials: render_context.global_buffers().buffer::<GPUMaterial>(),
            textures: render_context.global_buffers().buffer::<GPUTexture>(),
            lights: render_context.global_buffers().buffer::<GPULight>(),
            data_buffer_0: render_context.global_buffers().vector::<RayPackedData>(),
            data_buffer_1: render_context
                .global_buffers()
                .vector::<RadiancePackedData>(),
            data_buffer_2: render_context
                .global_buffers()
                .vector::<ThroughputPackedData>(),
            dimensions: (0, 0),
        }
    }

    fn init(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("indirect_bounce_pass::init");

        if self.meshlets.read().unwrap().is_empty() {
            return;
        }

        // === Traversal pass bindings ===
        // Matches compute_ray_traversal.wgsl layout

        // Group 0: geometry + BVH
        self.traversal_binding_data
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
                &mut *self.instances.write().unwrap(),
                Some("Instances"),
                BindingInfo {
                    group_index: 0,
                    binding_index: 3,
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
                    binding_index: 4,
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
                    binding_index: 5,
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
                    binding_index: 6,
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
                    binding_index: 7,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            );

        // Group 1: data buffers
        self.traversal_binding_data
            .add_buffer(
                &mut *self.data_buffer_0.write().unwrap(),
                Some("DataBuffer_0"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::ReadWrite | BindingFlags::Storage,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.data_buffer_1.write().unwrap(),
                Some("DataBuffer_1"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::ReadWrite | BindingFlags::Storage,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.data_buffer_2.write().unwrap(),
                Some("DataBuffer_2"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::ReadWrite | BindingFlags::Storage,
                    ..Default::default()
                },
            );

        {
            let mut pass = self.traversal_pass.get_mut();
            pass.init(render_context, &mut self.traversal_binding_data, None);
        }

        // === Shade pass bindings ===
        // Matches compute_bounce_shade.wgsl layout

        // Group 0: geometry
        self.shade_binding_data
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
            );

        // Group 1: materials, textures, lights, BVH, data buffers
        self.shade_binding_data
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
                &mut *self.bvh.write().unwrap(),
                Some("BVH"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 3,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.data_buffer_0.write().unwrap(),
                Some("DataBuffer_0"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 4,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::ReadWrite | BindingFlags::Storage,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.data_buffer_1.write().unwrap(),
                Some("DataBuffer_1"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 5,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::ReadWrite | BindingFlags::Storage,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.data_buffer_2.write().unwrap(),
                Some("DataBuffer_2"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 6,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::ReadWrite | BindingFlags::Storage,
                    ..Default::default()
                },
            );

        // Group 2: texture sampling
        self.shade_binding_data
            .add_default_sampler(
                BindingInfo {
                    group_index: 2,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    ..Default::default()
                },
                SamplerType::Unfiltered,
            )
            .add_material_textures(BindingInfo {
                group_index: 2,
                binding_index: 1,
                stage: ShaderStage::Compute,
                ..Default::default()
            });

        {
            let mut pass = self.shade_pass.get_mut();
            pass.init(render_context, &mut self.shade_binding_data, None);
        }
    }

    fn update(
        &mut self,
        render_context: &RenderContext,
        _surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
        if self.meshlets.read().unwrap().is_empty() {
            return;
        }

        inox_profiler::scoped_profile!("indirect_bounce_pass::update");

        // Runtime bounce count from CPU
        let num_bounces = self.constant_data.read().unwrap().num_bounces();
        if num_bounces == 0 {
            return;
        }

        let x = self.dimensions.0.div_ceil(8);
        let y = self.dimensions.1.div_ceil(8);

        for _bounce in 0..num_bounces {
            // 1. Trace rays through BVH
            {
                let pass = self.traversal_pass.get();
                pass.dispatch(
                    render_context,
                    &mut self.traversal_binding_data,
                    command_buffer,
                    x,
                    y,
                    1,
                );
            }

            // 2. Shade hits, accumulate radiance, generate next bounce ray
            {
                let pass = self.shade_pass.get();
                pass.dispatch(
                    render_context,
                    &mut self.shade_binding_data,
                    command_buffer,
                    x,
                    y,
                    1,
                );
            }
        }
    }
}

impl ComputeIndirectBouncePass {
    pub fn set_dimensions(&mut self, dimensions: (u32, u32)) -> &mut Self {
        self.dimensions = dimensions;
        self
    }
}
