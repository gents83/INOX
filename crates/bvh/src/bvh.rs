use std::{cmp::Ordering, fmt::Debug};

use inox_math::Vector3;
use inox_serialize::{Deserialize, Serialize};

use crate::AXIS_COUNT;

use super::aabb::AABB;

const INVALID_NODE: i32 = -1;

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy)]
#[serde(crate = "inox_serialize")]
pub struct BVHNode {
    aabb: AABB,
    left: i32,
    right: i32,
    parent: i32,
}

impl BVHNode {
    pub fn create(aabb: &AABB, parent: i32) -> Self {
        Self {
            aabb: *aabb,
            left: INVALID_NODE,
            right: INVALID_NODE,
            parent,
        }
    }
    pub fn min(&self) -> Vector3 {
        self.aabb.min()
    }
    pub fn max(&self) -> Vector3 {
        self.aabb.max()
    }
    pub fn aabb_index(&self) -> i32 {
        self.aabb.index()
    }
    pub fn set_aabb_index(&mut self, index: u32) -> &mut Self {
        self.aabb.set_index(index);
        self
    }
    pub fn right(&self) -> i32 {
        self.right
    }
    pub fn left(&self) -> i32 {
        self.left
    }
    pub fn parent(&self) -> i32 {
        self.parent
    }
    pub fn is_leaf(&self) -> bool {
        self.left == self.right
    }
    pub fn is_equal(&self, aabb: &AABB) -> bool {
        self.aabb.index() == aabb.index() && self.aabb.index() > 0
    }
}

impl Default for BVHNode {
    fn default() -> Self {
        let aabb = AABB::empty();
        Self::create(&aabb, INVALID_NODE)
    }
}

#[derive(Serialize, Deserialize, Default, PartialEq, Clone)]
#[serde(crate = "inox_serialize")]
pub struct BVHTree {
    nodes: Vec<BVHNode>,
}
impl BVHTree {
    pub fn new(aabb_list: &[AABB]) -> Self {
        let mut nodes: Vec<BVHNode> = aabb_list
            .iter()
            .enumerate()
            .map(|(i, aabb)| BVHNode::create(aabb, i as i32))
            .collect();
        Self::build_tree(&mut nodes, 0, aabb_list.len());
        let reversed = BVHTree { nodes };
        Self::reverse(reversed)
    }

    fn build_tree(nodes: &mut Vec<BVHNode>, start: usize, end: usize) -> u32 {
        if end - start <= 1 {
            return start as _;
        }

        let mut indices: Vec<usize> = (start..end).collect();
        let axis = Self::choose_split_axis_sah(nodes, &indices);
        indices.sort_by(|&a, &b| {
            nodes[a].aabb.center()[axis]
                .partial_cmp(&nodes[b].aabb.center()[axis])
                .unwrap_or(Ordering::Equal)
        });

        let mid = (start + end) / 2;
        let left_child = Self::build_tree(nodes, start, mid);
        let right_child = Self::build_tree(nodes, mid, end);

        let parent = nodes.len() as u32;
        nodes.push(BVHNode {
            aabb: AABB::compute_aabb(&nodes.iter().map(|&n| n.aabb).collect::<Vec<_>>()),
            left: left_child as _,
            right: right_child as _,
            parent: INVALID_NODE,
        });

        nodes[left_child as usize].parent = parent as i32;
        nodes[right_child as usize].parent = parent as i32;

        parent
    }

    fn choose_split_axis_centroid(nodes: &[BVHNode], indices: &[usize]) -> usize {
        // Implement your logic for choosing the split axis (e.g., based on SAH).
        // This is a placeholder; you may need to replace it with a proper implementation.
        // For simplicity, it currently chooses the axis with the maximum extent.
        let mut max_extent = 0.0;
        let mut split_axis = 0;

        for axis in 0..AXIS_COUNT {
            let min_value = indices
                .iter()
                .map(|&i| nodes[i].aabb.min()[axis])
                .fold(f32::INFINITY, f32::min);
            let max_value = indices
                .iter()
                .map(|&i| nodes[i].aabb.max()[axis])
                .fold(f32::NEG_INFINITY, f32::max);

            let extent = max_value - min_value;
            if extent > max_extent {
                max_extent = extent;
                split_axis = axis;
            }
        }

        split_axis
    }
    fn choose_split_axis_sah(nodes: &[BVHNode], indices: &[usize]) -> usize {
        let mut best_axis = 0;
        let mut best_cost = f32::INFINITY;

        for axis in 0..AXIS_COUNT {
            let min_value = indices
                .iter()
                .map(|&i| nodes[i].aabb.min()[axis])
                .fold(f32::INFINITY, f32::min);
            let max_value = indices
                .iter()
                .map(|&i| nodes[i].aabb.max()[axis])
                .fold(f32::NEG_INFINITY, f32::max);

            let bin_width = (max_value - min_value) / (indices.len() as f32);

            let mut bins_right = vec![0; indices.len()];

            let mut surface_area_left = 0.0;
            let mut surface_area_right = 0.0;

            for &i in indices {
                surface_area_right += nodes[i].aabb.surface_area();
            }

            let mut right_count = 0;

            for &i in indices {
                // Calculate bin index ensuring it's within bounds
                let bin = ((nodes[i].aabb.min()[axis] - min_value) / bin_width)
                    .clamp(0.0, bins_right.len() as f32 - 1.0) as usize;
                bins_right[bin] += 1;

                // Rest of the code remains the same
                surface_area_right -= nodes[i].aabb.surface_area();
                surface_area_left += nodes[i].aabb.surface_area();

                if right_count < indices.len() - 1 {
                    right_count += 1;
                }

                if right_count > 0 {
                    let cost = (surface_area_left + surface_area_right) * bin_width;

                    if cost < best_cost {
                        best_cost = cost;
                        best_axis = axis;
                    }
                }
            }
        }

        best_axis
    }

    pub fn nodes(&self) -> &[BVHNode] {
        &self.nodes
    }
    pub fn nodes_mut(&mut self) -> &mut [BVHNode] {
        &mut self.nodes
    }

    pub fn reverse(tree: Self) -> Self {
        let mut index_map = vec![0; tree.nodes.len()];
        let mut new_bvh = BVHTree::default();
        let mut nodes_to_visit = [tree.nodes.len() - 1].to_vec();
        while !nodes_to_visit.is_empty() {
            let old_index = nodes_to_visit.remove(0);
            index_map[old_index] = new_bvh.nodes.len();
            new_bvh.nodes.push(tree.nodes[old_index]);
            if !tree.nodes[old_index].is_leaf() {
                nodes_to_visit.insert(0, tree.nodes[old_index].right() as _);
                nodes_to_visit.insert(0, tree.nodes[old_index].left() as _);
            }
        }
        let entries_count = tree.nodes.len();
        let mut left_modified = vec![false; entries_count];
        let mut right_modified = vec![false; entries_count];
        let mut parent_modified = vec![false; entries_count];
        new_bvh.nodes.iter_mut().enumerate().for_each(|(i, node)| {
            (0..index_map.len()).for_each(|old_index| {
                if !left_modified[i] && node.left == old_index as i32 {
                    node.left = index_map[old_index] as i32;
                    left_modified[i] = true;
                }
                if !right_modified[i] && node.right == old_index as i32 {
                    node.right = index_map[old_index] as i32;
                    right_modified[i] = true;
                }
                if !parent_modified[i] && node.parent == old_index as i32 {
                    node.parent = index_map[old_index] as i32;
                    parent_modified[i] = true;
                }
            });
        });
        Self {
            nodes: new_bvh.nodes,
        }
    }
}

impl Debug for BVHTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BVH:\n").ok();
        self.nodes.iter().enumerate().for_each(|(i, n)| {
            f.write_fmt(format_args!("  Node[{i}]:\n")).ok();
            f.write_fmt(format_args!("      Left -> {}\n", n.left)).ok();
            f.write_fmt(format_args!("      Right -> {}\n", n.right))
                .ok();
            f.write_fmt(format_args!("      Parent -> {}\n", n.parent))
                .ok();
            f.write_fmt(format_args!("      Ref -> {}\n", n.aabb_index()))
                .ok();
        });
        Ok(())
    }
}

fn test_bvh_quad() {
    use inox_math::VecBase;

    struct Triangle {
        vertices: [Vector3; 3],
    }
    let triangles = [
        Triangle {
            vertices: [
                [0., 0., 0.].into(),
                [0., 1., 0.].into(),
                [1., 1., 0.].into(),
            ],
        },
        Triangle {
            vertices: [
                [1., 1., 0.].into(),
                [0., 0., 0.].into(),
                [1., 0., 0.].into(),
            ],
        },
    ];
    let min_v1 = triangles[0]
        .vertices
        .iter()
        .fold(Vector3::new(f32::MAX, f32::MAX, f32::MAX), |i, v| v.min(i));
    let max_v1 = triangles[0]
        .vertices
        .iter()
        .fold(Vector3::new(-f32::MAX, -f32::MAX, -f32::MAX), |i, v| {
            v.max(i)
        });
    let min_v2 = triangles[1]
        .vertices
        .iter()
        .fold(Vector3::new(f32::MAX, f32::MAX, f32::MAX), |i, v| v.min(i));
    let max_v2 = triangles[1]
        .vertices
        .iter()
        .fold(Vector3::new(-f32::MAX, -f32::MAX, -f32::MAX), |i, v| {
            v.max(i)
        });
    let aabbs = [
        AABB::create(min_v1, max_v1, 0),
        AABB::create(min_v2, max_v2, 1),
    ];
    let bvh = BVHTree::new(&aabbs);
    println!("{:?}", bvh);

    debug_assert!(bvh.nodes().len() == 3);

    debug_assert!(bvh.nodes()[0].min() == min_v1.min(min_v2));
    debug_assert!(bvh.nodes()[0].max() == max_v1.max(max_v2));
    debug_assert!(bvh.nodes()[0].parent() == -1);
    debug_assert!(bvh.nodes()[0].aabb_index() == -1);
    debug_assert!(bvh.nodes()[0].left() == 1);
    debug_assert!(bvh.nodes()[0].right() == 2);

    debug_assert!(bvh.nodes()[1].min() == min_v1);
    debug_assert!(bvh.nodes()[1].max() == max_v1);
    debug_assert!(bvh.nodes()[1].parent() == 0);
    debug_assert!(bvh.nodes()[1].aabb_index() == 0);
    debug_assert!(bvh.nodes()[1].left() == -1);
    debug_assert!(bvh.nodes()[1].right() == -1);

    debug_assert!(bvh.nodes()[2].min() == min_v2);
    debug_assert!(bvh.nodes()[2].max() == max_v2);
    debug_assert!(bvh.nodes()[2].parent() == 0);
    debug_assert!(bvh.nodes()[2].aabb_index() == 1);
    debug_assert!(bvh.nodes()[2].left() == -1);
    debug_assert!(bvh.nodes()[2].right() == -1);
}

fn test_bvh_cube() {
    use inox_math::VecBase;

    let min: Vector3 = [-100., -100., -100.].into();
    let max: Vector3 = [100., 100., 100.].into();
    let vertices: [Vector3; 8] = [
        Vector3::new(min.x, min.y, min.z),
        Vector3::new(max.x, min.y, min.z),
        Vector3::new(max.x, max.y, min.z),
        Vector3::new(min.x, max.y, min.z),
        Vector3::new(min.x, min.y, max.z),
        Vector3::new(max.x, min.y, max.z),
        Vector3::new(max.x, max.y, max.z),
        Vector3::new(min.x, max.y, max.z),
    ];
    let indices = [
        0, 1, 3, 3, 1, 2, 1, 5, 2, 2, 5, 6, 5, 4, 6, 6, 4, 7, 4, 0, 7, 7, 0, 3, 3, 2, 7, 7, 2, 6,
        4, 5, 0, 0, 5, 1,
    ];
    let mut i = 0;
    let mut aabbs = Vec::new();
    while i < indices.len() {
        let v1 = vertices[indices[i]];
        i += 1;
        let v2 = vertices[indices[i]];
        i += 1;
        let v3 = vertices[indices[i]];
        i += 1;
        let t_min = v1.min(v2.min(v3));
        let t_max = v1.max(v2.max(v3));
        aabbs.push(AABB::create(t_min, t_max, ((i - 1) / 3) as _));
    }
    let bvh = BVHTree::new(&aabbs);
    println!("{:?}", bvh);
}

#[test]
fn test_bvh() {
    test_bvh_quad();
    test_bvh_cube();
}
