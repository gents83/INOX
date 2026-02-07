use inox_bvh::{create_linearized_bvh, BVHTree, GPUBVHNode, AABB};
use inox_math::VecBase;
use inox_render::{
    BindingData, CommandBuffer, ConstantDataRw, GPUBuffer, GPUInstance, GPULight, GPUMaterial,
    GPUMesh, GPUMeshlet, GPUTransform, GPUVector, GPUVertexAttributes, GPUVertexIndices,
    GPUVertexPosition, INSTANCE_DATA_ID, Pass, RenderContext, RenderContextRc, TextureView,
};

use inox_core::ContextRc;
use inox_resources::{DataTypeResource, Resource, ResourceTrait};
use inox_uid::generate_random_uid;

use crate::{GEOMETRY_BUFFER_UID, SCENE_BUFFER_UID};

pub const COMPUTE_PATHTRACING_MERGE_NAME: &str = "ComputePathTracingMergePass";

#[derive(Default)]
pub struct ComputePathTracingMergePass {
    constant_data: ConstantDataRw,
    indices: GPUBuffer<GPUVertexIndices>,
    vertices_positions: GPUBuffer<GPUVertexPosition>,
    vertices_attributes: GPUBuffer<GPUVertexAttributes>,
    instances: GPUVector<GPUInstance>,
    transforms: GPUVector<GPUTransform>,
    meshes: GPUBuffer<GPUMesh>,
    meshlets: GPUBuffer<GPUMeshlet>,
    materials: GPUBuffer<GPUMaterial>,
    lights: GPUBuffer<GPULight>,
    geometry_buffer: GPUVector<u32>,
    scene_buffer: GPUVector<u32>,
}

unsafe impl Send for ComputePathTracingMergePass {}
unsafe impl Sync for ComputePathTracingMergePass {}

impl Pass for ComputePathTracingMergePass {
    fn name(&self) -> &str {
        COMPUTE_PATHTRACING_MERGE_NAME
    }
    fn static_name() -> &'static str {
        COMPUTE_PATHTRACING_MERGE_NAME
    }
    fn is_active(&self, _render_context: &RenderContext) -> bool {
        true
    }
    fn create(_context: &ContextRc, render_context: &RenderContextRc) -> Self
    where
        Self: Sized,
    {
        let geometry_buffer = render_context
            .global_buffers()
            .vector_with_id::<u32>(GEOMETRY_BUFFER_UID);
        let scene_buffer = render_context
            .global_buffers()
            .vector_with_id::<u32>(SCENE_BUFFER_UID);

        Self {
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
            lights: render_context.global_buffers().buffer::<GPULight>(),
            geometry_buffer,
            scene_buffer,
        }
    }
    fn init(&mut self, _render_context: &RenderContext) {}

    fn update(
        &mut self,
        render_context: &RenderContext,
        _surface_view: &TextureView,
        command_buffer: &mut CommandBuffer,
    ) {
        inox_profiler::scoped_profile!("pathtracing_merge_pass::update");

        let indices_size = self.indices.read().unwrap().data_size();
        let positions_size = self.vertices_positions.read().unwrap().data_size();
        let attributes_size = self.vertices_attributes.read().unwrap().data_size();

        let meshes_size = self.meshes.read().unwrap().data_size();
        let meshlets_size = self.meshlets.read().unwrap().data_size();
        let instances_size = self.instances.read().unwrap().data_size();
        let transforms_size = self.transforms.read().unwrap().data_size();
        let materials_size = self.materials.read().unwrap().data_size();
        let lights_size = self.lights.read().unwrap().data_size();

        // 1. Collect Instance AABBs and Build TLAS
        let mut instance_aabbs = Vec::new();
        {
            let instances = self.instances.read().unwrap();
            let transforms = self.transforms.read().unwrap();

            instances.data().iter().enumerate().for_each(|(i, instance)| {
                if instance.transform_id as usize >= transforms.data().len() {
                    return;
                }
                let transform = &transforms.data()[instance.transform_id as usize];

                let min_x = transform.bb_min_scale_y[0];
                let min_y = transform.bb_min_scale_y[1];
                let min_z = transform.bb_min_scale_y[2];
                let max_x = transform.bb_max_scale_z[0];
                let max_y = transform.bb_max_scale_z[1];
                let max_z = transform.bb_max_scale_z[2];

                let corners = [
                    inox_math::Vector3::new(min_x, min_y, min_z),
                    inox_math::Vector3::new(max_x, min_y, min_z),
                    inox_math::Vector3::new(min_x, max_y, min_z),
                    inox_math::Vector3::new(min_x, min_y, max_z),
                    inox_math::Vector3::new(max_x, max_y, min_z),
                    inox_math::Vector3::new(max_x, min_y, max_z),
                    inox_math::Vector3::new(min_x, max_y, max_z),
                    inox_math::Vector3::new(max_x, max_y, max_z),
                ];

                let position = inox_math::Vector3::new(transform.position_scale_x[0], transform.position_scale_x[1], transform.position_scale_x[2]);
                let scale = transform.position_scale_x[3];
                let orientation = inox_math::Vector4::from(transform.orientation);
                let q = inox_math::Quaternion::new(orientation.w, orientation.x, orientation.y, orientation.z);

                let matrix = inox_math::Matrix4::from_translation_orientation_scale(position, q, inox_math::Vector3::new(scale, scale, scale));

                let mut world_min = inox_math::Vector3::new(f32::MAX, f32::MAX, f32::MAX);
                let mut world_max = inox_math::Vector3::new(-f32::MAX, -f32::MAX, -f32::MAX);

                for c in corners {
                    let wc = matrix * inox_math::Vector4::new(c.x, c.y, c.z, 1.0);
                    let wcv = inox_math::Vector3::new(wc.x, wc.y, wc.z);
                    world_min = world_min.min(wcv);
                    world_max = world_max.max(wcv);
                }

                instance_aabbs.push(AABB::create(world_min, world_max, i as i32));
            });
        }

        let tlas_bvh = BVHTree::new(&instance_aabbs);
        let linear_tlas = create_linearized_bvh(&tlas_bvh);
        let tlas_size_bytes = linear_tlas.len() * std::mem::size_of::<GPUBVHNode>();

        // Build BLAS
        let mut bvh_data: Vec<u32> = Vec::new();
        let tlas_u32: &[u32] = unsafe { std::slice::from_raw_parts(linear_tlas.as_ptr() as *const u32, linear_tlas.len() * 8) };
        bvh_data.extend_from_slice(tlas_u32);

        let mut meshes_data = self.meshes.read().unwrap().data().to_vec();
        let meshlets_data = self.meshlets.read().unwrap().data();

        for i in 0..meshes_data.len() {
            let start = meshes_data[i].lods_meshlets_offset[0] as usize;
            let end = if i + 1 < meshes_data.len() {
                meshes_data[i+1].lods_meshlets_offset[0] as usize
            } else {
                meshlets_data.len()
            };

            if start >= end { continue; }

            let mut blas_aabbs = Vec::new();
            for m_idx in start..end {
                let m = &meshlets_data[m_idx];
                blas_aabbs.push(AABB::create(m.aabb_min.into(), m.aabb_max.into(), m_idx as i32));
            }

            let blas_bvh = BVHTree::new(&blas_aabbs);
            let linear_blas = create_linearized_bvh(&blas_bvh);

            let blas_offset_nodes = (bvh_data.len() / 8) as u32;
            let blas_u32: &[u32] = unsafe { std::slice::from_raw_parts(linear_blas.as_ptr() as *const u32, linear_blas.len() * 8) };
            bvh_data.extend_from_slice(blas_u32);
            meshes_data[i].blas_index = blas_offset_nodes;
        }

        // Calculate Offsets
        let meshes_offset = 0;
        let meshlets_offset = meshes_size;
        let instances_offset = meshlets_offset + meshlets_size;
        let transforms_offset = instances_offset + instances_size;
        let materials_offset = transforms_offset + transforms_size;
        let lights_offset = materials_offset + materials_size;
        let bvh_offset = lights_offset + lights_size;

        let scene_total_size = bvh_offset + bvh_data.len() * 4;
        let geometry_total_size = indices_size + positions_size + attributes_size;

        // Resize Buffers
        {
            let mut geometry_buffer = self.geometry_buffer.write().unwrap();
            if geometry_buffer.data_size() < geometry_total_size {
                let size_u32 = (geometry_total_size + 3) / 4;
                geometry_buffer.resize(size_u32 as usize, 0);
            }
        }
        {
            let mut scene_buffer = self.scene_buffer.write().unwrap();
            if scene_buffer.data_size() < scene_total_size {
                let size_u32 = (scene_total_size + 3) / 4;
                scene_buffer.resize(size_u32 as usize, 0);
            }
        }

        // Copy Geometry
        let indices_offset = 0;
        let positions_offset = indices_size;
        let attributes_offset = positions_offset + positions_size;

        if indices_size > 0 {
            let src = self.indices.read().unwrap();
            let dst = self.geometry_buffer.read().unwrap();
            command_buffer.copy_buffer_to_buffer(src.gpu_buffer().unwrap(), 0, dst.gpu_buffer().unwrap(), indices_offset, indices_size);
        }
        if positions_size > 0 {
            let src = self.vertices_positions.read().unwrap();
            let dst = self.geometry_buffer.read().unwrap();
            command_buffer.copy_buffer_to_buffer(src.gpu_buffer().unwrap(), 0, dst.gpu_buffer().unwrap(), positions_offset, positions_size);
        }
        if attributes_size > 0 {
            let src = self.vertices_attributes.read().unwrap();
            let dst = self.geometry_buffer.read().unwrap();
            command_buffer.copy_buffer_to_buffer(src.gpu_buffer().unwrap(), 0, dst.gpu_buffer().unwrap(), attributes_offset, attributes_size);
        }

        // Copy Scene
        let scene_buffer_res = self.scene_buffer.read().unwrap().gpu_buffer().unwrap().clone();

        // 1. Upload Meshes (CPU -> GPU)
        render_context.webgpu.queue.write_buffer(&scene_buffer_res, meshes_offset, bytemuck::cast_slice(&meshes_data));

        // 2. Copy Meshlets
        if meshlets_size > 0 {
            command_buffer.copy_buffer_to_buffer(self.meshlets.read().unwrap().gpu_buffer().unwrap(), 0, &scene_buffer_res, meshlets_offset, meshlets_size);
        }

        // 3. Copy Instances
        if instances_size > 0 {
            command_buffer.copy_buffer_to_buffer(self.instances.read().unwrap().gpu_buffer().unwrap(), 0, &scene_buffer_res, instances_offset, instances_size);
        }

        // 4. Copy Transforms
        if transforms_size > 0 {
            command_buffer.copy_buffer_to_buffer(self.transforms.read().unwrap().gpu_buffer().unwrap(), 0, &scene_buffer_res, transforms_offset, transforms_size);
        }

        // 5. Copy Materials
        if materials_size > 0 {
            command_buffer.copy_buffer_to_buffer(self.materials.read().unwrap().gpu_buffer().unwrap(), 0, &scene_buffer_res, materials_offset, materials_size);
        }

        // 6. Copy Lights
        if lights_size > 0 {
            command_buffer.copy_buffer_to_buffer(self.lights.read().unwrap().gpu_buffer().unwrap(), 0, &scene_buffer_res, lights_offset, lights_size);
        }

        // 7. Upload BVH (CPU -> GPU)
        render_context.webgpu.queue.write_buffer(&scene_buffer_res, bvh_offset, bytemuck::cast_slice(&bvh_data));

        // Update Offsets
        self.constant_data.write().unwrap()
            .set_geometry_buffer_offsets(render_context, (indices_offset / 4) as u32, (positions_offset / 4) as u32, (attributes_offset / 4) as u32)
            .set_scene_buffer_offsets(render_context, (meshes_offset / 4) as u32, (meshlets_offset / 4) as u32, (instances_offset / 4) as u32, (transforms_offset / 4) as u32)
            .set_materials_lights_offsets(render_context, (materials_offset / 4) as u32, (lights_offset / 4) as u32)
            .set_bvh_offset(render_context, (bvh_offset / 4) as u32);
    }
}
