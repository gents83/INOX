use crate::api::backend::BackendMesh;
use crate::CommandBuffer;

use super::data_formats::*;
use super::device::*;

#[derive(Default, Clone)]
pub struct GraphicsMesh {
    inner: BackendMesh,
    data: MeshData,
    mesh_categories: Vec<MeshCategoryId>,
}

unsafe impl Send for GraphicsMesh {}
unsafe impl Sync for GraphicsMesh {}

impl GraphicsMesh {
    pub fn reset_mesh_categories(&mut self) {
        self.mesh_categories.clear();
    }
    pub fn mesh_categories(&self) -> &[MeshCategoryId] {
        &self.mesh_categories
    }

    pub fn destroy(&mut self, device: &Device) {
        self.inner.delete(&*device);
    }

    pub fn bind_at_index(
        &mut self,
        device: &Device,
        mesh_category_identifier: MeshCategoryId,
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
                .create_vertex_buffer(&*device, self.data.vertices.as_slice());
        }
        if first_index as usize + indices.len() >= self.data.indices.len() {
            self.data
                .indices
                .resize_with((self.data.indices.len() + indices.len()) * 2, u32::default);
            self.inner
                .create_index_buffer(&*device, self.data.indices.as_slice());
        }

        self.mesh_categories.push(mesh_category_identifier);
        self.inner
            .bind_at_index(&*device, vertices, first_vertex, indices, first_index);
        self.data
            .set_mesh_at_index(vertices, first_vertex, indices, first_index)
    }

    pub fn bind_vertices(&self, command_buffer: &CommandBuffer) {
        self.inner.bind_vertices(&*command_buffer);
    }

    pub fn bind_indices(&self, command_buffer: &CommandBuffer) {
        self.inner.bind_indices(&*command_buffer);
    }

    pub fn draw(
        &mut self,
        device: &Device,
        command_buffer: &CommandBuffer,
        num_vertices: u32,
        num_indices: u32,
    ) {
        if !self.data.vertices.is_empty() {
            self.inner.draw(
                &*device,
                &*command_buffer,
                &self.data.vertices,
                num_vertices,
                &self.data.indices,
                num_indices,
            );
        }
    }
}
