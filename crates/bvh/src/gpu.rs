use inox_serialize::{Deserialize, Serialize};

use crate::{BVHTree, INVALID_INDEX};

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
    let mut linearized_bvh = Vec::<GPUBVHNode>::new();
    let bvh_nodes = bvh.nodes();
    let nodes_count = bvh_nodes.len();
    linearized_bvh.reserve(nodes_count);

    for i in 0..nodes_count {
        let node = &bvh_nodes[i];
        let is_leaf = node.is_leaf();

        linearized_bvh.push(GPUBVHNode {
            min: node.min().into(),
            max: node.max().into(),
            left_node: if is_leaf { -1 } else { node.left() },
            primitive_index: if is_leaf { node.aabb_index() } else { -1 },
        });
    }
    linearized_bvh
}

pub fn print_bvh(bvh: &[GPUBVHNode]) {
    println!("BVH {} - {}", 0, bvh.len());
    bvh.iter().enumerate().for_each(|(i, n)| {
        println!("  Node[{i}]:");
        println!("      Min -> {},{},{}", n.min[0], n.min[1], n.min[2]);
        println!("      Max -> {},{},{}", n.max[0], n.max[1], n.max[2]);
        println!("      Left -> {}", n.left_node);
        println!("      Prim [{}]", n.primitive_index);
    });
}
