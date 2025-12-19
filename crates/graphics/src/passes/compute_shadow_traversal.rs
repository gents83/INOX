use std::path::PathBuf;

use inox_bvh::GPUBVHNode;
use inox_render::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData,
    ConstantDataRw, GPUBuffer, GPUInstance, GPUMesh, GPUMeshlet, GPUTransform, GPUVector,
    GPUVertexAttributes, GPUVertexIndices, GPUVertexPosition, Pass, RenderContext, RenderContextRc,
    ShaderStage, TextureView, INSTANCE_DATA_ID,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;

use crate::{IntersectionPackedData, RayPackedData};

pub const COMPUTE_SHADOW_TRAVERSAL_PIPELINE: &str =
    "pipelines/ComputeShadowTraversal.compute_pipeline";
pub const COMPUTE_SHADOW_TRAVERSAL_NAME: &str = "ComputeShadowTraversalPass";

pub struct ComputeShadowTraversalPass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    shadow_rays: GPUVector<RayPackedData>,
    shadow_intersections: GPUVector<IntersectionPackedData>,
    indices: GPUBuffer<GPUVertexIndices>,
    vertices_position: GPUBuffer<GPUVertexPosition>,
    vertices_attributes: GPUBuffer<GPUVertexAttributes>,
    instances: GPUVector<GPUInstance>,
    transforms: GPUVector<GPUTransform>,
    meshes: GPUBuffer<GPUMesh>,
    meshlets: GPUBuffer<GPUMeshlet>,
    bvh: GPUVector<GPUBVHNode>,
}

unsafe impl Send for ComputeShadowTraversalPass {}
unsafe impl Sync for ComputeShadowTraversalPass {}

impl Pass for ComputeShadowTraversalPass {
    fn name(&self) -> &str {
        COMPUTE_SHADOW_TRAVERSAL_NAME
    }

    fn static_name() -> &'static str {
        COMPUTE_SHADOW_TRAVERSAL_NAME
    }

    fn is_active(&self, _render_context: &RenderContext) -> bool {
        true
    }

    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self
    where
        Self: Sized,
    {
        let data = ComputePassData {
            name: COMPUTE_SHADOW_TRAVERSAL_NAME.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_SHADOW_TRAVERSAL_PIPELINE)],
        };

        let shadow_rays = render_context
            .global_buffers()
            .vector_with_id::<RayPackedData>(crate::SHADOW_RAYS_ID);
        let shadow_intersections = render_context
            .global_buffers()
            .vector_with_id::<IntersectionPackedData>(crate::SHADOW_INTERSECTIONS_ID);
        let indices = render_context.global_buffers().buffer::<GPUVertexIndices>();
        let vertices_position = render_context
            .global_buffers()
            .buffer::<GPUVertexPosition>();
        let vertices_attributes = render_context
            .global_buffers()
            .buffer::<GPUVertexAttributes>();
        let instances = render_context
            .global_buffers()
            .vector_with_id::<GPUInstance>(INSTANCE_DATA_ID);
        let transforms = render_context.global_buffers().vector::<GPUTransform>();
        let meshes = render_context.global_buffers().buffer::<GPUMesh>();
        let meshlets = render_context.global_buffers().buffer::<GPUMeshlet>();
        let bvh = render_context.global_buffers().vector::<GPUBVHNode>();

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
            shadow_intersections,
            indices,
            vertices_position,
            vertices_attributes,
            instances,
            transforms,
            meshes,
            meshlets,
            bvh,
            binding_data: BindingData::new(render_context, COMPUTE_SHADOW_TRAVERSAL_NAME),
        }
    }

    fn init(&mut self, render_context: &RenderContext) {
        if self.instances.read().unwrap().is_empty() || self.meshlets.read().unwrap().is_empty() {
            return;
        }

        if self.shadow_rays.read().unwrap().is_empty()
            || self.shadow_intersections.read().unwrap().is_empty()
        {
            return;
        }

        // Don't initialize if BVH is empty - wgpu will reject empty buffer bindings
        if self.bvh.read().unwrap().is_empty() {
            return;
        }

        // Group 0: Constant data
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

        // Group 1: Geometry
        self.binding_data
            .add_buffer(
                &mut *self.indices.write().unwrap(),
                Some("indices"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.vertices_position.write().unwrap(),
                Some("vertices_positions"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.vertices_attributes.write().unwrap(),
                Some("vertices_attributes"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 2,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.instances.write().unwrap(),
                Some("instances"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 3,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.transforms.write().unwrap(),
                Some("transforms"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 4,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.meshes.write().unwrap(),
                Some("meshes"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 5,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.meshlets.write().unwrap(),
                Some("meshlets"),
                BindingInfo {
                    group_index: 1,
                    binding_index: 6,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            );

        // Group 2: BVH
        self.binding_data.add_buffer(
            &mut *self.bvh.write().unwrap(),
            Some("bvh"),
            BindingInfo {
                group_index: 2,
                binding_index: 0,
                stage: ShaderStage::Compute,
                flags: BindingFlags::Storage | BindingFlags::Read,
                ..Default::default()
            },
        );

        // Group 3: Ray data
        self.binding_data
            .add_buffer(
                &mut *self.shadow_rays.write().unwrap(),
                Some("shadow_rays"),
                BindingInfo {
                    group_index: 3,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.shadow_intersections.write().unwrap(),
                Some("shadow_intersections"),
                BindingInfo {
                    group_index: 3,
                    binding_index: 1,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::ReadWrite | BindingFlags::Storage,
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
        let ray_count = self.shadow_rays.read().unwrap().len() as u32;
        if ray_count == 0 {
            return;
        }

        let workgroup_size = 64;
        let dispatch_x = ray_count.div_ceil(workgroup_size);

        self.compute_pass.get().dispatch(
            render_context,
            &mut self.binding_data,
            command_buffer,
            dispatch_x,
            1,
            1,
        );
    }
}

impl ComputeShadowTraversalPass {
    pub fn shadow_intersections(&self) -> &GPUVector<IntersectionPackedData> {
        &self.shadow_intersections
    }
}
