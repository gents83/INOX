use inox_math::Vector3;

use crate::Partition;

use super::aabb::AABB;

const INVALID_NODE: i32 = -1;

#[derive(Clone, Copy)]
pub struct BHVNode {
    min: Vector3,
    left: u32,
    max: Vector3,
    right: u32,
}

impl BHVNode {
    pub fn create(aabb: &AABB) -> Self {
        Self {
            min: aabb.min(),
            max: aabb.max(),
            left: 0,
            right: 0,
        }
    }
    pub fn has_left(&self) -> bool {
        self.left != 0
    }
    pub fn has_right(&self) -> bool {
        self.right != 0
    }
    pub fn is_equal(&self, aabb: &AABB) -> bool {
        self.min == aabb.min() && self.max == aabb.max()
    }
}

impl Default for BHVNode {
    fn default() -> Self {
        let aabb = AABB::empty();
        Self::create(&aabb)
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
        let root_node = BHVNode::create(&aabb);
        tree.nodes.push(root_node);
        if list.len() > 1 {
            tree.add(&aabb, tree.nodes.len(), list);
        }
        tree
    }
    pub fn nodes(&self) -> &[BHVNode] {
        &self.nodes
    }
    fn add(&mut self, parent_aabb: &AABB, parent_index: usize, list: &[AABB]) {
        if parent_index > 1000 || parent_index == 99 {
            println!("eccolo");
        }
        let (left_group, right_group) = Partition::compute(parent_aabb, list);

        let left_aabb = AABB::compute_aabb(&left_group);
        let right_aabb = AABB::compute_aabb(&right_group);

        let node = BHVNode::create(&left_aabb);
        self.nodes.push(node);
        let left_index = self.nodes.len();
        self.nodes[parent_index].left = left_index as _;

        let node = BHVNode::create(&right_aabb);
        self.nodes.push(node);
        let right_index = self.nodes.len();
        self.nodes[parent_index].right = right_index as _;

        if left_group.len() > 1 {
            self.add(&left_aabb, left_index, &left_group);
        }
        if right_group.len() > 1 {
            self.add(&right_aabb, right_index, &right_group);
        }
    }
}
