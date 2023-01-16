use std::ops::Range;

use inox_bhv::BHVTree;
use inox_math::Vector4;

use crate::{DrawBHVNode, INVALID_INDEX};

#[inline]
pub fn compute_color_from_id(id: u32) -> Vector4 {
    let r = ((id & 0xFF) as f32) / 255.;
    let g = ((id >> 8) & 0xFF) as f32 / 255.;
    let b = ((id >> 16) & 0xFF) as f32 / 255.;
    let a = ((id >> 24) & 0xFF) as f32 / 255.;
    Vector4::new(r, g, b, a)
}

#[inline]
pub fn compute_id_from_color(color: Vector4) -> u32 {
    let color = color * 255.;
    (color.x as u32) | (color.y as u32) << 8 | (color.z as u32) << 16 | (color.w as u32) << 24
}

pub fn create_linearized_bhv(bhv: &BHVTree) -> Vec<DrawBHVNode> {
    inox_profiler::scoped_profile!("create_linearized_bhv");

    let mut linearized_bhv = Vec::<DrawBHVNode>::new();
    let bhv_nodes = bhv.nodes();
    let nodes_count = bhv_nodes.len();
    let mut current_index = 0;
    while current_index < nodes_count {
        let parent_index = bhv_nodes[current_index].parent();
        let is_leaf = bhv_nodes[current_index].is_leaf();
        let mut linearized_node = DrawBHVNode {
            min: bhv_nodes[current_index].min().into(),
            max: bhv_nodes[current_index].max().into(),
            reference: INVALID_INDEX,
            miss: INVALID_INDEX,
        };
        linearized_node.miss = if parent_index >= 0 {
            if bhv_nodes[parent_index as usize].left() == current_index as u32 {
                bhv_nodes[parent_index as usize].right() as _
            } else {
                let ancestor_index = bhv_nodes[parent_index as usize].parent();
                if ancestor_index >= 0
                    && bhv_nodes[ancestor_index as usize].right() != parent_index as u32
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

pub fn print_bhv(bhv: &[DrawBHVNode], bhv_range: &Range<usize>) {
    println!("BHV {} - {}", bhv_range.start, bhv_range.end + 1);
    bhv.iter().enumerate().for_each(|(i, n)| {
        println!("  Node[{}]:", i);
        println!("      Miss -> {}", n.miss);
        println!("      Ref [{}]", n.reference);
    });
}
