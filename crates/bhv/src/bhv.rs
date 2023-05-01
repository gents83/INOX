use std::fmt::Debug;

use inox_math::Vector3;

use crate::Partition;

use super::aabb::AABB;

const INVALID_NODE: i32 = -1;

#[derive(Clone, Copy)]
pub struct BHVNode {
    aabb: AABB,
    left: u32,
    right: u32,
    parent: i32,
}

impl BHVNode {
    pub fn create(aabb: &AABB, parent: i32) -> Self {
        Self {
            aabb: *aabb,
            left: 0,
            right: 0,
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
    pub fn right(&self) -> u32 {
        self.right
    }
    pub fn left(&self) -> u32 {
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

impl Default for BHVNode {
    fn default() -> Self {
        let aabb = AABB::empty();
        Self::create(&aabb, INVALID_NODE)
    }
}

#[derive(Default, Clone)]
pub struct BHVTree {
    nodes: Vec<BHVNode>,
}

impl BHVTree {
    pub fn new(list: &[AABB]) -> Self {
        let mut tree = Self::default();
        let aabb = AABB::compute_aabb(list);
        let root_node = BHVNode::create(&aabb, INVALID_NODE);
        tree.nodes.push(root_node);
        if list.len() > 1 {
            tree.add(&aabb, tree.nodes.len() - 1, list);
        } else {
            tree.nodes[0].set_aabb_index(0);
        }
        tree
    }
    pub fn nodes(&self) -> &[BHVNode] {
        &self.nodes
    }
    pub fn nodes_mut(&mut self) -> &mut [BHVNode] {
        &mut self.nodes
    }
    fn add(&mut self, parent_aabb: &AABB, parent_index: usize, list: &[AABB]) {
        let (mut left_group, mut right_group) = Partition::compute_sah(parent_aabb, list);

        let left_aabb = if left_group.len() > 1 {
            AABB::compute_aabb(&left_group)
        } else {
            left_group.pop().unwrap()
        };

        let node = BHVNode::create(&left_aabb, parent_index as _);
        self.nodes.push(node);
        let left_index = self.nodes.len() - 1;
        self.nodes[parent_index].left = left_index as _;

        if left_group.len() > 1 {
            self.add(&left_aabb, left_index, &left_group);
        }

        let right_aabb = if right_group.len() > 1 {
            AABB::compute_aabb(&right_group)
        } else {
            right_group.pop().unwrap()
        };

        let node = BHVNode::create(&right_aabb, parent_index as _);
        self.nodes.push(node);
        let right_index = self.nodes.len() - 1;
        self.nodes[parent_index].right = right_index as _;

        if right_group.len() > 1 {
            self.add(&right_aabb, right_index, &right_group);
        }
    }
    pub fn insert_at(&mut self, position: usize, tree: BHVTree) -> &mut Self {
        if position < self.nodes.len() {
            let mut index = position;
            for i in 0..tree.nodes.len() {
                let mut node = tree.nodes[i];
                node.parent += position as i32;
                if node.left > 0 {
                    node.left += position as u32;
                }
                if node.right > 0 {
                    node.right += position as u32;
                }
                if i == 0 {
                    node.parent = self.nodes[index].parent;
                    self.nodes.remove(position);
                }
                self.nodes.insert(index, node);
                index += 1;
            }
            let nodes_offset = tree.nodes.len() - 1;
            for i in index..self.nodes.len() {
                if self.nodes[i].parent > position as i32 {
                    self.nodes[i].parent += nodes_offset as i32;
                }
                if self.nodes[i].left > position as u32 {
                    self.nodes[i].left += nodes_offset as u32;
                }
                if self.nodes[i].right > position as u32 {
                    self.nodes[i].right += nodes_offset as u32;
                }
            }
        }
        self
    }
}

impl Debug for BHVTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BHV:\n").ok();
        self.nodes.iter().enumerate().for_each(|(i, n)| {
            f.write_fmt(format_args!("  Node[{i}]:\n")).ok();
            f.write_fmt(format_args!("      Left -> {}\n", n.left)).ok();
            f.write_fmt(format_args!("      Right -> {}\n", n.right))
                .ok();
        });
        Ok(())
    }
}
