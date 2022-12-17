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
        }
        tree
    }
    pub fn nodes(&self) -> &[BHVNode] {
        &self.nodes
    }
    fn add(&mut self, parent_aabb: &AABB, parent_index: usize, list: &[AABB]) {
        let (left_group, right_group) = Partition::compute(parent_aabb, list);

        let left_aabb = AABB::compute_aabb(&left_group);
        let right_aabb = AABB::compute_aabb(&right_group);

        let node = BHVNode::create(&left_aabb, parent_index as _);
        self.nodes.push(node);
        let left_index = self.nodes.len() - 1;
        self.nodes[parent_index].left = left_index as _;

        let node = BHVNode::create(&right_aabb, parent_index as _);
        self.nodes.push(node);
        let right_index = self.nodes.len() - 1;
        self.nodes[parent_index].right = right_index as _;

        if left_group.len() > 1 {
            self.add(&left_aabb, left_index, &left_group);
        }
        if right_group.len() > 1 {
            self.add(&right_aabb, right_index, &right_group);
        }
    }
}
