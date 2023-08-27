use inox_math::Vector3;

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
    fn calculate_sah(container: &AABB, aabbs: &[AABB], split_index: usize) -> f32 {
        let mut left_aabb = AABB::default();
        let mut right_aabb = AABB::default();

        (0..split_index).for_each(|i| {
            left_aabb.expand_to_include(&aabbs[i]);
        });
        (split_index..aabbs.len()).for_each(|i| {
            right_aabb.expand_to_include(&aabbs[i]);
        });
        let left_area = left_aabb.surface_area();
        let right_area = right_aabb.surface_area();
        let total_area = container.surface_area();

        0.125
            + (left_area * split_index as f32 + right_area * (aabbs.len() - split_index) as f32)
                / total_area
    }
    fn find_best_split(container: &AABB, aabbs: &[AABB], axis: usize) -> (usize, f32) {
        let mut sorted_aabbs = aabbs.to_vec();
        sorted_aabbs.sort_by(|a, b| a.center()[axis].partial_cmp(&b.center()[axis]).unwrap());

        let mut best_split = 0;
        let mut best_sah = f32::INFINITY;
        for i in 0..sorted_aabbs.len() - 1 {
            let sah = Self::calculate_sah(container, &sorted_aabbs, i + 1);

            if sah < best_sah {
                best_split = i + 1;
                best_sah = sah;
            }
        }
        (best_split, best_sah)
    }
    //screen area heuristic
    pub fn compute_sah(container: &AABB, list: &[AABB]) -> (Vec<AABB>, Vec<AABB>) {
        let (best_axis, _) = (0..AXIS_COUNT)
            .map(|axis| (axis, Self::find_best_split(container, list, axis).1))
            .min_by(|(_, cost1), (_, cost2)| cost1.partial_cmp(cost2).unwrap())
            .unwrap();
        let (split_index, _) = Self::find_best_split(container, list, best_axis);

        let mut aabbs = list.to_vec();
        aabbs.swap(split_index, list.len() - 1);
        let (left, right) = aabbs.split_at_mut(split_index);
        (left.to_vec(), right.to_vec())
    }
}
