use std::collections::HashMap;

use inox_graphics::{MeshletData, HALF_MESHLETS_GROUP_SIZE, MESHLETS_GROUP_SIZE};
use inox_math::{VecBaseFloat, Vector3};
use meshopt::DecodePosition;

const VERTICES_DISTANCE_EPSILON: f32 = 0.1;

#[derive(Default, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Clone)]
struct Edge {
    v1: u32,
    v2: u32,
}

impl Edge {
    fn create(i1: u32, i2: u32) -> Self {
        Self {
            v1: i2.min(i1),
            v2: i2.max(i1),
        }
    }
    fn add_to_hit_count(&self, edges_hit_count: &mut HashMap<Edge, u32>) {
        edges_hit_count
            .entry(self.clone())
            .and_modify(|v| *v += 1)
            .or_insert(1);
    }
    fn add_to_meshlets_map(
        &self,
        edge_meshlets_map: &mut HashMap<Edge, Vec<usize>>,
        meshlet_index: usize,
    ) {
        edge_meshlets_map
            .entry(self.clone())
            .and_modify(|v| {
                if !v.contains(&meshlet_index) {
                    v.push(meshlet_index)
                }
            })
            .or_insert(vec![meshlet_index]);
    }
}

#[derive(Debug, PartialEq, Clone)]
struct EdgePos {
    v1: Vector3,
    v2: Vector3,
}

impl EdgePos {
    fn create(p1: Vector3, p2: Vector3) -> Self {
        let v1 = if p1.x < p2.x {
            p1
        } else if p1.x > p2.x {
            p2
        } else if p1.y < p2.y {
            p1
        } else if p1.y > p2.y {
            p2
        } else if p1.z < p2.z {
            p1
        } else if p1.z > p2.z {
            p2
        } else {
            p1
        };
        let v2 = if v1 == p1 { p2 } else { p1 };
        Self { v1, v2 }
    }
    fn is_close_to(&self, other: &Self, epsilon: f32) -> bool {
        (self.v1 - other.v1).length() < epsilon && (self.v2 - other.v2).length() < epsilon
    }
    fn add_to_meshlets_map(
        &self,
        edgepos_meshlets_map: &mut Vec<(EdgePos, Vec<usize>)>,
        meshlet_index: usize,
        epsilon: f32,
    ) {
        if let Some(position) = edgepos_meshlets_map
            .iter()
            .position(|e| self.is_close_to(&e.0, epsilon))
        {
            if !edgepos_meshlets_map[position].1.contains(&meshlet_index) {
                edgepos_meshlets_map[position].1.push(meshlet_index)
            }
        } else {
            edgepos_meshlets_map.push((self.clone(), vec![meshlet_index]))
        }
    }
}

#[derive(Default, Debug, Clone)]
pub(crate) struct MeshletAdjacency {
    meshlet_index: u32,
    border_edges: Vec<Edge>,
    adjacent_meshlets: Vec<(u32, usize)>,
}

pub fn build_meshlets_adjacency<T>(
    meshlets: &[MeshletData],
    vertices: &[T],
    indices: &[u32],
) -> Vec<MeshletAdjacency>
where
    T: DecodePosition,
{
    let mut meshlets_info = Vec::with_capacity(meshlets.len());
    let mut edge_meshlets_map: HashMap<Edge, Vec<usize>> = HashMap::default();
    let mut edge_pos_meshlets_map: Vec<(EdgePos, Vec<usize>)> = Vec::default();
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
                let e1 = Edge::create(i1, i2);
                let e2 = Edge::create(i2, i3);
                let e3 = Edge::create(i3, i1);
                e1.add_to_hit_count(&mut edges_hit_count);
                e2.add_to_hit_count(&mut edges_hit_count);
                e3.add_to_hit_count(&mut edges_hit_count);
                e1.add_to_meshlets_map(&mut edge_meshlets_map, meshlet_index);
                e2.add_to_meshlets_map(&mut edge_meshlets_map, meshlet_index);
                e3.add_to_meshlets_map(&mut edge_meshlets_map, meshlet_index);
            }
            let mut border_edges = Vec::with_capacity(edges_hit_count.len());
            for (e, count) in edges_hit_count {
                if count == 1 {
                    let p1: Vector3 = vertices[e.v1 as usize].decode_position().into();
                    let p2: Vector3 = vertices[e.v2 as usize].decode_position().into();
                    let e_pos = EdgePos::create(p1, p2);
                    e_pos.add_to_meshlets_map(
                        &mut edge_pos_meshlets_map,
                        meshlet_index,
                        VERTICES_DISTANCE_EPSILON,
                    );
                    border_edges.push(e);
                }
            }
            meshlets_info.push(MeshletAdjacency {
                meshlet_index: meshlet_index as _,
                border_edges,
                adjacent_meshlets: Vec::default(),
            });
        });

    let num_meshlets = meshlets_info.len();
    debug_assert!(num_meshlets == meshlets.len());

    meshlets_info
        .iter_mut()
        .enumerate()
        .for_each(|(info_index, info)| {
            info.border_edges.iter().for_each(|e| {
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
    edge_pos_meshlets_map.iter().for_each(|(_, meshlets)| {
        meshlets.iter().for_each(|&m1| {
            if let Some(m1_index) = meshlets_info
                .iter()
                .position(|m| m.meshlet_index as usize == m1)
            {
                meshlets.iter().for_each(|&m2| {
                    if m1 != m2 {
                        if let Some(i) = meshlets_info[m1_index]
                            .adjacent_meshlets
                            .iter()
                            .position(|l| l.0 == m1 as u32)
                        {
                            meshlets_info[m1_index].adjacent_meshlets[i].1 += 1;
                        } else {
                            meshlets_info[m1_index]
                                .adjacent_meshlets
                                .push((m2 as u32, 1));
                        }
                        if let Some(m2_index) = meshlets_info
                            .iter()
                            .position(|m| m.meshlet_index as usize == m2)
                        {
                            if let Some(i) = meshlets_info[m2_index]
                                .adjacent_meshlets
                                .iter()
                                .position(|l| l.0 == m1 as u32)
                            {
                                meshlets_info[m2_index].adjacent_meshlets[i].1 += 1;
                            } else {
                                meshlets_info[m2_index]
                                    .adjacent_meshlets
                                    .push((m1 as u32, 1));
                            }
                        }
                    }
                });
            }
        });
    });
    let num_meshlets = meshlets_info.len();
    meshlets_info.iter_mut().for_each(|m| {
        if num_meshlets > 1 && m.adjacent_meshlets.is_empty() {
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
