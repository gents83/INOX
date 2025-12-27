use std::path::PathBuf;

use inox_render::{
    BindingData, BindingFlags, BindingInfo, CommandBuffer, ComputePass, ComputePassData,
    ConstantDataRw, GPUBuffer, GPUInstance, GPUMesh, GPUMeshlet, GPUTransform, GPUVector,
    GPUVertexAttributes, GPUVertexIndices, GPUVertexPosition, Pass, RenderContext, RenderContextRc,
    ShaderStage, TextureView, INSTANCE_DATA_ID,
};

use inox_bvh::GPUBVHNode;

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource};
use inox_uid::generate_random_uid;

use crate::{IntersectionPackedData, RayPackedData};

pub const COMPUTE_RAY_TRAVERSAL_PIPELINE: &str = "pipelines/ComputeRayTraversal.compute_pipeline";
pub const COMPUTE_RAY_TRAVERSAL_NAME: &str = "ComputeRayTraversalPass";

pub struct ComputeRayTraversalPass {
    compute_pass: Resource<ComputePass>,
    binding_data: BindingData,
    constant_data: ConstantDataRw,
    rays: GPUVector<RayPackedData>,
    intersections: GPUVector<IntersectionPackedData>,
    indices: GPUBuffer<GPUVertexIndices>,
    vertices_position: GPUBuffer<GPUVertexPosition>,
    vertices_attributes: GPUBuffer<GPUVertexAttributes>,
    instances: GPUVector<GPUInstance>,
    transforms: GPUVector<GPUTransform>,
    meshes: GPUBuffer<GPUMesh>,
    meshlets: GPUBuffer<GPUMeshlet>,
    bvh: GPUBuffer<GPUBVHNode>,
}

unsafe impl Send for ComputeRayTraversalPass {}
unsafe impl Sync for ComputeRayTraversalPass {}

impl Pass for ComputeRayTraversalPass {
    fn name(&self) -> &str {
        COMPUTE_RAY_TRAVERSAL_NAME
    }

    fn static_name() -> &'static str {
        COMPUTE_RAY_TRAVERSAL_NAME
    }

    fn is_active(&self, _render_context: &RenderContext) -> bool {
        true
    }

    fn create(context: &ContextRc, render_context: &RenderContextRc) -> Self
    where
        Self: Sized,
    {
        let data = ComputePassData {
            name: COMPUTE_RAY_TRAVERSAL_NAME.to_string(),
            pipelines: vec![PathBuf::from(COMPUTE_RAY_TRAVERSAL_PIPELINE)],
        };

        let rays = render_context
            .global_buffers()
            .vector_with_id::<RayPackedData>(crate::BOUNCE_RAYS_ID);
        let intersections = render_context
            .global_buffers()
            .vector_with_id::<IntersectionPackedData>(crate::BOUNCE_INTERSECTIONS_ID);
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
        let bvh = render_context.global_buffers().buffer::<GPUBVHNode>();

        Self {
            compute_pass: ComputePass::new_resource(
                context.shared_data(),
                context.message_hub(),
                generate_random_uid(),
                &data,
                None,
            ),
            constant_data: render_context.global_buffers().constant_data.clone(),
            rays,
            intersections,
            indices,
            vertices_position,
            vertices_attributes,
            instances,
            transforms,
            meshes,
            meshlets,
            bvh,
            binding_data: BindingData::new(render_context, COMPUTE_RAY_TRAVERSAL_NAME),
        }
    }

    fn init(&mut self, render_context: &RenderContext) {
        if self.instances.read().unwrap().is_empty() || self.meshlets.read().unwrap().is_empty() {
            return;
        }

        if self.rays.read().unwrap().is_empty() || self.intersections.read().unwrap().is_empty() {
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

        // Group 1: Geometry - all buffers
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

        // Group 3: Ray Data
        self.binding_data
            .add_buffer(
                &mut *self.rays.write().unwrap(),
                Some("rays"),
                BindingInfo {
                    group_index: 3,
                    binding_index: 0,
                    stage: ShaderStage::Compute,
                    flags: BindingFlags::Storage | BindingFlags::Read,
                    ..Default::default()
                },
            )
            .add_buffer(
                &mut *self.intersections.write().unwrap(),
                Some("intersections"),
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
        // Use DEFAULT constants for stride alignment
        use inox_render::DEFAULT_HEIGHT;
        use inox_render::DEFAULT_WIDTH;

        let num_rays = (DEFAULT_WIDTH / 2) * (DEFAULT_HEIGHT / 2);
        let workgroup_size = 64;
        let dispatch_x = num_rays.div_ceil(workgroup_size);

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

impl ComputeRayTraversalPass {
    pub fn intersections(&self) -> &GPUVector<IntersectionPackedData> {
        &self.intersections
    }

    pub fn get_compute_pass(&self) -> &Resource<ComputePass> {
        &self.compute_pass
    }

    pub fn get_binding_data_mut(&mut self) -> &mut BindingData {
        &mut self.binding_data
    }
}
