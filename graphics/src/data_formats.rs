use nrg_math::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct UniformData {
    pub model: Matrix4<f32>,
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>, 
}



#[derive(Debug, PartialEq, Clone, Copy)]
pub struct VertexData {
    pub pos: Vector3f,
    pub color: Vector3f,
    pub tex_coord: Vector2f,
    pub normal: Vector3f,     
}

impl Default for VertexData {
    fn default() -> VertexData {
        VertexData { 
            pos: [0.0, 0.0, 0.0].into(),
            color: [1.0, 1.0, 1.0].into(),
            tex_coord: [0.0, 0.0].into(),
            normal: [0.0, 0.0, 1.0].into(),
        }
    }
}
