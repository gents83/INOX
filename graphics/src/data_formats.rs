use nrg_math::*;

pub struct VertexData {
    pub pos: Vector2f,
    pub color: Vector3f, 
}

impl Default for VertexData {
    fn default() -> VertexData {
        VertexData { 
            pos: [0.0, 0.0].into(),
            color: [0.0, 0.0, 0.0].into(),
        }
    }
}
