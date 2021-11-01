use crate::api::backend::{BackendMesh, BackendPhysicalDevice};
use crate::{
    CommandBuffer, Device, MeshBindingData, MeshCategoryId, MeshData, MeshDataRef, VertexData,
};

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
        self.inner.delete(device);
    }

    pub fn bind_at_index(
        &mut self,
        device: &Device,
        physical_device: &BackendPhysicalDevice,
        binding_data: MeshBindingData,
    ) -> MeshDataRef {
        if binding_data.first_vertex as usize + binding_data.vertices.len()
            >= self.data.vertices.len()
        {
            self.data.vertices.resize_with(
                (self.data.vertices.len() + binding_data.vertices.len()) * 2,
                VertexData::default,
            );
            self.inner
                .create_vertex_buffer(device, physical_device, self.data.vertices.as_slice());
        }
        if binding_data.first_index as usize + binding_data.indices.len() >= self.data.indices.len()
        {
            self.data.indices.resize_with(
                (self.data.indices.len() + binding_data.indices.len()) * 2,
                u32::default,
            );
            self.inner
                .create_index_buffer(device, physical_device, self.data.indices.as_slice());
        }

        self.mesh_categories
            .push(binding_data.mesh_category_identifier);
        self.inner.bind_at_index(
            device,
            binding_data.vertices,
            binding_data.first_vertex,
            binding_data.indices,
            binding_data.first_index,
        );
        self.data.set_mesh_at_index(
            binding_data.vertices,
            binding_data.first_vertex,
            binding_data.indices,
            binding_data.first_index,
        )
    }

    pub fn bind_vertices(&self, command_buffer: &CommandBuffer) {
        self.inner.bind_vertices(command_buffer);
    }

    pub fn bind_indices(&self, command_buffer: &CommandBuffer) {
        self.inner.bind_indices(command_buffer);
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
                device,
                command_buffer,
                &self.data.vertices,
                num_vertices,
                &self.data.indices,
                num_indices,
            );
        }
    }
}
