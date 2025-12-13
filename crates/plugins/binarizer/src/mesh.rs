use std::mem::size_of;

use inox_math::{VecBase, VecBaseFloat, Vector2, Vector3, Vector4};
use inox_render::{MeshData, MeshletData, VertexAttributeLayout};
use inox_resources::to_slice;
use meshopt::DecodePosition;

const MESHLET_MAX_VERTICES: usize = 192;
const MESHLET_MAX_TRIANGLES: usize = 128;

#[derive(Debug, Clone, Copy)]
pub struct MeshVertex {
    pub pos: Vector4,
    pub color: Vector4,
    pub normal: Vector4,
    pub tangent: Vector4,
    pub uv_0: Vector2,
    pub uv_1: Vector2,
    pub uv_2: Vector2,
    pub uv_3: Vector2,
}

impl MeshVertex {
    pub fn is_same_of(&self, other: &Self) -> bool {
        self.pos.xyz() == other.pos.xyz()
            && self.color == other.color
            && self.normal.xyz() == other.normal.xyz()
            && self.tangent == other.tangent
            && self.uv_0 == other.uv_0
            && self.uv_1 == other.uv_1
            && self.uv_2 == other.uv_2
            && self.uv_3 == other.uv_3
    }
}

impl Default for MeshVertex {
    fn default() -> Self {
        Self {
            pos: Vector4::default_zero(),
            color: Vector4::default_zero(),
            normal: Vector4::default_zero(),
            tangent: Vector4::default_zero(),
            uv_0: Vector2::default_zero(),
            uv_1: Vector2::default_zero(),
            uv_2: Vector2::default_zero(),
            uv_3: Vector2::default_zero(),
        }
    }
}

impl meshopt::DecodePosition for MeshVertex {
    fn decode_position(&self) -> [f32; 3] {
        self.pos.xyz().into()
    }
}

pub fn optimize_mesh<T>(vertices: &[T], indices: &[u32]) -> (Vec<T>, Vec<u32>)
where
    T: Clone + Default,
{
    let vertices_bytes = to_slice(vertices);
    let vertex_stride = size_of::<T>();
    debug_assert!(
        vertex_stride.is_multiple_of(size_of::<f32>()),
        "Vertex size is not a multiple of f32 - meshopt will fail"
    );
    let vertex_data_adapter = meshopt::VertexDataAdapter::new(vertices_bytes, vertex_stride, 0);

    let mut new_indices = meshopt::optimize_vertex_cache(indices, vertices.len());
    let threshold = 1.01; // allow up to 1% worse ACMR to get more reordering opportunities for overdraw
    meshopt::optimize_overdraw_in_place(
        new_indices.as_mut_slice(),
        vertex_data_adapter.as_ref().unwrap(),
        threshold,
    );
    let new_vertices = meshopt::optimize_vertex_fetch(&mut new_indices, vertices);

    (new_vertices, new_indices)
}

pub fn create_mesh_data(
    vertex_layout: VertexAttributeLayout,
    vertices: &[MeshVertex],
    indices: &[u32],
) -> MeshData {
    let mut mesh_data = MeshData {
        vertex_layout: VertexAttributeLayout::HasPosition,
        aabb_max: Vector3::new(-f32::INFINITY, -f32::INFINITY, -f32::INFINITY),
        aabb_min: Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
        ..Default::default()
    };
    vertices.iter().for_each(|v| {
        mesh_data.aabb_max = mesh_data.aabb_max.max(v.pos.xyz());
        mesh_data.aabb_min = mesh_data.aabb_min.min(v.pos.xyz());
    });
    mesh_data.vertex_layout = vertex_layout;
    mesh_data.indices = indices.to_vec();
    mesh_data.vertex_positions.reserve(vertices.len());
    mesh_data
        .vertex_attributes
        .reserve(VertexAttributeLayout::all().stride_in_count() * vertices.len());
    vertices.iter().for_each(|v| {
        mesh_data.insert_position(v.pos.xyz());
        if vertex_layout.intersects(VertexAttributeLayout::HasColor) {
            mesh_data.insert_color(v.color);
        }
        if vertex_layout.intersects(VertexAttributeLayout::HasNormal) {
            mesh_data.insert_normal(v.normal.xyz());
        }
        if vertex_layout.intersects(VertexAttributeLayout::HasTangent) {
            mesh_data.insert_tangent(v.tangent);
        }
        if vertex_layout.intersects(VertexAttributeLayout::HasUV1) {
            mesh_data.insert_uv(v.uv_0);
        }
        if vertex_layout.intersects(VertexAttributeLayout::HasUV2) {
            mesh_data.insert_uv(v.uv_1);
        }
        if vertex_layout.intersects(VertexAttributeLayout::HasUV3) {
            mesh_data.insert_uv(v.uv_2);
        }
        if vertex_layout.intersects(VertexAttributeLayout::HasUV4) {
            mesh_data.insert_uv(v.uv_3);
        }
    });
    mesh_data
}

pub fn compute_meshlets<T>(
    vertices: &[T],
    indices: &[u32],
    starting_offset: u32,
) -> (Vec<MeshletData>, Vec<u32>)
where
    T: DecodePosition,
{
    let vertices_bytes = to_slice(vertices);
    let vertex_stride = size_of::<T>();
    debug_assert!(
        vertex_stride.is_multiple_of(size_of::<f32>()),
        "Vertex size is not a multiple of f32 - meshopt will fail"
    );
    let vertex_data_adapter = meshopt::VertexDataAdapter::new(vertices_bytes, vertex_stride, 0);

    let mut new_meshlets = Vec::new();
    let cone_weight = 0.5;
    let meshlets = meshopt::build_meshlets(
        indices,
        vertex_data_adapter.as_ref().unwrap(),
        MESHLET_MAX_VERTICES,
        MESHLET_MAX_TRIANGLES,
        cone_weight,
    );
    debug_assert!(!meshlets.meshlets.is_empty());

    let mut new_indices = Vec::new();
    for m in meshlets.iter() {
        let mut meshlet_indices = Vec::new();
        debug_assert!(m.triangles.len() % 3 == 0);
        let mut aabb_max = Vector3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
        let mut aabb_min = Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        m.triangles.iter().for_each(|&i| {
            let index = m.vertices[i as usize] as usize;
            meshlet_indices.push(index as u32);
            let pos = vertices[index].decode_position().into();
            aabb_min = aabb_min.min(pos);
            aabb_max = aabb_max.max(pos);
        });
        let bounding_data = meshopt::compute_cluster_bounds(
            &meshlet_indices,
            vertex_data_adapter.as_ref().unwrap(),
        );
        let index_offset = new_indices.len();
        new_indices.append(&mut meshlet_indices);
        debug_assert!(new_indices.len() % 3 == 0);
        new_meshlets.push(MeshletData {
            indices_offset: starting_offset + index_offset as u32,
            indices_count: m.triangles.len() as _,
            aabb_min,
            aabb_max,
            error: 0.0,
            bounding_sphere: Vector4::new(
                bounding_data.center[0],
                bounding_data.center[1],
                bounding_data.center[2],
                bounding_data.radius,
            ),
            parent_error: f32::INFINITY,
            parent_bounding_sphere: Vector4::default_zero(),
            ..Default::default()
        });
    }
    (new_meshlets, new_indices)
}

fn compute_vertex_locks(
    groups: &[Vec<u32>],
    parent_meshlets: &mut [MeshletData],
    vertices: &[MeshVertex],
    indices: &[u32],
) -> Vec<bool> {
    let mut vertex_meshlet_owner = vec![-1; vertices.len()];
    let mut vertex_locks = vec![false; vertices.len()];
    groups.iter().for_each(|meshlets_indices| {
        meshlets_indices.iter().for_each(|&meshlet_index| {
            let meshlet = &parent_meshlets[meshlet_index as usize];
            let count = meshlet.indices_count;
            for i in 0..count {
                let global_index = indices[meshlet.indices_offset as usize + i as usize] as usize;

                if vertex_meshlet_owner[global_index] == -1
                    || vertex_meshlet_owner[global_index] == meshlet_index as i32
                {
                    vertex_meshlet_owner[global_index] = meshlet_index as i32;
                } else {
                    vertex_meshlet_owner[global_index] = -2;
                    vertex_locks[global_index] = true;
                }
            }
        });
    });
    vertex_locks
}

pub fn compute_clusters(
    groups: &[Vec<u32>],
    global_meshlets: &mut [MeshletData],
    global_meshlets_offset: usize,
    mesh_indices_offset: usize,
    vertices: &[MeshVertex],
    indices: &[u32],
    _lod_level: i32,
) -> (Vec<u32>, Vec<MeshletData>) {
    let mut indices_offset = mesh_indices_offset;
    let mut cluster_indices = Vec::new();
    let mut cluster_meshlets = Vec::new();

    let vertex_locks = compute_vertex_locks(groups, global_meshlets, vertices, indices);

    groups.iter().for_each(|meshlets_indices| {
        let mut group_indices = Vec::new();
        let mut group_vertices = Vec::new();
        let num_meshlet_per_group = meshlets_indices.len();

        let mut weight = 0.0_f32;
        let mut center = Vector3::default_zero();

        meshlets_indices.iter().for_each(|&meshlet_index| {
            let meshlet = &global_meshlets[meshlet_index as usize];
            let count = meshlet.indices_count;
            for i in 0..count {
                let global_index = indices[meshlet.indices_offset as usize + i as usize] as usize;
                let group_index = if let Some(index) =
                    group_vertices.iter().position(|v: &MeshVertex| {
                        v.pos.w as usize == global_index || v.is_same_of(&vertices[global_index])
                    }) {
                    index
                } else {
                    let mut v = vertices[global_index];
                    v.pos.w = global_index as f32;
                    group_vertices.push(v);
                    group_vertices.len() - 1
                };
                group_indices.push(group_index as u32);
            }

            center += meshlet.bounding_sphere.xyz() * meshlet.bounding_sphere.z;
            weight += meshlet.bounding_sphere.z;
        });
        if weight > 0.0 {
            center /= weight;
        }
        let mut radius = 0.0_f32;
        let mut max_error = 0.0_f32;
        meshlets_indices.iter().for_each(|&meshlet_index| {
            let meshlet = &global_meshlets[meshlet_index as usize];
            let d = meshlet.bounding_sphere.xyz() - center;
            radius = radius.max(meshlet.bounding_sphere.z + d.length());
            max_error = max_error.max(meshlet.error);
        });

        let vertices_bytes = to_slice(&group_vertices);
        let vertex_stride = size_of::<MeshVertex>();
        debug_assert!(
            vertex_stride.is_multiple_of(size_of::<f32>()),
            "Vertex size is not a multiple of f32 - meshopt will fail"
        );
        let vertex_data_adapter = meshopt::VertexDataAdapter::new(vertices_bytes, vertex_stride, 0);

        let mut target_count = num_meshlet_per_group.div_ceil(2) * MESHLET_MAX_TRIANGLES * 3;
        if target_count >= group_indices.len() {
            target_count = group_indices.len() / 2;
        }

        let mut simplification_error = 0.0;

        let mut simplified_indices = meshopt::simplify_with_locks(
            &group_indices,
            vertex_data_adapter.as_ref().unwrap(),
            &vertex_locks,
            target_count,
            f32::MAX,
            meshopt::SimplifyOptions::Sparse | meshopt::SimplifyOptions::ErrorAbsolute,
            Some(&mut simplification_error),
        );
        max_error += simplification_error;

        meshlets_indices.iter().for_each(|&meshlet_index| {
            let meshlet = &mut global_meshlets[meshlet_index as usize];
            meshlet.parent_bounding_sphere = Vector4::new(center.x, center.y, center.z, radius);
            meshlet.parent_error = max_error;
        });
        //if simplified_indices.len() >= group_indices.len() {
        //    inox_log::debug_log!(
        //        "No simplification happened [from {} to {}]",
        //        group_indices.len(),
        //        simplified_indices.len(),
        //    );
        //}
        if simplified_indices.is_empty() {
            simplified_indices = group_indices;
        }

        let (mut new_meshlets, group_indices) =
            compute_meshlets(&group_vertices, &simplified_indices, indices_offset as u32);

        let mut global_group_indices = Vec::with_capacity(group_indices.len());
        group_indices.iter().for_each(|&i| {
            global_group_indices.push(group_vertices[i as usize].pos.w as u32);
        });

        new_meshlets.iter_mut().for_each(|m| {
            m.error = max_error;
            m.bounding_sphere = Vector4::new(center.x, center.y, center.z, radius);

            meshlets_indices.iter().for_each(|&meshlet_index| {
                m.child_meshlets
                    .push(global_meshlets_offset as u32 + meshlet_index);
            });
        });

        indices_offset += global_group_indices.len();

        cluster_indices.append(&mut global_group_indices);
        cluster_meshlets.append(&mut new_meshlets);
    });
    (cluster_indices, cluster_meshlets)
}
