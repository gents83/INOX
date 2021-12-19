use sabi_serialize::*;

pub const MAX_TEXTURE_COORDS_SETS: usize = 4;

#[repr(C, align(16))]
#[derive(Serializable, Debug, PartialEq, Clone, Copy)]
pub struct VertexData {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
    pub color: [f32; 4],
    pub tex_coord: [[f32; 2]; MAX_TEXTURE_COORDS_SETS],
}

impl Default for VertexData {
    fn default() -> VertexData {
        VertexData {
            pos: [0.; 3],
            normal: [0., 0., 1.],
            tangent: [0., 0., 1.],
            color: [1.; 4],
            tex_coord: [[0.; 2]; MAX_TEXTURE_COORDS_SETS],
        }
    }
}
