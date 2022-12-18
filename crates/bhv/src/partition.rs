use inox_math::{VecBase, Vector3};

use crate::{AABB, AXIS_COUNT};

const SPLIT_COUNT: usize = 2;

#[derive(Default)]
pub struct Partition {
    nodes: Vec<AABB>,
    container: AABB,
}

impl Partition {
    fn add(&mut self, aabb: &AABB) -> &mut Self {
        self.container.expand_to_include(aabb);
        self.nodes.push(*aabb);
        self
    }
    fn size(&self) -> Vector3 {
        self.container.size()
    }
    pub fn compute(container: &AABB, list: &[AABB]) -> (Vec<AABB>, Vec<AABB>) {
        let (partition, splice_size) = {
            let (partition, splice_size) = Partition::compute_csh(container, list);
            if Self::validate_partition(&partition) {
                (partition, splice_size)
            } else {
                Partition::compute_cdh(container, list)
            }
        };
        if Self::validate_partition(&partition) {
            let (left_group, right_group) = Self::compute_sah(&partition, splice_size);
            (left_group, right_group)
        } else {
            //simple sorting from left to right and splitting in half then
            let mut list = list.to_vec();
            list.sort_by(|a, b| a.min().x.partial_cmp(&b.min().x).unwrap());
            let mut left_group = Vec::new();
            let mut right_group = Vec::new();
            list.iter().enumerate().for_each(|(i, aabb)| {
                if i < list.len() / 2 {
                    left_group.push(*aabb);
                } else {
                    right_group.push(*aabb);
                }
            });
            (left_group, right_group)
        }
    }
    fn validate_partition(partition: &[[Partition; SPLIT_COUNT]; AXIS_COUNT]) -> bool {
        let mut is_valid = true;
        (0..SPLIT_COUNT).for_each(|split| {
            is_valid &= !(0..AXIS_COUNT).all(|axis| partition[axis][split].nodes.is_empty());
        });
        is_valid
    }
    fn create_partition(
        list: &[AABB],
        splice_size: Vector3,
        min: Vector3,
    ) -> [[Partition; SPLIT_COUNT]; AXIS_COUNT] {
        const EPSILON: f32 = 0.0001;
        let mut partition: [[Partition; SPLIT_COUNT]; AXIS_COUNT] = Default::default();
        list.iter().for_each(|aabb| {
            let center = aabb.min() + aabb.size() / 2.;
            let mut is_assigned = [false; AXIS_COUNT];
            let mut centroid = min;
            (0..SPLIT_COUNT).for_each(|split| {
                let split_min = centroid;
                centroid += splice_size;
                let split_max = centroid;
                (0..AXIS_COUNT).for_each(|axis| {
                    if !is_assigned[axis]
                        && ((split == 0
                            && center[axis] >= (split_min[axis] - EPSILON)
                            && center[axis] <= split_max[axis])
                            || (split == (AXIS_COUNT - 1)
                                && center[axis] > split_min[axis]
                                && center[axis] <= (split_max[axis] + EPSILON))
                            || (center[axis] > split_min[axis] && center[axis] <= split_max[axis]))
                    {
                        partition[axis][split].add(aabb);
                        is_assigned[axis] = true;
                    }
                });
            });
        });
        partition
    }
    //centroid distance heuristic
    fn compute_cdh(
        container: &AABB,
        list: &[AABB],
    ) -> ([[Partition; SPLIT_COUNT]; AXIS_COUNT], Vector3) {
        let mut min = container.max();
        let mut max = container.min();
        list.iter().for_each(|aabb| {
            min = min.min(aabb.center());
            max = max.max(aabb.center());
        });
        let splice_size = (max - min) / (SPLIT_COUNT as f32);
        let partition = Self::create_partition(list, splice_size, min);
        (partition, splice_size)
    }
    //container space heuristic
    fn compute_csh(
        container: &AABB,
        list: &[AABB],
    ) -> ([[Partition; SPLIT_COUNT]; AXIS_COUNT], Vector3) {
        let splice_size = container.size() / (SPLIT_COUNT as f32);
        let partition = Self::create_partition(list, splice_size, container.min());
        (partition, splice_size)
    }
    //screen area heuristic
    fn compute_sah(
        partition: &[[Partition; SPLIT_COUNT]; AXIS_COUNT],
        splice_size: Vector3,
    ) -> (Vec<AABB>, Vec<AABB>) {
        let mut ratio_diff = [[0_f32; SPLIT_COUNT - 1]; AXIS_COUNT];
        (0..AXIS_COUNT).for_each(|axis| {
            (0..SPLIT_COUNT - 1).for_each(|split| {
                let mut left_ratio = 0_f32;
                for split_index in 0..(split + 1) {
                    let partition_size = partition[axis][split_index].size();
                    left_ratio += partition_size[axis] / splice_size[axis];
                }
                let mut right_ratio = 0_f32;
                for split_index in (split + 1)..SPLIT_COUNT {
                    let partition_size = partition[axis][split_index].size();
                    right_ratio += partition_size[axis] / splice_size[axis];
                }
                ratio_diff[axis][split] = (left_ratio - right_ratio).max(right_ratio - left_ratio);
            });
        });
        let mut best_separation_axis = 0;
        let mut best_separation_axis_split = 0;
        let reference_diff = f32::MAX;
        (0..AXIS_COUNT).for_each(|axis| {
            let mut axis_diff = f32::MAX;
            let mut best_split = 0;
            (0..SPLIT_COUNT - 1).for_each(|split| {
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
