#![allow(dead_code)]
use std::{cmp::Ordering, collections::BinaryHeap};

use inox_math::{Mat4Ops, MatBase, Matrix4, VecBase, VecBaseFloat, Vector3, Vector4};
use meshopt::DecodePosition;

const COST_EPS: f32 = 1e32;
const DIST_EPS: f32 = 1e32;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Edge {
    pub v1: usize,
    pub v2: usize,
    pub quadric_error: Matrix4, // sum of errors matrices
    pub v: Vector3,             // resolved position
    pub cost: f32,              //cost of edge shrinkage
}

impl Edge {
    pub fn new(v1: usize, v2: usize, qv: Matrix4, v: Vector3, cost: f32) -> Self {
        Self {
            v1,
            v2,
            quadric_error: qv,
            v,
            cost,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct State {
    pub edge_cost: f32,
    pub edge_index: usize,
}

impl State {
    pub fn new(cost: f32, index: usize) -> Self {
        Self {
            edge_cost: cost,
            edge_index: index,
        }
    }
}

impl Eq for State {}

impl Ord for State {
    fn cmp(&self, other: &State) -> Ordering {
        other.partial_cmp(self).unwrap()
    }
}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for State {
    fn partial_cmp(&self, other: &State) -> Option<Ordering> {
        other.edge_cost.partial_cmp(&self.edge_cost)
    }
}

fn matrix4_split(mat: &Matrix4) -> Matrix4 {
    let mut ret = *mat;
    ret[3][0] = 0.0;
    ret[3][1] = 0.0;
    ret[3][2] = 0.0;
    ret[3][3] = 1.0;
    ret
}

fn matrix4_multiply_by_vec4(mat: &Matrix4, v: Vector4) -> f32 {
    // vQv^T
    let mut tmp = [0.0, 0.0, 0.0, 0.0];
    (0..4).for_each(|i| {
        for j in 0..4 {
            tmp[i] += v[j] * mat[j][i];
        }
    });
    let mut ret = 0.0;
    (0..4).for_each(|i| {
        ret += tmp[i] * v[i];
    });
    ret
}

//This is the error matrix used to calculate the plane, where % is the vector cross product I defined.
//The time complexity of the entire function is O(1)
fn compute_quadric_error(v1: &Vector3, v2: &Vector3, v3: &Vector3) -> Matrix4 {
    let mut q = Matrix4::default_identity();
    let norm = ((*v1 - *v3).cross(*v2 - *v3)).normalized();
    let v = [norm.x, norm.y, norm.z, -norm.dot_product(*v3)];
    for i in 0..4 {
        for j in 0..4 {
            q[i][j] = v[i] * v[j];
        }
    }
    q
}

fn is_edge_valid(
    pos: &[Vector3],
    q: &[Matrix4],
    i1: usize,
    i2: usize,
) -> Option<(Matrix4, Vector3, f32)> {
    let e = pos[i1] - pos[i2];
    if e.dot_product(e) >= DIST_EPS {
        return None;
    }
    let qv = q[i1] + q[i2];
    let inv = matrix4_split(&qv).try_inverse(); // inverse of Gaussian elimination
    let v = match inv {
        Some(inv) => Vector3::new(inv[0][3], inv[1][3], inv[2][3]), //new vertex
        None => (pos[i1] + pos[i2]) * 0.5,                          //use midpoint
    };
    let cost = matrix4_multiply_by_vec4(&qv, Vector4::new(v.x, v.y, v.z, 1.));
    if cost >= COST_EPS {
        return None;
    }
    Some((qv, v, cost))
}

fn is_triangle_valid(dele: &[bool], tri: &[(usize, usize, usize)], i: usize) -> bool {
    !(dele[tri[i].0] || dele[tri[i].1] || dele[tri[i].2])
}

fn is_triangle_valid_with_locked(
    dele: &[bool],
    locked_indices: &[u32],
    tri: &[(usize, usize, usize)],
    i: usize,
) -> bool {
    // Only consider triangles where none of the vertices are locked or deleted
    !(dele[tri[i].0]
        || dele[tri[i].1]
        || dele[tri[i].2]
        || locked_indices.contains(&(tri[i].0 as u32))
        || locked_indices.contains(&(tri[i].1 as u32))
        || locked_indices.contains(&(tri[i].2 as u32)))
}

fn is_in_triangle(tri: &[(usize, usize, usize)], i: usize, v: usize) -> bool {
    tri[i].0 == v || tri[i].1 == v || tri[i].2 == v
}

pub fn simplify<T>(
    vertices: &[T],
    indices: &[u32],
    ratio: f32,
    locked_indices: Option<&[u32]>,
) -> Vec<u32>
where
    T: Clone + Default + DecodePosition,
{
    let vertices_count = vertices.len();
    let triangle_count = indices.len() / 3;

    let mut pos: Vec<Vector3> = vertices
        .iter()
        .map(|v| v.decode_position().into())
        .collect::<_>();
    let mut tri = Vec::with_capacity(triangle_count);
    for triangle_index in 0..triangle_count {
        let v1: usize = indices[triangle_index * 3] as usize;
        let v2 = indices[triangle_index * 3 + 1] as usize;
        let v3 = indices[triangle_index * 3 + 2] as usize;
        tri.push((v1, v2, v3));
    }

    let mut vertices_error = vec![Matrix4::default_identity(); vertices_count];
    let mut surface_error = Vec::with_capacity(triangle_count);
    let mut triangles_per_vertex = vec![Vec::default(); vertices_count];

    tri.iter().enumerate().for_each(|(i, &(v1, v2, v3))| {
        let qk = compute_quadric_error(&pos[v1], &pos[v2], &pos[v3]);
        surface_error.push(qk);
        vertices_error[v1] += qk;
        vertices_error[v2] += qk;
        vertices_error[v3] += qk;
        triangles_per_vertex[v1].push(i);
        triangles_per_vertex[v2].push(i);
        triangles_per_vertex[v3].push(i);
    });

    let mut deleted_vertices = vec![false; vertices_count];
    let mut edge = vec![];
    let mut heap = BinaryHeap::new();

    tri.iter().for_each(|&(v1, v2, v3)| {
        if let Some((qv, v, cost)) = is_edge_valid(&pos, &vertices_error, v1, v2) {
            edge.push(Edge::new(v1, v2, qv, v, cost));
            heap.push(State::new(cost, edge.len() - 1));
        }
        if let Some((qv, v, cost)) = is_edge_valid(&pos, &vertices_error, v2, v3) {
            edge.push(Edge::new(v2, v3, qv, v, cost));
            heap.push(State::new(cost, edge.len() - 1));
        }
        if let Some((qv, v, cost)) = is_edge_valid(&pos, &vertices_error, v1, v3) {
            edge.push(Edge::new(v1, v3, qv, v, cost));
            heap.push(State::new(cost, edge.len() - 1));
        }
    });

    let mut limit = (triangle_count as f32 * (1.0 - ratio)) as i32;

    while let Some(state) = heap.pop() {
        let e = edge[state.edge_index];
        //If a point is deleted, this edge must have been deleted
        if deleted_vertices[e.v1] || deleted_vertices[e.v2] {
            continue;
        }
        let mut adiacency_v = Vec::new();
        let mut edge_v = Vec::new();
        vertices_error.push(Matrix4::default_identity());
        pos.push(e.v);
        deleted_vertices.push(false);
        let v = pos.len() - 1;

        //Classify the two vertices and adjacent faces of this edge
        triangles_per_vertex[e.v1].iter().for_each(|&i| {
            let is_valid_triangle = if let Some(locked_indices) = locked_indices {
                is_triangle_valid_with_locked(&deleted_vertices, locked_indices, &tri, i)
            } else {
                is_triangle_valid(&deleted_vertices, &tri, i)
            };
            if is_valid_triangle {
                if !is_in_triangle(&tri, i, e.v2) {
                    //A triangle with only one vertex being e.v1
                    if tri[i].1 == e.v1 {
                        tri[i] = (tri[i].1, tri[i].2, tri[i].0); // Find the vertex that is v1 and swap it to the first position
                    } else if tri[i].2 == e.v1 {
                        tri[i] = (tri[i].2, tri[i].0, tri[i].1); // Find the vertex that is v1 and swap it to the first position
                    }
                    let (v2, v3) = (tri[i].1, tri[i].2);
                    //Calculate the error matrix between the new point and the other two vertices
                    let error = compute_quadric_error(&pos[v], &pos[v2], &pos[v3]);
                    let error_difference = error - surface_error[i];
                    adiacency_v.push(i);
                    //Update the contribution of 3 points
                    vertices_error[v] += error;
                    vertices_error[v2] += error_difference;
                    vertices_error[v3] += error_difference;
                    //Directly replace the old with the new one
                    surface_error[i] = error;
                    tri[i].0 = v;
                    //Add the points to be connected by v to the candidate array, wait for the merger to be completed, and then connect the edges
                    edge_v.push(v2);
                    edge_v.push(v3);
                } else {
                    //A triangle with two vertices e.v1 and e.v2
                    let v3 = tri[i].0 + tri[i].1 + tri[i].2 - e.v1 - e.v2;
                    vertices_error[v3] -= surface_error[i]; //Delete this face directly
                    edge_v.push(v3);
                }
            }
        });
        //Enumerate the faces adjacent to v2 - only type_a should be managed, because type_b was counted previously
        triangles_per_vertex[e.v2].iter().for_each(|&i| {
            let is_valid_triangle = if let Some(locked_indices) = locked_indices {
                is_triangle_valid_with_locked(&deleted_vertices, locked_indices, &tri, i)
            } else {
                is_triangle_valid(&deleted_vertices, &tri, i)
            };
            if is_valid_triangle && !is_in_triangle(&tri, i, e.v1) {
                //A triangle with only one vertex being e.v2
                if tri[i].1 == e.v2 {
                    tri[i] = (tri[i].1, tri[i].2, tri[i].0);
                } else if tri[i].2 == e.v2 {
                    tri[i] = (tri[i].2, tri[i].0, tri[i].1);
                }
                let (v2, v3) = (tri[i].1, tri[i].2);
                let qk = compute_quadric_error(&pos[v], &pos[v2], &pos[v3]);
                let dq = qk - surface_error[i];
                adiacency_v.push(i);
                vertices_error[v] += qk;
                vertices_error[v2] += dq;
                vertices_error[v3] += dq;
                surface_error[i] = qk;
                tri[i].0 = v;
                edge_v.push(v2);
                edge_v.push(v3);
            }
        });
        deleted_vertices[e.v1] = true;
        deleted_vertices[e.v2] = true;
        edge_v.iter().for_each(|&vi| {
            if !deleted_vertices[vi] {
                if let Some((qv, vr, cost)) = is_edge_valid(&pos, &vertices_error, v, vi) {
                    edge.push(Edge::new(v, vi, qv, vr, cost));
                    heap.push(State::new(cost, edge.len() - 1));
                }
                deleted_vertices[vi] = true;
            }
        });
        edge_v.iter().for_each(|&vi| {
            deleted_vertices[vi] = false;
        });
        triangles_per_vertex.push(adiacency_v);
        limit -= 2;
        if limit < 0 {
            break;
        }
    }

    let mut new_pos = Vec::with_capacity(vertices_count);
    let mut new_indices = Vec::with_capacity(indices.len());
    let mut cnt = 0;
    let mut id = vec![-1_i32; pos.len()];
    for i in 0..tri.len() {
        if is_triangle_valid(&deleted_vertices, &tri, i) {
            if id[tri[i].0] == -1 {
                id[tri[i].0] = cnt;
                new_pos.push(pos[tri[i].0]);
                cnt += 1;
            }
            new_indices.push(id[tri[i].0] as u32);
            if id[tri[i].1] == -1 {
                id[tri[i].1] = cnt;
                new_pos.push(pos[tri[i].1]);
                cnt += 1;
            }
            new_indices.push(id[tri[i].1] as u32);
            if id[tri[i].2] == -1 {
                id[tri[i].2] = cnt;
                new_pos.push(pos[tri[i].2]);
                cnt += 1;
            }
            new_indices.push(id[tri[i].2] as u32);
        }
    }
    new_indices
}
