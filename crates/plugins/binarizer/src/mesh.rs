use std::{collections::HashMap, mem::size_of};

use inox_graphics::{
    MeshData, MeshletData, VertexAttributeLayout, HALF_MESHLETS_GROUP_SIZE, MESHLETS_GROUP_SIZE,
};
use inox_math::{VecBase, Vector2, Vector3, Vector4};
use inox_resources::to_slice;
use meshopt::DecodePosition;

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

#[derive(Default, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Clone)]
struct Edge {
    v1: u32,
    v2: u32,
}
#[derive(Default, Debug, Clone)]
pub(crate) struct MeshletAdjacency {
    meshlet_index: u32,
    edges: Vec<Edge>,
    adjacent_meshlets: Vec<(u32, usize)>,
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
    let max_vertices = 64;
    let max_triangles = 124;
    let cone_weight = 0.7;
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

pub fn build_meshlets_adjacency(
    meshlets: &[MeshletData],
    indices: &[u32],
) -> Vec<MeshletAdjacency> {
    let mut meshlets_info = Vec::with_capacity(meshlets.len());
    let mut edge_meshlets_map: HashMap<Edge, Vec<usize>> = HashMap::default();
    meshlets
        .iter()
        .enumerate()
        .for_each(|(meshlet_index, meshlet)| {
            let triangle_count = meshlet.indices_count / 3;
            let mut edges_hit_count: HashMap<Edge, u32> = HashMap::default();
            for triangle_index in 0..triangle_count {
                let i1 = indices[(meshlet.indices_offset + triangle_index * 3) as usize];
                let i2 = indices[(meshlet.indices_offset + triangle_index * 3 + 1) as usize];
                let i3 = indices[(meshlet.indices_offset + triangle_index * 3 + 2) as usize];
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
                    .entry(e1.clone())
                    .and_modify(|v| *v += 1)
                    .or_insert(1);
                edges_hit_count
                    .entry(e2.clone())
                    .and_modify(|v| *v += 1)
                    .or_insert(1);
                edges_hit_count
                    .entry(e3.clone())
                    .and_modify(|v| *v += 1)
                    .or_insert(1);
                edge_meshlets_map
                    .entry(e1)
                    .and_modify(|v| {
                        if !v.contains(&meshlet_index) {
                            v.push(meshlet_index)
                        }
                    })
                    .or_insert(vec![meshlet_index]);
                edge_meshlets_map
                    .entry(e2)
                    .and_modify(|v| {
                        if !v.contains(&meshlet_index) {
                            v.push(meshlet_index)
                        }
                    })
                    .or_insert(vec![meshlet_index]);
                edge_meshlets_map
                    .entry(e3)
                    .and_modify(|v| {
                        if !v.contains(&meshlet_index) {
                            v.push(meshlet_index)
                        }
                    })
                    .or_insert(vec![meshlet_index]);
            }
            let mut edges = Vec::new();
            for (e, count) in edges_hit_count {
                if count == 1 {
                    edges.push(e);
                }
            }
            meshlets_info.push(MeshletAdjacency {
                meshlet_index: meshlet_index as _,
                edges,
                adjacent_meshlets: Vec::default(),
            });
        });

    let num_meshlets = meshlets_info.len();
    debug_assert!(num_meshlets == meshlets.len());

    meshlets_info
        .iter_mut()
        .enumerate()
        .for_each(|(info_index, info)| {
            info.edges.iter().for_each(|e| {
                if edge_meshlets_map[e].len() > 1 {
                    edge_meshlets_map[e].iter().for_each(|&meshlet_index| {
                        if meshlet_index != info_index {
                            if let Some(i) = info
                                .adjacent_meshlets
                                .iter()
                                .position(|l| l.0 == meshlet_index as u32)
                            {
                                info.adjacent_meshlets[i].1 += 1;
                            } else {
                                info.adjacent_meshlets.push((meshlet_index as u32, 1));
                            }
                        }
                    });
                }
            });
        });
    meshlets_info.iter_mut().for_each(|m| {
        if m.adjacent_meshlets.is_empty() {
            println!("Meshlet {} has no adjacency", m.meshlet_index);
        }
        m.adjacent_meshlets
            .sort_by(|(_i, a), (_j, b)| b.partial_cmp(a).unwrap());
    });
    meshlets_info
}

fn fill_with_info_and_adjacency(
    original_meshlets_info: &[MeshletAdjacency],
    meshlet_info: &MeshletAdjacency,
    meshlets_to_add: &mut Vec<MeshletAdjacency>,
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

pub fn group_meshlets(meshlets_info: &[MeshletAdjacency]) -> Vec<Vec<u32>> {
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
                if let Some(index) = m
                    .adjacent_meshlets
                    .iter()
                    .position(|v| v.0 == meshlet_info.meshlet_index)
                {
                    m.adjacent_meshlets.remove(index);
                } else {
                    println!(
                        "Expecting to find meshlet {} but it's not there {:?}",
                        meshlet_info.meshlet_index, m.adjacent_meshlets
                    )
                }
                meshlet_group.push(m);
            }
        }
        let mut should_retry = meshlet_group.is_empty();
        should_retry |= meshlet_group.len() == 1 && available_meshlets.len() > MESHLETS_GROUP_SIZE;
        should_retry &= !meshlet_group.iter().all(|mi| {
            meshlets_info[mi.meshlet_index as usize]
                .adjacent_meshlets
                .is_empty()
        });
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
                    let mut a = original.adjacent_meshlets.len() as i32 - 1;
                    while a >= 0 && stealed.len() < HALF_MESHLETS_GROUP_SIZE {
                        let mut j = meshlets_groups.len() as i32 - 1;
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

pub fn compute_clusters(
    groups: &[Vec<u32>],
    parent_meshlets: &mut [MeshletData],
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
                    let group_index = if let Some(index) = group_vertices
                        .iter()
                        .position(|v: &LocalVertex| v.pos == pos)
                    {
                        index
                    } else {
                        group_vertices.push(LocalVertex {
                            pos: vertices[global_index].pos,
                            global_index,
                        });
                        group_vertices.len() - 1
                    };
                    group_index
                };
                group_indices.push(group_index as u32);
            }
            aabb_max = aabb_max.max(meshlet.aabb_max);
            aabb_min = aabb_min.min(meshlet.aabb_min);
        });

        let (optimized_vertices, optimized_indices) =
            optimize_mesh(&group_vertices, &group_indices);

        let threshold = 1. / MESHLETS_GROUP_SIZE as f32;
        let target_count = (optimized_indices.len() as f32 * threshold) as usize / 3 * 3;
        let target_error = 0.01;

        let mut simplified_indices = meshopt::simplify_decoder(
            &optimized_indices,
            &optimized_vertices,
            target_count,
            target_error,
            meshopt::SimplifyOptions::LockBorder,
            None,
        );

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
        });
        indices_offset += global_group_indices.len();

        meshlets_indices.iter().for_each(|&meshlet_index| {
            let meshlet = &mut parent_meshlets[meshlet_index as usize];
            for i in 0..meshlets.len() {
                meshlet
                    .child_meshlets
                    .push((cluster_meshlets.len() + i) as u32);
            }
        });
        cluster_indices.append(&mut global_group_indices);
        cluster_meshlets.append(&mut meshlets);
    });
    (cluster_indices, cluster_meshlets)
}
