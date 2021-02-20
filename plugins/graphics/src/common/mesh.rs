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

    pub fn finalize(&mut self) -> &mut Self {
        if !self.data.vertices.is_empty() {
            self.inner
                .create_vertex_buffer(&self.device.inner, self.data.vertices.as_slice());
            self.inner
                .create_index_buffer(&self.device.inner, self.data.indices.as_slice());
        }
        self
    }

    pub fn draw(&self) {
        if !self.data.vertices.is_empty() {
            self.inner.draw(&self.device.inner);
        }
    }
}
