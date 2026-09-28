use std::{cmp::Ordering, fmt::Debug};

use inox_math::Vector3;
use inox_serialize::{Deserialize, Serialize};

use crate::AXIS_COUNT;

use super::aabb::AABB;

pub const INVALID_NODE: i32 = -1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BVHBuildError {
    InvalidBounds { index: usize },
    InvalidReference { index: usize },
    TooManyNodes,
}

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
        Self::try_new(aabb_list).expect("invalid BVH input")
    }

    pub fn try_new(aabb_list: &[AABB]) -> Result<Self, BVHBuildError> {
        if aabb_list.len() > (i32::MAX as usize).div_ceil(2) {
            return Err(BVHBuildError::TooManyNodes);
        }
        for (index, aabb) in aabb_list.iter().enumerate() {
            if !aabb.is_valid() {
                return Err(BVHBuildError::InvalidBounds { index });
            }
            if aabb.index() < 0 {
                return Err(BVHBuildError::InvalidReference { index });
            }
        }
        if aabb_list.is_empty() {
            return Ok(Self::default());
        }
        let mut nodes: Vec<BVHNode> = aabb_list
            .iter()
            .enumerate()
            .map(|(i, aabb)| {
                BVHNode::create(
                    aabb,
                    if aabb_list.len() == 1 {
                        INVALID_NODE
                    } else {
                        i as i32
                    },
                )
            })
            .collect();
        Self::build_tree(&mut nodes, 0, aabb_list.len());
        Ok(Self::reverse(BVHTree { nodes }))
    }

    fn build_tree(nodes: &mut Vec<BVHNode>, start: usize, end: usize) -> u32 {
        if end - start <= 1 {
            return start as _;
        }

        // The recursive ranges contain the original leaves. Sort that range in
        // place; the old implementation sorted a temporary index list and then
        // ignored it, producing partitions unrelated to the selected axis.
        let indices: Vec<usize> = (start..end).collect();
        let (axis, split) = Self::choose_split_sah(nodes, &indices);
        nodes[start..end].sort_by(|a, b| {
            a.aabb.center()[axis]
                .partial_cmp(&b.aabb.center()[axis])
                .unwrap_or(Ordering::Equal)
        });

        let mid = start + split;
        let left_child = Self::build_tree(nodes, start, mid);
        let right_child = Self::build_tree(nodes, mid, end);
        let parent = nodes.len() as u32;
        let aabb = AABB::compute_aabb(&[
            nodes[left_child as usize].aabb,
            nodes[right_child as usize].aabb,
        ]);
        nodes.push(BVHNode {
            aabb,
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
    fn choose_split_sah(nodes: &[BVHNode], indices: &[usize]) -> (usize, usize) {
        let fallback = || {
            let axis = Self::choose_split_axis_centroid(nodes, indices);
            (axis, indices.len() / 2)
        };
        if indices.len() < 2 {
            return fallback();
        }

        let mut best = None;
        for axis in 0..AXIS_COUNT {
            let min_value = indices
                .iter()
                .map(|&index| nodes[index].aabb.min()[axis])
                .fold(f32::INFINITY, f32::min);
            let max_value = indices
                .iter()
                .map(|&index| nodes[index].aabb.max()[axis])
                .fold(f32::NEG_INFINITY, f32::max);
            if !min_value.is_finite() || !max_value.is_finite() || max_value <= min_value {
                continue;
            }
            let mut sorted = indices.to_vec();
            sorted.sort_by(|&a, &b| {
                nodes[a].aabb.center()[axis]
                    .partial_cmp(&nodes[b].aabb.center()[axis])
                    .unwrap_or(Ordering::Equal)
            });
            let mut prefix = Vec::with_capacity(sorted.len());
            for (position, &index) in sorted.iter().enumerate() {
                let bounds = if position == 0 {
                    nodes[index].aabb
                } else {
                    AABB::compute_aabb(&[prefix[position - 1], nodes[index].aabb])
                };
                prefix.push(bounds);
            }
            let mut suffix = vec![AABB::empty(); sorted.len()];
            for position in (0..sorted.len()).rev() {
                suffix[position] = if position + 1 == sorted.len() {
                    nodes[sorted[position]].aabb
                } else {
                    AABB::compute_aabb(&[suffix[position + 1], nodes[sorted[position]].aabb])
                };
            }
            for split in 1..sorted.len() {
                let left = prefix[split - 1].surface_area();
                let right = suffix[split].surface_area();
                let cost = left * split as f32 + right * (sorted.len() - split) as f32;
                if cost.is_finite() && best.is_none_or(|(_, _, best_cost)| cost < best_cost) {
                    best = Some((axis, split, cost));
                }
            }
        }
        best.map(|(axis, split, _)| (axis, split))
            .unwrap_or_else(fallback)
    }

    pub fn nodes(&self) -> &[BVHNode] {
        &self.nodes
    }
    pub fn nodes_mut(&mut self) -> &mut [BVHNode] {
        &mut self.nodes
    }

    pub fn validate(&self) -> Result<(), BVHBuildError> {
        if self.nodes.is_empty() {
            return Ok(());
        }
        if self.nodes[0].parent != INVALID_NODE {
            return Err(BVHBuildError::InvalidReference { index: 0 });
        }
        let mut visited = vec![false; self.nodes.len()];
        let mut stack = vec![0usize];
        while let Some(index) = stack.pop() {
            if index >= self.nodes.len() || visited[index] {
                return Err(BVHBuildError::InvalidReference { index });
            }
            visited[index] = true;
            let node = &self.nodes[index];
            if !node.aabb.is_valid() {
                return Err(BVHBuildError::InvalidBounds { index });
            }
            if node.is_leaf() {
                if node.aabb.index() < 0 {
                    return Err(BVHBuildError::InvalidReference { index });
                }
                continue;
            }
            if node.left < 0 || node.right < 0 {
                return Err(BVHBuildError::InvalidReference { index });
            }
            let left = node.left as usize;
            let right = node.right as usize;
            if left >= self.nodes.len() || right >= self.nodes.len() || left == right {
                return Err(BVHBuildError::InvalidReference { index });
            }
            if self.nodes[left].parent != index as i32 || self.nodes[right].parent != index as i32 {
                return Err(BVHBuildError::InvalidReference { index });
            }
            stack.push(left);
            stack.push(right);
        }
        if visited.iter().any(|visited| !visited) {
            return Err(BVHBuildError::InvalidReference { index: 0 });
        }
        Ok(())
    }

    pub fn reverse(tree: Self) -> Self {
        if tree.nodes.is_empty() {
            return tree;
        }
        let mut index_map = vec![INVALID_NODE; tree.nodes.len()];
        let mut nodes = Vec::with_capacity(tree.nodes.len());
        let mut nodes_to_visit = vec![tree.nodes.len() - 1];
        while let Some(old_index) = nodes_to_visit.pop() {
            index_map[old_index] = nodes.len() as i32;
            nodes.push(tree.nodes[old_index]);
            if !tree.nodes[old_index].is_leaf() {
                // Push right first so the left child is visited first.
                nodes_to_visit.push(tree.nodes[old_index].right() as usize);
                nodes_to_visit.push(tree.nodes[old_index].left() as usize);
            }
        }
        for node in &mut nodes {
            node.left = if node.left >= 0 {
                index_map[node.left as usize]
            } else {
                INVALID_NODE
            };
            node.right = if node.right >= 0 {
                index_map[node.right as usize]
            } else {
                INVALID_NODE
            };
            node.parent = if node.parent >= 0 {
                index_map[node.parent as usize]
            } else {
                INVALID_NODE
            };
        }
        Self { nodes }
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
