use super::data_formats::*;
use super::device::*;

const MAX_BUFFER_SIZE: usize = 4096;

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

    pub fn fill_mesh_with_max_buffers(&mut self) {
        self.data
            .vertices
            .resize_with(MAX_BUFFER_SIZE, VertexData::default);
        self.data.indices.resize_with(MAX_BUFFER_SIZE, u32::default);
        self.finalize();
    }

    fn finalize(&mut self) -> &mut Self {
        if !self.data.vertices.is_empty() {
            self.inner
                .create_vertex_buffer(&self.device.inner, self.data.vertices.as_slice());
        }
        if !self.data.indices.is_empty() {
            self.inner
                .create_index_buffer(&self.device.inner, self.data.indices.as_slice());
        }
        self
    }

    pub fn draw(&mut self, num_vertices: usize, num_indices: usize) {
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
