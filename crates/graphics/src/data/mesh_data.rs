use std::{mem::size_of, path::PathBuf};

use inox_math::{is_point_in_triangle, Vector2, Vector3, Vector4};
use inox_resources::{from_u8_slice, from_u8_slice_mut, to_u8_slice};
use inox_serialize::{Deserialize, Serialize, SerializeFile};

use crate::{create_quad_with_texture, VertexData, VertexFormat, VertexFormatBits};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "inox_serialize")]
pub struct MeshletData {
    pub center: Vector3,
    pub radius: f32,
    pub cone_axis: Vector3,
    pub cone_cutoff: f32,
    pub vertices_count: u32,
    pub vertices_offset: u32,
    pub indices_offset: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "inox_serialize")]
pub struct MeshData {
    pub vertex_format: Vec<VertexFormat>,
    pub vertices: Vec<u8>,
    pub indices: Vec<u32>,
    pub material: PathBuf,
    pub meshlets: Vec<MeshletData>,
}

impl SerializeFile for MeshData {
    fn extension() -> &'static str {
        "mesh"
    }
}

impl MeshData {
    pub fn new(vertex_format: Vec<VertexFormat>) -> Self {
        Self {
            vertex_format,
            vertices: Vec::new(),
            indices: Vec::new(),
            material: PathBuf::new(),
            meshlets: Vec::new(),
        }
    }
    pub fn vertex_format(&self) -> VertexFormatBits {
        VertexFormat::to_bits(&self.vertex_format)
    }
    pub fn vertex_size(&self) -> usize {
        self.vertex_format.iter().map(|a| a.size()).sum::<usize>()
    }
    pub fn vertex_count(&self) -> usize {
        self.vertices.len() / self.vertex_size()
    }
    pub fn attribute_offset(&self, attribute: VertexFormat) -> usize {
        let mut attribute_offset = 0;
        self.vertex_format
            .iter()
            .take_while(|&a| *a != attribute)
            .for_each(|a| {
                attribute_offset += a.size();
            });
        attribute_offset
    }
    fn attribute<T>(&self, i: usize, attribute: VertexFormat) -> &T {
        let attribute_offset = self.attribute_offset(attribute);
        let index = attribute_offset + i * self.vertex_size();
        let data = from_u8_slice::<T>(&self.vertices[index..(index + size_of::<T>())]);
        &data[0]
    }
    fn attribute_mut<T>(&mut self, i: usize, attribute: VertexFormat) -> &mut T {
        let attribute_offset = self.attribute_offset(attribute);
        let index = attribute_offset + i * self.vertex_size();
        let data = from_u8_slice_mut::<T>(&mut self.vertices[index..(index + size_of::<T>())]);
        &mut data[0]
    }

    pub fn pos3(&self, i: usize) -> &Vector3 {
        self.attribute::<Vector3>(i, VertexFormat::PositionF32x3)
    }
    pub fn pos3_mut(&mut self, i: usize) -> &mut Vector3 {
        self.attribute_mut::<Vector3>(i, VertexFormat::PositionF32x3)
    }
    pub fn color(&self, i: usize) -> &Vector4 {
        self.attribute::<Vector4>(i, VertexFormat::ColorF32x4)
    }
    pub fn color_mut(&mut self, i: usize) -> &mut Vector4 {
        self.attribute_mut::<Vector4>(i, VertexFormat::ColorF32x4)
    }

    pub fn clear(&mut self) -> &mut Self {
        self.vertices.clear();
        self.indices.clear();
        self
    }

    pub fn compute_min_max(&self) -> (Vector3, Vector3) {
        let mut min = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Vector3::new(f32::MIN, f32::MIN, f32::MIN);
        for i in 0..self.vertex_count() {
            let pos = self.pos3(i);
            min.x = min.x.min(pos.x);
            min.y = min.y.min(pos.y);
            min.z = min.z.min(pos.z);
            max.x = max.x.max(pos.x);
            max.y = max.y.max(pos.y);
            max.z = max.z.max(pos.z);
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
        for i in 0..self.vertex_count() {
            let c = self.color_mut(i);
            *c = color;
        }
        self
    }

    pub fn append_mesh<T>(&mut self, vertices: &[T], indices: &[u32]) {
        let first_vertex = self.vertices.len() as u32;
        let first_index = self.indices.len() as u32;

        self.vertices.append(&mut to_u8_slice(vertices).to_vec());
        self.indices.append(&mut indices.to_vec());

        let last_index = self.indices.len() as u32 - 1;

        self.indices[first_index as usize..(last_index + 1) as usize]
            .iter_mut()
            .for_each(|i| *i += first_vertex);
        self.compute_center();
    }

    pub fn add_quad_default<T>(&mut self, rect: Vector4, z: f32)
    where
        T: VertexData + Copy,
    {
        let tex_coords = [0.0, 0.0, 1.0, 1.0].into();
        let (vertices, indices) = create_quad_with_texture::<T>(rect, z, tex_coords, None);
        self.append_mesh(&vertices, &indices);
    }

    pub fn add_quad<T>(
        &mut self,
        rect: Vector4,
        z: f32,
        tex_coords: Vector4,
        index_start: Option<usize>,
    ) where
        T: VertexData + Copy,
    {
        let (vertices, indices) = create_quad_with_texture::<T>(rect, z, tex_coords, index_start);
        self.append_mesh(&vertices, &indices);
    }

    pub fn clip_in_rect(&mut self, clip_rect: Vector4) -> &mut Self {
        for i in 0..self.vertex_count() {
            let pos = self.pos3_mut(i);
            pos.x = pos.x.max(clip_rect.x);
            pos.x = pos.x.min(clip_rect.z);
            pos.y = pos.y.max(clip_rect.y);
            pos.y = pos.y.min(clip_rect.w);
        }
        self.compute_center();
        self
    }

    pub fn is_inside(&self, pos_in_screen_space: Vector2) -> bool {
        let mut i = 0;
        let count = self.indices.len();
        while i < count {
            let v1 = self.pos3(self.indices[i] as usize).xy();
            let v2 = self.pos3(self.indices[i + 1] as usize).xy();
            let v3 = self.pos3(self.indices[i + 2] as usize).xy();
            if is_point_in_triangle(v1, v2, v3, pos_in_screen_space.x, pos_in_screen_space.y) {
                return true;
            }
            i += 3;
        }
        false
    }
}
