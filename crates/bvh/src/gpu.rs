use inox_serialize::{Deserialize, Serialize};

use crate::BVHTree;

#[repr(C)]
#[derive(Serialize, Deserialize, Default, PartialEq, Clone, Copy, Debug)]
#[serde(crate = "inox_serialize")]
pub struct GPUBVHNode {
    pub min: [f32; 3],
    pub left_node: i32,
    pub max: [f32; 3],
    pub primitive_index: i32,
}

pub fn create_linearized_bvh(bvh: &BVHTree) -> Vec<GPUBVHNode> {
    let bvh_nodes = bvh.nodes();
    let mut linearized_bvh = Vec::with_capacity(bvh_nodes.len());

    for node in bvh_nodes {
        let is_leaf = node.primitive_count > 0;

        let mut gpu_node = GPUBVHNode {
            min: node.min,
            left_node: -1,
            max: node.max,
            primitive_index: -1,
        };

        if is_leaf {
            gpu_node.primitive_index = node.left_first as i32;
        } else {
            gpu_node.left_node = node.left_first as i32;
        }

        linearized_bvh.push(gpu_node);
    }

    linearized_bvh
}

pub fn print_bvh(bvh: &[GPUBVHNode]) {
    println!("BVH {} - {}", 0, bvh.len());
    bvh.iter().enumerate().for_each(|(i, n)| {
        println!("  Node[{i}]:");
        println!("      Min -> {},{},{}", n.min[0], n.min[1], n.min[2]);
        println!("      Max -> {},{},{}", n.max[0], n.max[1], n.max[2]);
        println!("      LeftNode -> {}", n.left_node);
        println!("      PrimitiveIndex [{}]", n.primitive_index);
    });
}
