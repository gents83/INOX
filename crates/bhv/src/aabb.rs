use inox_math::{VecBase, Vector3};

pub const AXIS_COUNT: usize = 3;
pub(crate) const INVALID_INDEX: i32 = -1;

#[derive(Clone, Copy)]
pub struct AABB {
    min: Vector3,
    max: Vector3,
    index: i32,
}

impl Default for AABB {
    fn default() -> Self {
        Self::empty()
    }
}

impl AABB {
    pub fn empty() -> Self {
        Self {
            max: [f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY].into(),
            min: [f32::INFINITY, f32::INFINITY, f32::INFINITY].into(),
            index: INVALID_INDEX,
        }
    }
    pub fn create(min: Vector3, max: Vector3, index: i32) -> Self {
        Self { max, min, index }
    }
    pub fn compute_aabb(list: &[AABB]) -> Self {
        let mut total = Self::empty();
        list.iter().for_each(|aabb| {
            total.expand_to_include(aabb);
        });
        total
    }
    pub fn min_axis(&self, axis_index: usize) -> f32 {
        self.min[axis_index]
    }
    pub fn max_axis(&self, axis_index: usize) -> f32 {
        self.max[axis_index]
    }
    pub fn index(&self) -> i32 {
        self.index
    }
    pub fn center(&self) -> Vector3 {
        self.min + self.size() * 0.5
    }
    pub fn set_index(&mut self, index: u32) -> &mut Self {
        self.index = index as _;
        self
    }
    pub fn set_min(&mut self, min: Vector3) -> &mut Self {
        self.min = min;
        self
    }
    pub fn set_max(&mut self, max: Vector3) -> &mut Self {
        self.max = max;
        self
    }
    pub fn min(&self) -> Vector3 {
        self.min
    }
    pub fn max(&self) -> Vector3 {
        self.max
    }
    pub fn size(&self) -> Vector3 {
        self.max - self.min
    }
    pub fn expand_to_include(&mut self, other: &AABB) {
        self.max = self.max.max(other.max).max(other.min);
        self.min = self.min.min(other.min).min(other.max);
    }
}
