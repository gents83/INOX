use inox_serialize::{Deserialize, Serialize};

use crate::{BVHBuildError, BVHTree, INVALID_INDEX};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinearizedBVHError {
    InvalidTree(BVHBuildError),
    InvalidEscape { index: usize, escape: i32 },
}

#[repr(C)]
#[derive(Serialize, Deserialize, Default, PartialEq, Clone, Copy, Debug)]
#[serde(crate = "inox_serialize")]
pub struct GPUBVHNode {
    pub min: [f32; 3],
    pub miss: i32,
    pub max: [f32; 3],
    pub reference: i32,
}

pub fn try_create_linearized_bvh(bvh: &BVHTree) -> Result<Vec<GPUBVHNode>, LinearizedBVHError> {
    bvh.validate().map_err(LinearizedBVHError::InvalidTree)?;
    let mut linearized_bvh = Vec::<GPUBVHNode>::new();
    let bvh_nodes = bvh.nodes();
    let nodes_count = bvh_nodes.len();
    let mut current_index = 0;
    while current_index < nodes_count {
        let parent_index = bvh_nodes[current_index].parent();
        let is_leaf = bvh_nodes[current_index].is_leaf();
        let mut linearized_node = GPUBVHNode {
            min: bvh_nodes[current_index].min().into(),
            max: bvh_nodes[current_index].max().into(),
            reference: INVALID_INDEX,
            miss: INVALID_INDEX,
        };
        linearized_node.miss = if parent_index >= 0 {
            if bvh_nodes[parent_index as usize].left() == current_index as i32 {
                bvh_nodes[parent_index as usize].right() as _
            } else {
                let ancestor_index = bvh_nodes[parent_index as usize].parent();
                if ancestor_index >= 0 && bvh_nodes[ancestor_index as usize].right() != parent_index
                {
                    bvh_nodes[ancestor_index as usize].right() as _
                } else {
                    linearized_bvh[parent_index as usize].miss
                }
            }
        } else {
            INVALID_INDEX
        };
        if is_leaf {
            linearized_node.reference = bvh_nodes[current_index].aabb_index();
        }
        linearized_bvh.push(linearized_node);
        current_index += 1;
    }
    validate_linearized_bvh(&linearized_bvh)?;
    Ok(linearized_bvh)
}

pub fn create_linearized_bvh(bvh: &BVHTree) -> Vec<GPUBVHNode> {
    try_create_linearized_bvh(bvh).expect("invalid BVH tree")
}

pub fn validate_linearized_bvh(nodes: &[GPUBVHNode]) -> Result<(), LinearizedBVHError> {
    for (index, node) in nodes.iter().enumerate() {
        let bounds_are_finite = node
            .min
            .iter()
            .chain(node.max.iter())
            .all(|value| value.is_finite());
        if !bounds_are_finite
            || node
                .min
                .iter()
                .zip(node.max.iter())
                .any(|(min, max)| min > max)
        {
            return Err(LinearizedBVHError::InvalidTree(
                BVHBuildError::InvalidBounds { index },
            ));
        }
        if (node.miss < INVALID_INDEX || node.miss >= nodes.len() as i32)
            && node.miss != INVALID_INDEX
        {
            return Err(LinearizedBVHError::InvalidEscape {
                index,
                escape: node.miss,
            });
        }
        if node.reference < INVALID_INDEX {
            return Err(LinearizedBVHError::InvalidTree(
                BVHBuildError::InvalidReference { index },
            ));
        }
    }
    Ok(())
}

pub fn print_bvh(bvh: &[GPUBVHNode]) {
    println!("BVH {} - {}", 0, bvh.len());
    bvh.iter().enumerate().for_each(|(i, n)| {
        println!("  Node[{i}]:");
        println!("      Min -> {},{},{}", n.min[0], n.min[1], n.min[2]);
        println!("      Max -> {},{},{}", n.max[0], n.max[1], n.max[2]);
        println!("      Miss -> {}", n.miss);
        println!("      Ref [{}]", n.reference);
    });
}
