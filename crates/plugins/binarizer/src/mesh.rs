use std::mem::size_of;

use inox_graphics::{MeshData, MeshletData, VertexAttributeLayout};
use inox_math::{VecBase, Vector2, Vector3, Vector4};
use inox_resources::to_slice;
use meshopt::DecodePosition;

use crate::adjacency::find_border_vertices;

#[derive(Debug, Clone, Copy)]
pub struct MeshVertex {
    pub pos: Vector3,
    pub color: Option<Vector4>,
    pub normal: Option<Vector3>,
    pub tangent: Option<Vector4>,
    pub uv_0: Option<Vector2>,
    pub uv_1: Option<Vector2>,
    pub uv_2: Option<Vector2>,
    pub uv_3: Option<Vector2>,
}

impl Default for MeshVertex {
    fn default() -> Self {
        Self {
            pos: Vector3::default_zero(),
            color: None,
            normal: None,
            tangent: None,
            uv_0: None,
            uv_1: None,
            uv_2: None,
            uv_3: None,
        }
    }
}

impl meshopt::DecodePosition for MeshVertex {
    fn decode_position(&self) -> [f32; 3] {
        self.pos.into()
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct LocalVertex {
    pos: Vector3,
    global_index: usize,
}

impl Default for LocalVertex {
    fn default() -> Self {
        Self {
            pos: Vector3::default_zero(),
            global_index: 0,
        }
    }
}

impl meshopt::DecodePosition for LocalVertex {
    fn decode_position(&self) -> [f32; 3] {
        self.pos.into()
    }
}

pub fn optimize_mesh<T>(vertices: &[T], indices: &[u32]) -> (Vec<T>, Vec<u32>)
where
    T: Clone + Default + DecodePosition,
{
    let positions = vertices
        .iter()
        .map(|vertex| vertex.decode_position())
        .collect::<Vec<[f32; 3]>>();

    let (num_vertices, vertices_remap_table) =
        meshopt::generate_vertex_remap(&positions, Some(indices));

    let new_vertices =
        meshopt::remap_vertex_buffer(vertices, num_vertices, vertices_remap_table.as_slice());
    let remapped_indices =
        meshopt::remap_index_buffer(Some(indices), num_vertices, vertices_remap_table.as_slice());

    let mut new_indices = meshopt::optimize_vertex_cache(&remapped_indices, num_vertices);

    let vertices_bytes = to_slice(&positions);
    let vertex_stride = size_of::<[f32; 3]>();
    let vertex_data_adapter = meshopt::VertexDataAdapter::new(vertices_bytes, vertex_stride, 0);
    meshopt::optimize_overdraw_in_place(
        &mut new_indices,
        vertex_data_adapter.as_ref().unwrap(),
        1.05,
    );
    let new_vertices = meshopt::optimize_vertex_fetch(&mut new_indices, &new_vertices);

    (new_vertices, new_indices)
}

pub fn create_mesh_data(vertices: &[MeshVertex], indices: &[u32]) -> MeshData {
    let mut mesh_data = MeshData {
        vertex_layout: VertexAttributeLayout::HasPosition,
        aabb_max: Vector3::new(-f32::INFINITY, -f32::INFINITY, -f32::INFINITY),
        aabb_min: Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
        ..Default::default()
    };
    vertices.iter().for_each(|v| {
        mesh_data.aabb_max = mesh_data.aabb_max.max(v.pos);
        mesh_data.aabb_min = mesh_data.aabb_min.min(v.pos);
    });
    mesh_data.indices = indices.to_vec();
    mesh_data.vertex_positions.reserve(vertices.len());
    mesh_data
        .vertex_attributes
        .reserve(VertexAttributeLayout::all().stride_in_count() * vertices.len());
    vertices.iter().for_each(|v| {
        mesh_data.insert_position(v.decode_position().into());
        if let Some(c) = v.color {
            mesh_data.vertex_layout |= VertexAttributeLayout::HasColor;
            mesh_data.insert_color(c);
        }
        if let Some(n) = v.normal {
            mesh_data.vertex_layout |= VertexAttributeLayout::HasNormal;
            mesh_data.insert_normal(n);
        }
        if let Some(t) = v.tangent {
            mesh_data.vertex_layout |= VertexAttributeLayout::HasTangent;
            mesh_data.insert_tangent(t);
        }
        if let Some(uv) = v.uv_0 {
            mesh_data.vertex_layout |= VertexAttributeLayout::HasUV1;
            mesh_data.insert_uv(uv);
        }
        if let Some(uv) = v.uv_1 {
            mesh_data.vertex_layout |= VertexAttributeLayout::HasUV2;
            mesh_data.insert_uv(uv);
        }
        if let Some(uv) = v.uv_2 {
            mesh_data.vertex_layout |= VertexAttributeLayout::HasUV3;
            mesh_data.insert_uv(uv);
        }
        if let Some(uv) = v.uv_3 {
            mesh_data.vertex_layout |= VertexAttributeLayout::HasUV4;
            mesh_data.insert_uv(uv);
        }
    });
    mesh_data
}

pub fn compute_meshlets<T>(vertices: &[T], indices: &[u32]) -> (Vec<MeshletData>, Vec<u32>)
where
    T: DecodePosition,
{
    let positions = vertices
        .iter()
        .map(|vertex| vertex.decode_position())
        .collect::<Vec<[f32; 3]>>();
    let vertices_bytes = to_slice(&positions);
    let vertex_stride = size_of::<[f32; 3]>();
    let vertex_data_adapter = meshopt::VertexDataAdapter::new(vertices_bytes, vertex_stride, 0);

    let mut new_meshlets = Vec::new();
    let max_vertices = 128;
    let max_triangles = 256;
    let cone_weight = 0.5;
    let meshlets = meshopt::build_meshlets(
        indices,
        vertex_data_adapter.as_ref().unwrap(),
        max_vertices,
        max_triangles,
        cone_weight,
    );
    debug_assert!(!meshlets.meshlets.is_empty());

    let mut new_indices = Vec::new();
    for m in meshlets.iter() {
        let index_offset = new_indices.len();
        debug_assert!(m.triangles.len() % 3 == 0);
        let mut aabb_max = Vector3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
        let mut aabb_min = Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        m.triangles.iter().for_each(|&i| {
            let index = m.vertices[i as usize] as usize;
            new_indices.push(index as u32);
            let pos = positions[index].into();
            aabb_min = aabb_min.min(pos);
            aabb_max = aabb_max.max(pos);
        });
        debug_assert!(new_indices.len() % 3 == 0);
        new_meshlets.push(MeshletData {
            indices_offset: index_offset as _,
            indices_count: m.triangles.len() as _,
            aabb_min,
            aabb_max,
            ..Default::default()
        });
    }
    (new_meshlets, new_indices)
}

pub fn compute_clusters(
    groups: &[Vec<u32>],
    parent_meshlets: &mut [MeshletData],
    parent_meshlets_offset: usize,
    mesh_indices_offset: usize,
    vertices: &[MeshVertex],
    indices: &[u32],
) -> (Vec<u32>, Vec<MeshletData>) {
    let mut indices_offset = mesh_indices_offset;
    let mut cluster_indices = Vec::new();
    let mut cluster_meshlets = Vec::new();
    groups.iter().for_each(|meshlets_indices| {
        let mut group_indices = Vec::new();
        let mut group_vertices = Vec::new();
        let mut aabb_max = Vector3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
        let mut aabb_min = Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);

        meshlets_indices.iter().for_each(|&meshlet_index| {
            let meshlet = &parent_meshlets[meshlet_index as usize];
            let count = meshlet.indices_count;
            for i in 0..count {
                let global_index = indices[meshlet.indices_offset as usize + i as usize] as usize;
                let group_index = if let Some(index) = group_vertices
                    .iter()
                    .position(|v: &LocalVertex| v.global_index == global_index)
                {
                    index
                } else {
                    let pos = vertices[global_index].pos;
                    group_vertices.push(LocalVertex { pos, global_index });
                    group_vertices.len() - 1
                };
                group_indices.push(group_index as u32);
            }
            aabb_max = aabb_max.max(meshlet.aabb_max);
            aabb_min = aabb_min.min(meshlet.aabb_min);
        });

        let (optimized_vertices, optimized_indices) = (group_vertices, group_indices);
        //optimize_mesh(&group_vertices, &group_indices);

        let target_count = (optimized_indices.len() as f32 * 0.5) as usize;
        let target_error = 0.01;

        let border_vertices = find_border_vertices(&optimized_indices);
        let mut simplified_indices = meshopt::simplify_decoder(
            &optimized_indices,
            &optimized_vertices,
            target_count,
            target_error,
            0,
            Some(&border_vertices),
        );

        if simplified_indices.len() >= optimized_indices.len() {
            inox_log::debug_log!(
                "No simplification happened [from {} to {}] even if only {} locked_vertices",
                optimized_indices.len(),
                simplified_indices.len(),
                border_vertices.len()
            );
        }

        if simplified_indices.is_empty() {
            simplified_indices = optimized_indices;
        }

        let (mut meshlets, group_indices) =
            compute_meshlets(&optimized_vertices, &simplified_indices);

        let mut global_group_indices = Vec::with_capacity(group_indices.len());
        group_indices.iter().for_each(|&i| {
            global_group_indices.push(optimized_vertices[i as usize].global_index as u32);
        });
        meshlets.iter_mut().for_each(|m| {
            m.indices_offset += indices_offset as u32;
            meshlets_indices.iter().for_each(|&meshlet_index| {
                m.child_meshlets
                    .push(parent_meshlets_offset as u32 + meshlet_index);
            });
        });
        indices_offset += global_group_indices.len();

        cluster_indices.append(&mut global_group_indices);
        cluster_meshlets.append(&mut meshlets);
    });
    (cluster_indices, cluster_meshlets)
}

#[test]
fn simplify_test() {
    // 4----5----6
    // |    |    |
    // 1----2----7
    // |    |    |
    // 0----3----8
    #[rustfmt::skip]
    let vertices = [
        LocalVertex{ pos: Vector3::new(0., 0., 0.), global_index: 0 },
        LocalVertex{ pos: Vector3::new(0., 1., 0.), global_index: 1 },
        LocalVertex{ pos: Vector3::new(1., 1., 0.), global_index: 2 },
        LocalVertex{ pos: Vector3::new(1., 0., 0.), global_index: 3 },
        LocalVertex{ pos: Vector3::new(0., 2., 0.), global_index: 4 },
        LocalVertex{ pos: Vector3::new(1., 2., 0.), global_index: 5 },
        LocalVertex{ pos: Vector3::new(2., 2., 0.), global_index: 6 },
        LocalVertex{ pos: Vector3::new(2., 1., 0.), global_index: 7 },
        LocalVertex{ pos: Vector3::new(2., 0., 0.), global_index: 8 },
    ];
    #[rustfmt::skip]
    let indices = [
        0, 1, 2,
        2, 3, 0,
        1, 4, 5,
        5, 2, 1,
        2, 5, 6,
        6, 7, 2,
        2, 7, 3,
        3, 7, 8,
    ];
    let target_count = 6;
    let target_error = 0.01;

    let simplified_indices = meshopt::simplify_decoder(
        &indices,
        &vertices,
        target_count,
        target_error,
        1, //meshopt::SimplifyOptions::LockBorder,
        None,
    );

    debug_assert!(
        simplified_indices.len() < indices.len(),
        "No simplification happened"
    );
}
