use inox_math::Vector3;

use crate::{AABB, AXIS_COUNT};

const SPLIT_COUNT: usize = 2;

#[derive(Default)]
pub struct SAHPartition {
    nodes: Vec<AABB>,
    container: AABB,
}

impl SAHPartition {
    fn add(&mut self, aabb: &AABB) -> &mut Self {
        self.container.expand_to_include(aabb);
        self.nodes.push(*aabb);
        self
    }
    fn size(&self) -> Vector3 {
        self.container.size()
    }
    pub fn compute(container: &AABB, list: &[AABB]) -> (Vec<AABB>, Vec<AABB>) {
        let centroid = container.min() + container.size() / 2.;
        let splice_size = container.size() / (SPLIT_COUNT as f32);
        let mut partition: [[SAHPartition; SPLIT_COUNT]; AXIS_COUNT] = Default::default();
        list.iter().for_each(|aabb| {
            let mut center = container.min();
            (0..AXIS_COUNT).for_each(|axis| {
                (0..SPLIT_COUNT).for_each(|split| {
                    center += splice_size;
                    if center[axis] < centroid[axis] {
                        partition[axis][split].add(aabb);
                    }
                });
            });
        });
        let mut ratio_diff = [[0_f32; SPLIT_COUNT]; AXIS_COUNT];
        (0..AXIS_COUNT).for_each(|axis| {
            (0..SPLIT_COUNT).for_each(|split| {
                let partition_size = partition[axis][split].size();
                let mut left_ratio = 0_f32;
                for _ in 0..split {
                    left_ratio += partition_size[axis] / splice_size[axis];
                }
                let mut right_ratio = 0_f32;
                for _ in (split + 1)..SPLIT_COUNT {
                    right_ratio += partition_size[axis] / splice_size[axis];
                }
                ratio_diff[axis][split] = (left_ratio - right_ratio).max(right_ratio - left_ratio);
            });
        });
        let mut best_separation_axis = 0;
        let mut best_separation_axis_split = 0;
        let reference_diff = 1.;
        (0..AXIS_COUNT).for_each(|axis| {
            let mut axis_diff = 1.;
            let mut best_split = 0;
            (0..SPLIT_COUNT).for_each(|split| {
                if ratio_diff[axis][split] < axis_diff {
                    axis_diff = ratio_diff[axis][split];
                    best_split = split;
                }
            });
            if axis_diff < reference_diff {
                best_separation_axis = axis;
                best_separation_axis_split = best_split;
            }
        });
        let mut left_nodes = Vec::new();
        let mut right_nodes = Vec::new();
        partition[best_separation_axis]
            .iter()
            .enumerate()
            .for_each(|(index, partition)| {
                if index <= best_separation_axis_split {
                    left_nodes.extend_from_slice(&partition.nodes);
                } else {
                    right_nodes.extend_from_slice(&partition.nodes);
                }
            });
        (left_nodes, right_nodes)
    }
}
