use sabi_math::{VecBase, Vector2, Vector3, Vector4};
use sabi_serialize::{Deserialize, Serialize};

pub const MAX_TEXTURE_COORDS_SETS: usize = 4;

#[repr(C, align(16))]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
#[serde(crate = "sabi_serialize")]
pub struct VertexData {
    pub pos: Vector3,
    pub normal: Vector3,
    pub tangent: Vector3,
    pub color: Vector4,
    pub tex_coord: [Vector2; MAX_TEXTURE_COORDS_SETS],
}

impl Default for VertexData {
    fn default() -> VertexData {
        VertexData {
            pos: Vector3::default_zero(),
            normal: Vector3::new(0., 0., 1.),
            tangent: Vector3::new(0., 0., 1.),
            color: Vector4::new(1., 1., 1., 1.),
            tex_coord: [Vector2::default_zero(); MAX_TEXTURE_COORDS_SETS],
        }
    }
}
