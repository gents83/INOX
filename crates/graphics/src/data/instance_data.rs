use sabi_math::{VecBase, Vector4};

use crate::INVALID_INDEX;

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct InstanceData {
    pub id: Vector4,
    pub matrix: [[f32; 4]; 4],
    pub draw_area: Vector4,
    pub material_index: i32,
}

impl Default for InstanceData {
    fn default() -> Self {
        Self {
            id: Vector4::default_zero(),
            matrix: [[0.; 4]; 4],
            draw_area: [0., 0., f32::MAX, f32::MAX].into(),
            material_index: INVALID_INDEX,
        }
    }
}
