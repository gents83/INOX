use inox_serialize::{Deserialize, Serialize};

use crate::{BVHTree, INVALID_INDEX};

#[repr(C, align(16))]
#[derive(Serialize, Deserialize, Default, PartialEq, Clone, Copy, Debug)]
#[serde(crate = "inox_serialize")]
pub struct GPUBVHNode {
    pub min: [f32; 3],
    pub miss: i32,
    pub max: [f32; 3],
    pub reference: i32,
}

pub fn create_linearized_bhv(bhv: &BVHTree) -> Vec<GPUBVHNode> {
    let mut linearized_bhv = Vec::<GPUBVHNode>::new();
    let bhv_nodes = bhv.nodes();
    let nodes_count = bhv_nodes.len();
    let mut current_index = 0;
    while current_index < nodes_count {
        let parent_index = bhv_nodes[current_index].parent();
        let is_leaf = bhv_nodes[current_index].is_leaf();
        let mut linearized_node = GPUBVHNode {
            min: bhv_nodes[current_index].min().into(),
            max: bhv_nodes[current_index].max().into(),
            reference: INVALID_INDEX,
            miss: INVALID_INDEX,
        };
        linearized_node.miss = if parent_index >= 0 {
            if bhv_nodes[parent_index as usize].left() == current_index as i32 {
                bhv_nodes[parent_index as usize].right() as _
            } else {
                let ancestor_index = bhv_nodes[parent_index as usize].parent();
                if ancestor_index >= 0 && bhv_nodes[ancestor_index as usize].right() != parent_index
                {
                    bhv_nodes[ancestor_index as usize].right() as _
                } else {
                    linearized_bhv[parent_index as usize].miss
                }
            }
        } else {
            INVALID_INDEX
        };
        if is_leaf {
            linearized_node.reference = bhv_nodes[current_index].aabb_index();
        }
        linearized_bhv.push(linearized_node);
        current_index += 1;
    }
    linearized_bhv
}

pub fn print_bhv(bhv: &[GPUBVHNode]) {
    println!("BHV {} - {}", 0, bhv.len());
    bhv.iter().enumerate().for_each(|(i, n)| {
        println!("  Node[{i}]:");
        println!("      Miss -> {}", n.miss);
        println!("      Ref [{}]", n.reference);
    });
}
