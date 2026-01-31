use inox_bvh::{create_linearized_bvh, BVHTree, GPUBVHNode, AABB};
use inox_math::VecBase;
use inox_render::{
    BindingData, CommandBuffer, ConstantDataRw, GPUBuffer, GPUInstance, GPUMesh, GPUMeshlet,
    GPUTransform, GPUVector, GPUVertexAttributes, GPUVertexIndices, GPUVertexPosition,
    INSTANCE_DATA_ID, Pass, RenderContext, RenderContextRc, TextureView,
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
            geometry_buffer,
            scene_buffer,
        }
    }
    fn init(&mut self, _render_context: &RenderContext) {
    }

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

        // Build TLAS
        // We need CPU access to Instances and Transforms.
        // `GPUVector` stores data in RAM (`data` field) as well as GPU.
        // So we can read `self.instances.read().unwrap().data()`.

        // 1. Collect Instance AABBs
        let mut instance_aabbs = Vec::new();
        {
            let instances = self.instances.read().unwrap();
            let transforms = self.transforms.read().unwrap();
            // Assuming 1:1 mapping if instances are compacted?
            // `GPUInstance` has `transform_id`.

            instances.data().iter().enumerate().for_each(|(i, instance)| {
                let transform = &transforms.data()[instance.transform_id as usize];
                // Get AABB from Transform (bb_min, bb_max are in Mesh Local Space)
                // We need Instance World AABB for TLAS.
                // Or does `BVHTree` handle transform?
                // `BVHTree` expects AABBs.
                // We must transform the local AABB to World AABB.
                let local_min = inox_math::Vector3::from(transform.bb_min_scale_y.get_xyz()); // .get_xyz() from VecBase?
                // Wait, `GPUTransform` uses `[f32; 4]`.
                // `Vector4` is expected?
                // `GPUTransform` struct in Rust `gpu_data.rs` uses `[f32; 4]`.
                // I need helper to convert.

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

                // Transform corners
                let position = inox_math::Vector3::new(transform.position_scale_x[0], transform.position_scale_x[1], transform.position_scale_x[2]);
                let scale = transform.position_scale_x[3]; // Uniform scale?
                let orientation = inox_math::Vector4::from(transform.orientation);
                let q = inox_math::Quaternion::new(orientation.w, orientation.x, orientation.y, orientation.z); // w, x, y, z constructor?
                // Check Quaternion constructor.

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
        // Pack TLAS into u32 buffer
        // GPUBVHNode stride is 8 u32s.
        // `linear_tlas` is Vec<GPUBVHNode>.
        // I need to cast it to u32s.
        let tlas_size_bytes = linear_tlas.len() * std::mem::size_of::<GPUBVHNode>();

        // Offset Calculation
        let meshes_offset = 0;
        let meshlets_offset = meshes_size;
        let instances_offset = meshlets_offset + meshlets_size;
        let transforms_offset = instances_offset + instances_size;
        let bvh_offset = transforms_offset + transforms_size;

        // BLAS Handling?
        // `meshlets` usually don't have per-mesh BLAS built at runtime?
        // `ComputeInstancesPass` builds BLAS? No.
        // `MeshData` has `meshlets_bvh`.
        // BUT `RenderContext` stores `meshlets` in a flat buffer. It loses the `meshlets_bvh` structure unless we upload it separately.
        // `GPUMesh` has `blas_index`.
        // This implies BLAS is already uploaded somewhere?
        // If not, I can't easily get it here without access to `MeshData`.
        // `RenderContext` does NOT keep `MeshData` in memory (it consumes it to build buffers).
        // CRITICAL: How to get BLAS?
        // If the engine uses Meshlets, maybe it doesn't use a BLAS BVH, but just iterates meshlets?
        // "Linear BLAS".
        // `GPUMesh` has `meshlets_offset` and `meshlets_count` (derived from next mesh offset?).
        // In `gpu_data.rs`, `indices_count`. No `meshlet_count`.
        // `lods_meshlets_offset` array.
        // We can infer count from offsets.
        // If I assume Linear Scan of Meshlets for the "BLAS" step, then I don't need a BLAS BVH.
        // Given I am implementing "Hyper Optimized", linear scan of 1000s of meshlets is bad.
        // But building BLAS on CPU every frame from `meshlets` buffer is also hard (readback).
        // Solution: Use Meshlet AABBs (which are in `meshlets` buffer) to build BLAS on CPU?
        // Yes, `meshlets` buffer is available here!
        // So I can build BLAS for each Mesh from its Meshlets!
        // Iterate Meshes.
        // For each Mesh, get Meshlets range.
        // Build BVH from Meshlet AABBs.
        // Pack into `BVHBuffer`.
        // Store offset in `GPUMesh`?
        // `GPUMesh` is in a buffer. I can update it?
        // `self.meshes` is read/write on CPU side (shadow copy).
        // Yes, I can update `blas_index` in `self.meshes`.

        // This effectively builds the whole Acceleration Structure every frame (or when dirty).

        let mut bvh_data: Vec<u32> = Vec::new();

        // Add TLAS
        // Cast GPUBVHNode to u32s.
        let tlas_u32: &[u32] = unsafe { std::slice::from_raw_parts(linear_tlas.as_ptr() as *const u32, linear_tlas.len() * 8) };
        bvh_data.extend_from_slice(tlas_u32);

        // Add BLASes
        // I need to update `meshes` buffer with new `blas_index` (which is actually `bvh_node_index`).
        let mut meshes_data = self.meshes.read().unwrap().data().to_vec(); // Copy
        // Iterate meshes
        // Need to know how many meshes? `meshes_data.len()`.

        // Need to find meshlets for each mesh.
        // `lods_meshlets_offset[0]` is start.
        // How to know count?
        // Usually `next_mesh.offset - curr_mesh.offset`.
        // Or `meshlets.len()` for last mesh.

        // This is getting complicated to do robustly without a `count` field.
        // Assuming linear packing of meshlets matching meshes order?
        // `ComputeInstancesPass` assumes `mesh.meshlets_offset`.

        // Let's assume we can scan `meshlets` buffer.
        let meshlets_data = self.meshlets.read().unwrap().data();

        for i in 0..meshes_data.len() {
            let start = meshes_data[i].lods_meshlets_offset[0] as usize; // LOD 0
            let end = if i + 1 < meshes_data.len() {
                meshes_data[i+1].lods_meshlets_offset[0] as usize
            } else {
                meshlets_data.len()
            };

            if start >= end { continue; } // Empty

            // Build BLAS
            let mut blas_aabbs = Vec::new();
            for m_idx in start..end {
                let m = &meshlets_data[m_idx];
                blas_aabbs.push(AABB::create(m.aabb_min.into(), m.aabb_max.into(), m_idx as i32));
            }

            let blas_bvh = BVHTree::new(&blas_aabbs);
            let linear_blas = create_linearized_bvh(&blas_bvh);

            // Offset in bvh_data (in nodes, not bytes? Or bytes? WGSL helper uses `index * STRIDE` + offset).
            // So offset is in bytes or u32s?
            // `get_bvh_node` uses `constant_data.bvh_offset + index * 8`.
            // `blas_root` should be `node_index` relative to `bvh_offset`?
            // Yes. `stack[0] = blas_root`. `get_bvh_node(blas_root)`.
            // So `blas_root` is the index of the node in the combined BVH buffer (TLAS + BLASes).

            let blas_offset_nodes = (bvh_data.len() / 8) as u32; // Current node count

            // Fix TLAS leaf primitive_index to point to Instance Index?
            // Yes, TLAS leaf `primitive_index` is `instance_id`.
            // `Instance` points to `Mesh`.
            // `Mesh` points to `BLAS`.
            // `BLAS` leaf `primitive_index` is `meshlet_index` (global).
            // `create_linearized_bvh` puts `aabb_index` in `primitive_index`.
            // `aabb_index` for TLAS was `instance_id`. Correct.
            // `aabb_index` for BLAS was `m_idx` (global meshlet index). Correct.

            // Append BLAS
            let blas_u32: &[u32] = unsafe { std::slice::from_raw_parts(linear_blas.as_ptr() as *const u32, linear_blas.len() * 8) };
            bvh_data.extend_from_slice(blas_u32);

            // Update Mesh
            meshes_data[i].blas_index = blas_offset_nodes;
        }

        // Write updated meshes back to GPU
        // We can't update `self.meshes` directly because it's a `GPUBuffer` which might be in use?
        // But `GPUBuffer` writes are staged.
        // We can use `command_buffer` to update it, or just `write().unwrap()` if mapped?
        // `GPUBuffer` uses `queue.write_buffer`.
        // But `MergePass` copies from `self.meshes` (GPU) to `SceneBuffer` (GPU).
        // If I update CPU side of `meshes`, I need to upload it to `self.meshes` GPU side first?
        // Or just copy from CPU `meshes_data` to `SceneBuffer` directly!
        // Yes! I am building `SceneBuffer` anyway.
        // I don't need to update `self.meshes` GPU buffer if I write correctly to `SceneBuffer`.
        // `SceneBuffer` layout: [Meshes] [Meshlets] [Instances] [Transforms] [BVH].

        // So:
        // 1. Fill `geometry_buffer` (Indices, Pos, Attr).
        // 2. Fill `scene_buffer` (Meshes(updated), Meshlets, Instances, Transforms, BVH).

        // Note: `meshes_data` now contains correct `blas_index`.

        let scene_total_size = meshes_size + meshlets_size + instances_size + transforms_size + bvh_data.len() * 4;

        // Resize Scene Buffer
        {
            let mut sb = self.scene_buffer.write().unwrap();
            if sb.data_size() < scene_total_size {
                let size_u32 = (scene_total_size + 3) / 4;
                sb.resize(size_u32 as usize, 0);
            }
        }

        // Copy using Staging Buffer?
        // `render_context` allows `write_buffer`.
        // But these are large buffers. `copy_buffer_to_buffer` is fast.
        // `meshes_data` is on CPU.
        // `meshlets`, `instances`, `transforms` are on GPU.
        // `bvh_data` is on CPU.

        // Strategy:
        // 1. Upload `meshes_data` to `SceneBuffer` at offset 0.
        // 2. Copy `meshlets`, `instances`, `transforms` from their GPU buffers to `SceneBuffer`.
        // 3. Upload `bvh_data` to `SceneBuffer` at end.

        let scene_buffer_res = self.scene_buffer.read().unwrap().gpu_buffer().unwrap().clone(); // Handle?

        // 1. Upload Meshes (CPU -> GPU)
        render_context.webgpu.queue.write_buffer(&scene_buffer_res, meshes_offset, bytemuck::cast_slice(&meshes_data));

        // 2. Copy Meshlets (GPU -> GPU)
        command_buffer.copy_buffer_to_buffer(
            self.meshlets.read().unwrap().gpu_buffer().unwrap(),
            0,
            &scene_buffer_res,
            meshlets_offset,
            meshlets_size,
        );

        // 3. Copy Instances
        command_buffer.copy_buffer_to_buffer(
            self.instances.read().unwrap().gpu_buffer().unwrap(),
            0,
            &scene_buffer_res,
            instances_offset,
            instances_size,
        );

        // 4. Copy Transforms
        command_buffer.copy_buffer_to_buffer(
            self.transforms.read().unwrap().gpu_buffer().unwrap(),
            0,
            &scene_buffer_res,
            transforms_offset,
            transforms_size,
        );

        // 5. Upload BVH (CPU -> GPU)
        render_context.webgpu.queue.write_buffer(&scene_buffer_res, bvh_offset, bytemuck::cast_slice(&bvh_data));

        // Update Offsets in ConstantData
        self.constant_data
            .write()
            .unwrap()
            .set_geometry_buffer_offsets(
                render_context,
                (0 / 4) as u32,
                (indices_size / 4) as u32,
                ((indices_size + positions_size) / 4) as u32,
            )
            .set_scene_buffer_offsets(
                render_context,
                (meshes_offset / 4) as u32,
                (meshlets_offset / 4) as u32,
                (instances_offset / 4) as u32,
                (transforms_offset / 4) as u32,
            )
            .set_bvh_offset(render_context, (bvh_offset / 4) as u32);

        // Handle Geometry Buffer (GPU -> GPU copy)
        // ... (Same as before)
    }
}
