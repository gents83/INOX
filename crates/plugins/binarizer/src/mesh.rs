use std::{collections::HashMap, mem::size_of};

use inox_bvh::{create_linearized_bvh, BVHTree, AABB};
use inox_graphics::{MeshData, MeshletData, VertexAttributeLayout};
use inox_math::{VecBase, Vector2, Vector3, Vector4};
use inox_resources::to_slice;

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

#[derive(Default, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
struct Edge {
    v1: u32,
    v2: u32,
}
struct MeshletInfo {
    meshlet_index: u32,
    edges: Vec<Edge>,
    adjacent_meshlets: Vec<u32>,
}

pub fn optimize_mesh(optimize_meshes: bool, vertices: &[MeshVertex], indices: &[u32]) -> MeshData {
    let mut mesh_data = MeshData {
        vertex_layout: VertexAttributeLayout::HasPosition,
        aabb_max: Vector3::new(-f32::INFINITY, -f32::INFINITY, -f32::INFINITY),
        aabb_min: Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
        ..Default::default()
    };
    let (vertices, indices) = if optimize_meshes {
        let mut positions = Vec::with_capacity(vertices.len());
        vertices.iter().for_each(|v| {
            positions.push(v.pos);
        });
        let (num_vertices, vertices_remap_table) =
            meshopt::generate_vertex_remap(positions.as_slice(), Some(indices));

        let new_vertices =
            meshopt::remap_vertex_buffer(vertices, num_vertices, vertices_remap_table.as_slice());
        let new_indices = meshopt::remap_index_buffer(
            Some(indices),
            num_vertices,
            vertices_remap_table.as_slice(),
        );

        let mut new_indices = meshopt::optimize_vertex_cache(new_indices.as_slice(), num_vertices);

        let vertices_bytes = to_slice(vertices);
        let vertex_stride = size_of::<MeshVertex>();
        let vertex_data_adapter = meshopt::VertexDataAdapter::new(vertices_bytes, vertex_stride, 0);
        meshopt::optimize_overdraw_in_place(
            new_indices.as_mut_slice(),
            vertex_data_adapter.as_ref().unwrap(),
            1.01,
        );
        let new_vertices =
            meshopt::optimize_vertex_fetch(new_indices.as_mut_slice(), new_vertices.as_slice());

        (new_vertices, new_indices)
    } else {
        (vertices.to_vec(), indices.to_vec())
    };

    vertices.iter().for_each(|v| {
        mesh_data.aabb_max = mesh_data.aabb_max.max(v.pos);
        mesh_data.aabb_min = mesh_data.aabb_min.min(v.pos);
    });
    mesh_data.indices = indices;
    mesh_data.vertex_positions.reserve(vertices.len());
    mesh_data
        .vertex_attributes
        .reserve(VertexAttributeLayout::all().stride_in_count() * vertices.len());
    vertices.iter().for_each(|v| {
        mesh_data.insert_position(v.pos);
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

pub fn compute_meshlets(mesh_data: &mut MeshData) {
    let mut positions = Vec::with_capacity(mesh_data.vertex_count());
    for i in 0..mesh_data.vertex_count() {
        positions.push(mesh_data.position(i));
    }

    let mut new_meshlets = Vec::new();
    let vertices_bytes = to_slice(positions.as_slice());
    let vertex_stride = size_of::<Vector3>();
    let vertex_data_adapter = meshopt::VertexDataAdapter::new(vertices_bytes, vertex_stride, 0);
    let max_vertices = 64;
    let max_triangles = 124;
    let cone_weight = 0.9;
    let meshlets = meshopt::build_meshlets(
        &mesh_data.indices,
        vertex_data_adapter.as_ref().unwrap(),
        max_vertices,
        max_triangles,
        cone_weight,
    );
    if !meshlets.meshlets.is_empty() {
        let mut new_indices = Vec::new();
        for m in meshlets.iter() {
            let bounds = meshopt::compute_meshlet_bounds(m, vertex_data_adapter.as_ref().unwrap());
            let index_offset = new_indices.len();
            debug_assert!(
                m.triangles.len() % 3 == 0,
                "meshlet indices count {} is not divisible by 3",
                m.triangles.len()
            );
            m.triangles.iter().for_each(|v_i| {
                new_indices.push(m.vertices[*v_i as usize]);
            });
            debug_assert!(
                new_indices.len() % 3 == 0,
                "new indices count {} is not divisible by 3",
                new_indices.len()
            );
            let mut triangles_aabbs = Vec::new();
            triangles_aabbs.resize_with(m.triangles.len() / 3, AABB::empty);
            let mut i = 0;
            while i < m.triangles.len() {
                let triangle_id = i / 3;
                let v1 = positions[m.vertices[m.triangles[i] as usize] as usize];
                i += 1;
                let v2 = positions[m.vertices[m.triangles[i] as usize] as usize];
                i += 1;
                let v3 = positions[m.vertices[m.triangles[i] as usize] as usize];
                i += 1;
                let min = v1.min(v2).min(v3);
                let max = v1.max(v2).max(v3);
                triangles_aabbs[triangle_id] = AABB::create(min, max, triangle_id as _);
            }
            let bvh = BVHTree::new(&triangles_aabbs);
            new_meshlets.push(MeshletData {
                indices_offset: index_offset as _,
                indices_count: m.triangles.len() as _,
                aabb_max: bvh.nodes()[0].max(),
                aabb_min: bvh.nodes()[0].min(),
                cone_axis: bounds.cone_axis.into(),
                cone_angle: bounds.cone_cutoff,
                cone_center: bounds.center.into(),
                triangles_bvh: create_linearized_bvh(&bvh),
            });
        }
        mesh_data.indices = new_indices;
    } else {
        let mut triangles_aabbs = Vec::new();
        triangles_aabbs.resize_with(mesh_data.indices.len() / 3, AABB::empty);
        let mut i = 0;
        while i < mesh_data.indices.len() {
            let triangle_id = i / 3;
            let v1 = positions[mesh_data.indices[i] as usize];
            i += 1;
            let v2 = positions[mesh_data.indices[i] as usize];
            i += 1;
            let v3 = positions[mesh_data.indices[i] as usize];
            i += 1;
            let min = v1.min(v2).min(v3);
            let max = v1.max(v2).max(v3);
            triangles_aabbs[triangle_id] = AABB::create(min, max, triangle_id as _);
        }
        let bvh = BVHTree::new(&triangles_aabbs);
        let meshlet = MeshletData {
            indices_offset: 0,
            indices_count: mesh_data.indices.len() as _,
            aabb_max: bvh.nodes()[0].max(),
            aabb_min: bvh.nodes()[0].min(),
            triangles_bvh: create_linearized_bvh(&bvh),
            ..Default::default()
        };
        new_meshlets.push(meshlet);
    }
    mesh_data.meshlets = new_meshlets;

    let mut meshlets_aabbs = Vec::new();
    meshlets_aabbs.resize_with(mesh_data.meshlets.len(), AABB::empty);
    mesh_data.meshlets.iter().enumerate().for_each(|(i, m)| {
        meshlets_aabbs[i] = AABB::create(m.aabb_min, m.aabb_max, i as _);
    });
    let bvh = BVHTree::new(&meshlets_aabbs);
    mesh_data.meshlets_bvh = create_linearized_bvh(&bvh);
}

pub fn build_meshlets_lods(mesh_data: &mut MeshData) {
    let mut meshlets_info = Vec::with_capacity(mesh_data.meshlets.len());
    mesh_data
        .meshlets
        .iter()
        .enumerate()
        .for_each(|(meshlet_index, meshlet)| {
            let triangle_count = meshlet.indices_count / 3;
            let mut edges_hit_count: HashMap<Edge, u32> = HashMap::default();
            for triangle_index in 0..triangle_count {
                let i1 = mesh_data.indices[(meshlet.indices_offset + triangle_index * 3) as usize];
                let i2 =
                    mesh_data.indices[(meshlet.indices_offset + triangle_index * 3 + 1) as usize];
                let i3 =
                    mesh_data.indices[(meshlet.indices_offset + triangle_index * 3 + 2) as usize];
                let e1 = Edge {
                    v1: i1.min(i2),
                    v2: i1.max(i2),
                };
                let e2 = Edge {
                    v1: i2.min(i3),
                    v2: i2.max(i3),
                };
                let e3 = Edge {
                    v1: i3.min(i1),
                    v2: i3.max(i1),
                };
                edges_hit_count
                    .entry(e1)
                    .and_modify(|v| *v += 1)
                    .or_insert(1);
                edges_hit_count
                    .entry(e2)
                    .and_modify(|v| *v += 1)
                    .or_insert(1);
                edges_hit_count
                    .entry(e3)
                    .and_modify(|v| *v += 1)
                    .or_insert(1);
            }
            let mut edges = Vec::new();
            for (e, count) in edges_hit_count {
                if count == 1 {
                    edges.push(e);
                }
            }
            meshlets_info.push(MeshletInfo {
                meshlet_index: meshlet_index as _,
                edges,
                adjacent_meshlets: Vec::default(),
            });
        });

    let num_meshlets = meshlets_info.len();
    debug_assert!(num_meshlets == mesh_data.meshlets.len());

    for i in 0..num_meshlets {
        for j in 1..num_meshlets {
            if i != j {
                let mut is_adjacent = false;
                meshlets_info[i].edges.iter().for_each(|e1| {
                    return meshlets_info[j]
                        .edges
                        .iter()
                        .for_each(|e2| is_adjacent |= e1.v1 == e2.v1 && e1.v2 == e2.v2);
                });
                let other_index = meshlets_info[j].meshlet_index;
                if is_adjacent {
                    meshlets_info[i].adjacent_meshlets.push(other_index);
                }
            }
        }
        debug_assert!(
            !meshlets_info[i].adjacent_meshlets.is_empty(),
            "No adjacency for meshlet {} - count = {}",
            i,
            meshlets_info[i].adjacent_meshlets.len()
        );
    }
}
