use crate::data_formats::*;
use crate::device::*;

pub struct Mesh {
    pub inner: super::api::backend::mesh::Mesh,
    pub vertices: Vec<VertexData>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn create() -> Mesh {
        Self {
            inner: super::api::backend::mesh::Mesh::new(),
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn set_vertices(&mut self, device:&Device, vertex_data:&[VertexData]) -> &mut Self {
        self.vertices.clear();
        self.vertices.extend_from_slice(vertex_data);
        self.inner.create_vertex_buffer(&device.get_internal_device(), self.vertices.as_slice());
        self
    }

    pub fn set_indices(&mut self, device:&Device, indices_data:&[u32]) -> &mut Self {
        self.indices.clear();
        self.indices.extend_from_slice(indices_data);
        self.inner.create_index_buffer(&device.get_internal_device(), self.indices.as_slice());
        self
    }
    
    pub fn destroy(&mut self, device:&Device) {
        self.vertices.clear();
        self.indices.clear();
        self.inner.delete(&device.get_internal_device());
    }

    pub fn draw(&self, device:&Device) {
        self.inner.draw(&device.get_internal_device());
    }
}