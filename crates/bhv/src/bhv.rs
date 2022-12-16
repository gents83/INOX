use inox_math::Vector3;

use crate::SAHPartition;

use super::aabb::AABB;

const INVALID_NODE: i32 = -1;

#[derive(Clone, Copy)]
struct BHVNode {
    min: Vector3,
    parent: i32,
    max: Vector3,
    children: u32, // 16 bit left + 16 bit right - 0 means no children
}

impl BHVNode {
    pub fn create(aabb: &AABB, parent: i32) -> Self {
        Self {
            min: aabb.min(),
            max: aabb.max(),
            parent,
            children: 0,
        }
    }
    pub fn has_left(&self) -> bool {
        ((self.children & 0xFFFF0000) >> 16) > 0
    }
    pub fn has_right(&self) -> bool {
        (self.children & 0x0000FFFF) > 0
    }
}

impl Default for BHVNode {
    fn default() -> Self {
        let aabb = AABB::empty();
        Self::create(&aabb, INVALID_NODE)
    }
}

#[derive(Default, Clone)]
struct BHVTree {
    nodes: Vec<BHVNode>,
}

impl BHVTree {
    pub fn new(list: &[AABB]) -> Self {
        let mut tree = Self::default();
        let aabb = AABB::compute_aabb(list);
        let root_node = BHVNode::create(&aabb, INVALID_NODE);
        tree.nodes.push(root_node);
        tree.add(&aabb, tree.nodes.len(), list);
        tree
    }
    fn add(&mut self, parent_aabb: &AABB, parent_index: usize, list: &[AABB]) {
        let (left_group, right_group) = SAHPartition::compute(parent_aabb, list);
        let left_aabb = AABB::compute_aabb(&left_group);
        let right_aabb = AABB::compute_aabb(&right_group);
        let mut left_index = 0;
        let mut right_index = 0;
        if !left_group.is_empty() {
            let node = BHVNode::create(&left_aabb, parent_index as _);
            self.nodes.push(node);
            left_index = self.nodes.len();
            self.nodes[parent_index].children |= (left_index as u32) << 16;
        }
        if !right_group.is_empty() {
            let node = BHVNode::create(&right_aabb, parent_index as _);
            self.nodes.push(node);
            right_index = self.nodes.len();
            self.nodes[parent_index].children |= right_index as u32;
        }
        if left_index > 0 {
            self.add(&left_aabb, left_index, &left_group);
        }
        if right_index > 0 {
            self.add(&right_aabb, left_index, &left_group);
        }
    }
}
