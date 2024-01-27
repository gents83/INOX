use std::{collections::HashMap, mem::size_of};

use inox_bvh::{create_linearized_bvh, BVHTree, AABB};
use inox_graphics::{
    MeshData, MeshletData, VertexAttributeLayout, HALF_MESHLETS_GROUP_SIZE, MESHLETS_GROUP_SIZE,
};
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

#[derive(Default, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Clone)]
pub struct Edge {
    v1: u32,
    v2: u32,
}
#[derive(Default, Debug, Clone)]
pub struct MeshletInfo {
    meshlet_index: u32,
    edges: Vec<Edge>,
    adjacent_meshlets: Vec<(u32, usize)>,
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
                triangles_bvh: create_linearized_bvh(&bvh),
                ..Default::default()
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
    //lod_level = 0
    mesh_data.meshlets[0] = new_meshlets;
    println!("LOD {} has {} Meshlets", 0, mesh_data.meshlets[0].len());

    let mut meshlets_aabbs = Vec::new();
    meshlets_aabbs.resize_with(mesh_data.meshlets[0].len(), AABB::empty);
    mesh_data.meshlets[0].iter().enumerate().for_each(|(i, m)| {
        meshlets_aabbs[i] = AABB::create(m.aabb_min, m.aabb_max, i as _);
    });
    let bvh = BVHTree::new(&meshlets_aabbs);
    mesh_data.meshlets_bvh[0] = create_linearized_bvh(&bvh);
}

pub fn build_meshlets_info(mesh_data: &mut MeshData, lod_level: usize) -> Vec<MeshletInfo> {
    let mut meshlets_info = Vec::with_capacity(mesh_data.meshlets[lod_level].len());
    mesh_data.meshlets[lod_level]
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
    debug_assert!(num_meshlets == mesh_data.meshlets[lod_level].len());
    if num_meshlets > 1 {
        for i in 0..num_meshlets {
            for j in 0..num_meshlets {
                if i != j {
                    let num_adjacency = meshlets_info[i].edges.iter().fold(0, |c, e1| {
                        let num =
                            meshlets_info[j]
                                .edges
                                .iter()
                                .fold(0, |c, e2| if e1 == e2 { c + 1 } else { c });
                        c + num
                    });
                    let other_index = meshlets_info[j].meshlet_index;
                    if num_adjacency > 0 {
                        meshlets_info[i]
                            .adjacent_meshlets
                            .push((other_index, num_adjacency));
                    }
                }
            }
            if meshlets_info[i].adjacent_meshlets.is_empty() {
                println!("No adjacency for meshlet {} of {}", i, num_meshlets);
            }
        }
    }
    meshlets_info.iter_mut().for_each(|m| {
        m.adjacent_meshlets
            .sort_by(|(_i, a), (_j, b)| b.partial_cmp(a).unwrap());
    });
    meshlets_info
}

fn fill_with_info_and_adjacency(
    original_meshlets_info: &[MeshletInfo],
    meshlet_info: &MeshletInfo,
    meshlets_to_add: &mut Vec<MeshletInfo>,
) {
    if !meshlets_to_add
        .iter()
        .any(|m| m.meshlet_index == meshlet_info.meshlet_index)
    {
        if let Some(p) = original_meshlets_info
            .iter()
            .position(|m| m.meshlet_index == meshlet_info.meshlet_index)
        {
            let original = original_meshlets_info[p].clone();
            meshlets_to_add.push(original);
            original_meshlets_info[p]
                .adjacent_meshlets
                .iter()
                .for_each(|&(i, _)| {
                    fill_with_info_and_adjacency(
                        original_meshlets_info,
                        &original_meshlets_info[i as usize],
                        meshlets_to_add,
                    );
                });
        }
    }
}

pub fn group_meshlets(meshlets_info: &[MeshletInfo]) -> Vec<Vec<u32>> {
    let mut available_meshlets = meshlets_info.to_vec();
    let mut meshlets_groups = Vec::new();
    while !available_meshlets.is_empty() {
        let mut meshlet_group = Vec::new();
        meshlet_group.push(available_meshlets.remove(0));
        let mut meshlet_current_index = 0;
        while meshlet_group.len() < MESHLETS_GROUP_SIZE {
            let mut max_adjacency_value = -1;
            let mut adjacent_index = -1;
            meshlet_group.iter().enumerate().for_each(|(i, m)| {
                if let Some(index) = m
                    .adjacent_meshlets
                    .iter()
                    .position(|v| v.1 as i32 > max_adjacency_value)
                {
                    max_adjacency_value = m.adjacent_meshlets[index].1 as i32;
                    adjacent_index = index as i32;
                    meshlet_current_index = i;
                }
            });
            if max_adjacency_value < 0 || adjacent_index < 0 {
                break;
            }
            let meshlet_info = &mut meshlet_group[meshlet_current_index];
            let (other_index, _) = meshlet_info
                .adjacent_meshlets
                .remove(adjacent_index as usize);
            if let Some(other_available_index) = available_meshlets
                .iter()
                .position(|m| m.meshlet_index == other_index)
            {
                let mut m = available_meshlets.remove(other_available_index);
                let index = m
                    .adjacent_meshlets
                    .iter()
                    .position(|v| v.0 == meshlet_info.meshlet_index)
                    .unwrap();
                m.adjacent_meshlets.remove(index);
                meshlet_group.push(m);
            }
        }
        let mut should_retry = meshlet_group.is_empty();
        should_retry = should_retry
            || (meshlet_group.len() == 1 && available_meshlets.len() > MESHLETS_GROUP_SIZE);
        if !should_retry || available_meshlets.is_empty() {
            meshlets_groups.push(meshlet_group);
        } else {
            //steal from groups already created
            let mut stealed = Vec::new();
            meshlet_group.iter().for_each(|info| {
                if let Some(p) = meshlets_info
                    .iter()
                    .position(|m| m.meshlet_index == info.meshlet_index)
                {
                    let original = &meshlets_info[p];
                    let mut a = (original.adjacent_meshlets.len() - 1) as i32;
                    while a >= 0 && stealed.len() < HALF_MESHLETS_GROUP_SIZE {
                        let mut j = (meshlets_groups.len() - 1) as i32;
                        while a >= 0 && j >= 0 && stealed.len() < HALF_MESHLETS_GROUP_SIZE {
                            if meshlets_groups[j as usize].len() > HALF_MESHLETS_GROUP_SIZE {
                                if let Some(i) = meshlets_groups[j as usize].iter().position(|m| {
                                    m.meshlet_index == original.adjacent_meshlets[a as usize].0
                                }) {
                                    stealed.push(meshlets_groups[j as usize].remove(i));
                                    a -= 1;
                                }
                            }
                            j -= 1;
                        }
                        a -= 1;
                    }
                }
            });
            meshlet_group.append(&mut stealed);
            if meshlet_group.len() == 1 {
                //readd all to the available meshlets
                let last_index = available_meshlets.len();
                meshlet_group.iter().for_each(|info| {
                    fill_with_info_and_adjacency(meshlets_info, info, &mut available_meshlets);
                });
                (last_index..available_meshlets.len()).for_each(|i| {
                    let info = &available_meshlets[i];
                    meshlets_groups.retain(|group| {
                        !group.iter().any(|m| m.meshlet_index == info.meshlet_index)
                    });
                });
            } else {
                meshlets_groups.push(meshlet_group);
            }
        }
    }
    debug_assert!(available_meshlets.is_empty());
    let num_total_meslets = meshlets_groups.iter().fold(0, |i, e| i + e.len());
    debug_assert!(
        num_total_meslets == meshlets_info.len(),
        "Not enough meshlets {}/{} in {} groups",
        num_total_meslets,
        meshlets_info.len(),
        meshlets_groups.len()
    );
    meshlets_groups
        .iter()
        .map(|info| info.iter().map(|m| m.meshlet_index).collect::<_>())
        .collect::<_>()
}

pub fn generate_meshlets_for_level(
    lod_level: usize,
    groups: &[Vec<u32>],
    mesh_data: &mut MeshData,
) {
    let previous_lod_level = lod_level - 1;
    let mut positions = Vec::with_capacity(mesh_data.vertex_count());
    for i in 0..mesh_data.vertex_count() {
        positions.push(mesh_data.position(i));
    }

    let vertices_bytes = to_slice(positions.as_slice());
    let vertex_stride = size_of::<Vector3>();
    let vertex_data_adapter = meshopt::VertexDataAdapter::new(vertices_bytes, vertex_stride, 0);

    groups.iter().for_each(|meshlets_indices| {
        let mut group_indices = Vec::new();
        let first_meshlet = &mesh_data.meshlets[previous_lod_level][meshlets_indices[0] as usize];
        let mut aabb_max = first_meshlet.aabb_max;
        let mut aabb_min = first_meshlet.aabb_min;

        meshlets_indices.iter().for_each(|&meshlet_index| {
            let meshlet = &mesh_data.meshlets[previous_lod_level][meshlet_index as usize];
            let offset = meshlet.indices_offset;
            let count = meshlet.indices_count;
            for i in 0..count {
                let vertex_index = mesh_data.indices[(offset + i) as usize];
                group_indices.push(vertex_index);
            }
            aabb_max = aabb_max.max(meshlet.aabb_max);
            aabb_min = aabb_min.min(meshlet.aabb_min);
        });

        let threshold = 0.25f32;
        let target_count = (group_indices.len() as f32 * threshold) as usize / 3 * 3;
        let target_error = 1e-2f32;

        let mut meshlet_indices = meshopt::simplify(
            group_indices.as_slice(),
            vertex_data_adapter.as_ref().unwrap(),
            target_count,
            target_error,
            meshopt::SimplifyOptions::LockBorder,
            None,
        );
        let indices_offset = mesh_data.indices.len();
        let indices_count = meshlet_indices.len();

        let mut triangles_aabbs = Vec::new();
        let mut i = 0;
        while i < meshlet_indices.len() {
            let v1 = mesh_data.position(mesh_data.indices[i] as _);
            i += 1;
            let v2 = mesh_data.position(mesh_data.indices[i] as _);
            i += 1;
            let v3 = mesh_data.position(mesh_data.indices[i] as _);
            i += 1;
            let min = v1.min(v2).min(v3);
            let max = v1.max(v2).max(v3);
            let triangle_id = triangles_aabbs.len();
            triangles_aabbs.push(AABB::create(min, max, triangle_id as _));
        }
        mesh_data.indices.append(&mut meshlet_indices);

        let bvh = BVHTree::new(&triangles_aabbs);
        let meshlet_data = MeshletData {
            aabb_min,
            indices_offset: indices_offset as _,
            aabb_max,
            indices_count: indices_count as _,
            child_meshlets: meshlets_indices.clone(),
            triangles_bvh: create_linearized_bvh(&bvh),
        };
        if mesh_data.meshlets.len() <= lod_level {
            mesh_data.meshlets.push(Vec::new());
        }
        mesh_data.meshlets[lod_level].push(meshlet_data);

        let mut meshlets_aabbs = Vec::new();
        meshlets_aabbs.resize_with(mesh_data.meshlets[lod_level].len(), AABB::empty);
        mesh_data.meshlets[lod_level]
            .iter()
            .enumerate()
            .for_each(|(i, m)| {
                meshlets_aabbs[i] = AABB::create(m.aabb_min, m.aabb_max, i as _);
            });
        let bvh = BVHTree::new(&meshlets_aabbs);
        if mesh_data.meshlets_bvh.len() <= lod_level {
            mesh_data.meshlets_bvh.push(Vec::new());
        }
        mesh_data.meshlets_bvh[lod_level] = create_linearized_bvh(&bvh);
    });
    println!(
        "LOD {} has {} Meshlets",
        lod_level,
        mesh_data.meshlets[lod_level].len()
    );
}
