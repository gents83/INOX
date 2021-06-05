use super::data_formats::*;
use super::device::*;

#[derive(Clone)]
pub struct Mesh {
    inner: crate::api::backend::mesh::Mesh,
    device: Device,
    pub data: MeshData,
}

impl Mesh {
    pub fn create(device: &Device) -> Mesh {
        Self {
            inner: crate::api::backend::mesh::Mesh::default(),
            device: device.clone(),
            data: MeshData::default(),
        }
    }
    pub fn destroy(&mut self) {
        self.inner.delete(&self.device.inner);
    }

    pub fn bind_at_index(
        &mut self,
        vertices: &[VertexData],
        first_vertex: u32,
        indices: &[u32],
        first_index: u32,
    ) -> MeshDataRef {
        if first_vertex as usize + vertices.len() >= self.data.vertices.len() {
            self.data.vertices.resize_with(
                (self.data.vertices.len() + vertices.len()) * 2,
                VertexData::default,
            );
            self.inner
                .create_vertex_buffer(&self.device.inner, self.data.vertices.as_slice());
        }
        if first_index as usize + indices.len() >= self.data.indices.len() {
            self.data
                .indices
                .resize_with((self.data.indices.len() + indices.len()) * 2, u32::default);
            self.inner
                .create_index_buffer(&self.device.inner, self.data.indices.as_slice());
        }

        self.inner.bind_at_index(
            &self.device.inner,
            vertices,
            first_vertex,
            indices,
            first_index,
        );
        self.data
            .set_mesh_at_index(vertices, first_vertex, indices, first_index)
    }

    pub fn bind_vertices(&self) {
        self.inner.bind_vertices(&self.device.inner);
    }

    pub fn bind_indices(&self) {
        self.inner.bind_indices(&self.device.inner);
    }

    pub fn draw(&mut self, num_vertices: u32, num_indices: u32) {
        if !self.data.vertices.is_empty() {
            self.inner.draw(
                &self.device.inner,
                &self.data.vertices,
                num_vertices,
                &self.data.indices,
                num_indices,
            );
        }
    }
}
