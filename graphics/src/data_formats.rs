use nrg_math::*;

pub struct UniformData {
    pub model: Matrix4<f32>,
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>, 
}

pub struct VertexData {
    pub pos: Vector2f,
    pub color: Vector3f,
    pub tex_coord: Vector2f,     
}

impl Default for VertexData {
    fn default() -> VertexData {
        VertexData { 
            pos: [0.0, 0.0].into(),
            color: [0.0, 0.0, 0.0].into(),
            tex_coord: [0.0, 0.0].into(),
        }
    }
}
