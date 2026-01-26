use std::fmt::Debug;

use inox_math::Vector3;
use inox_serialize::{Deserialize, Serialize};

use crate::AABB;

#[repr(C)]
#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Default)]
#[serde(crate = "inox_serialize")]
pub struct BVHNode {
    pub min: [f32; 3],
    pub left_first: u32,
    pub max: [f32; 3],
    pub primitive_count: u32,
}

impl BVHNode {
    pub fn min(&self) -> Vector3 {
        self.min.into()
    }
    pub fn max(&self) -> Vector3 {
        self.max.into()
    }
    pub fn primitive_count(&self) -> u32 {
        self.primitive_count
    }
    pub fn is_leaf(&self) -> bool {
        self.primitive_count > 0
    }
}

#[derive(Serialize, Deserialize, Default, PartialEq, Clone)]
#[serde(crate = "inox_serialize")]
pub struct BVHTree {
    nodes: Vec<BVHNode>,
}

impl BVHTree {
    pub fn new(aabb_list: &[AABB]) -> Self {
        if aabb_list.is_empty() {
            return Self::default();
        }

        // Use 2 triangles per AABB to better approximate the centroid.
        // Triangle 1: min, max, min
        // Triangle 2: max, min, max
        // Vertices must be 4 floats.
        let mut vertices = Vec::with_capacity(aabb_list.len() * 6);
        for aabb in aabb_list {
            let min = aabb.min();
            let max = aabb.max();

            vertices.push([min.x, min.y, min.z, 1.0]);
            vertices.push([max.x, max.y, max.z, 1.0]);
            vertices.push([min.x, min.y, min.z, 1.0]);

            vertices.push([max.x, max.y, max.z, 1.0]);
            vertices.push([min.x, min.y, min.z, 1.0]);
            vertices.push([max.x, max.y, max.z, 1.0]);
        }

        let mut bvh = tinybvh_rs::bvh::BVH::new(vertices.as_slice().into()).unwrap();

        // Ensure at most 1 primitive per leaf so we can store a single index.
        bvh.split_leaves(1);

        let bvh_data = bvh.data();
        let tiny_nodes = bvh_data.nodes();
        let indices = bvh_data.indices();

        let nodes: Vec<BVHNode> = tiny_nodes
            .iter()
            .map(|n| {
                let mut node = BVHNode {
                    min: n.min,
                    left_first: n.left_first,
                    max: n.max,
                    primitive_count: n.tri_count,
                };

                if n.is_leaf() {
                    // Resolve indirection.
                    // left_first is the index into the `indices` array.
                    // indices[left_first] is the triangle index.
                    // We mapped AABB i to triangles 2*i and 2*i+1.
                    // So aabb_index = triangle_index / 2.
                    if n.tri_count > 0 {
                        let first_index = n.left_first as usize;
                        if first_index < indices.len() {
                            let tri_index = indices[first_index];
                            let aabb_index = tri_index / 2;
                            node.left_first = aabb_index;
                        } else {
                            // Should not happen if tinybvh is correct
                             debug_assert!(false, "Index out of bounds");
                        }
                    }
                }
                node
            })
            .collect();

        Self { nodes }
    }

    pub fn nodes(&self) -> &[BVHNode] {
        &self.nodes
    }
}

impl Debug for BVHTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BVH:\n").ok();
        self.nodes.iter().enumerate().for_each(|(i, n)| {
            f.write_fmt(format_args!("  Node[{i}]:\n")).ok();
            f.write_fmt(format_args!("      Min -> {:?}\n", n.min)).ok();
            f.write_fmt(format_args!("      Max -> {:?}\n", n.max)).ok();
            f.write_fmt(format_args!("      LeftFirst -> {}\n", n.left_first)).ok();
            f.write_fmt(format_args!("      Count -> {}\n", n.primitive_count)).ok();
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{BVHTree, AABB};
    use inox_math::Vector3;

    #[test]
    fn test_bvh_build() {
        let min = Vector3::new(0.0, 0.0, 0.0);
        let max = Vector3::new(1.0, 1.0, 1.0);
        let aabb = AABB::create(min, max, 0);

        let bvh = BVHTree::new(&[aabb]);
        assert!(!bvh.nodes().is_empty());

        // Verify leaf resolution
        let leaf = bvh.nodes().iter().find(|n| n.is_leaf()).expect("Should have a leaf");
        assert_eq!(leaf.left_first, 0); // Should resolve to AABB 0
        println!("{:?}", bvh);
    }
}
