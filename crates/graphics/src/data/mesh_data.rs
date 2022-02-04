use std::{ops::Range, path::PathBuf};

use inox_math::{is_point_in_triangle, Vector2, Vector3, Vector4};
use inox_serialize::{Deserialize, Serialize, SerializeFile};

use crate::{create_quad, VertexData};

pub struct MeshBindingData {
    pub vertices: Range<u32>,
    pub indices: Range<u32>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "inox_serialize")]
pub struct MeshData {
    pub vertices: Vec<VertexData>,
    pub indices: Vec<u32>,
    pub material: PathBuf,
}

impl SerializeFile for MeshData {
    fn extension() -> &'static str {
        "mesh"
    }
}

impl Default for MeshData {
    fn default() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            material: PathBuf::new(),
        }
    }
}

impl MeshData {
    pub fn clear(&mut self) -> &mut Self {
        self.vertices.clear();
        self.indices.clear();
        self
    }

    pub fn compute_min_max(&self) -> (Vector3, Vector3) {
        let mut min = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Vector3::new(f32::MIN, f32::MIN, f32::MIN);
        for v in self.vertices.iter() {
            min.x = min.x.min(v.pos.x);
            min.y = min.y.min(v.pos.y);
            min.z = min.z.min(v.pos.z);
            max.x = max.x.max(v.pos.x);
            max.y = max.y.max(v.pos.y);
            max.z = max.z.max(v.pos.z);
        }
        (min, max)
    }

    pub fn compute_center(&self) -> Vector3 {
        let (min, max) = self.compute_min_max();
        [
            min.x + (max.x - min.x) * 0.5,
            min.y + (max.y - min.y) * 0.5,
            min.z + (max.z - min.z) * 0.5,
        ]
        .into()
    }

    pub fn set_vertex_color(&mut self, color: Vector4) -> &mut Self {
        for v in self.vertices.iter_mut() {
            v.color = color;
        }
        self
    }
    pub fn set_mesh_at_index(
        &mut self,
        vertices: &[VertexData],
        first_vertex: u32,
        indices: &[u32],
        first_index: u32,
    ) {
        let start = first_vertex as usize;
        let end = start + vertices.len();
        self.vertices[start..end].copy_from_slice(vertices);

        let start = first_index as usize;
        let end = start + indices.len();
        self.indices[start..end].copy_from_slice(indices);
    }

    pub fn append_mesh(&mut self, vertices: &[VertexData], indices: &[u32]) {
        let first_vertex = self.vertices.len() as u32;
        let first_index = self.indices.len() as u32;

        self.vertices.append(&mut vertices.to_vec());
        self.indices.append(&mut indices.to_vec());

        let last_index = self.indices.len() as u32 - 1;

        self.indices[first_index as usize..(last_index + 1) as usize]
            .iter_mut()
            .for_each(|i| *i += first_vertex);
        self.compute_center();
    }

    pub fn add_quad_default(&mut self, rect: Vector4, z: f32) {
        let tex_coords = [0.0, 0.0, 1.0, 1.0].into();
        let (vertices, indices) = create_quad(rect, z, tex_coords, None);
        self.append_mesh(&vertices, &indices);
    }

    pub fn add_quad(
        &mut self,
        rect: Vector4,
        z: f32,
        tex_coords: Vector4,
        index_start: Option<usize>,
    ) {
        let (vertices, indices) = create_quad(rect, z, tex_coords, index_start);
        self.append_mesh(&vertices, &indices);
    }

    pub fn clip_in_rect(&mut self, clip_rect: Vector4) -> &mut Self {
        for v in self.vertices.iter_mut() {
            v.pos.x = v.pos.x.max(clip_rect.x);
            v.pos.x = v.pos.x.min(clip_rect.z);
            v.pos.y = v.pos.y.max(clip_rect.y);
            v.pos.y = v.pos.y.min(clip_rect.w);
        }
        self.compute_center();
        self
    }

    pub fn is_inside(&self, pos_in_screen_space: Vector2) -> bool {
        let mut i = 0;
        let count = self.indices.len();
        while i < count {
            let v1 = self.vertices[self.indices[i] as usize].pos.xy();
            let v2 = self.vertices[self.indices[i + 1] as usize].pos.xy();
            let v3 = self.vertices[self.indices[i + 2] as usize].pos.xy();
            if is_point_in_triangle(v1, v2, v3, pos_in_screen_space.x, pos_in_screen_space.y) {
                return true;
            }
            i += 3;
        }
        false
    }
}
