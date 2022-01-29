use sabi_resources::DataTypeResource;

use crate::{GpuBuffer, Mesh, MeshId, RenderContext, VertexData};

#[derive(Default)]
pub struct GraphicsMesh {
    vertex_buffer: GpuBuffer<VertexData, { wgpu::BufferUsages::bits(&wgpu::BufferUsages::VERTEX) }>,
    index_buffer: GpuBuffer<u32, { wgpu::BufferUsages::bits(&wgpu::BufferUsages::INDEX) }>,
}

unsafe impl Send for GraphicsMesh {}
unsafe impl Sync for GraphicsMesh {}

impl GraphicsMesh {
    pub fn vertex_count(&self) -> usize {
        self.vertex_buffer.len()
    }
    pub fn index_count(&self) -> usize {
        self.index_buffer.len()
    }
    pub fn vertex_buffer(&self) -> Option<wgpu::BufferSlice> {
        if let Some(buffer) = self.vertex_buffer.gpu_buffer() {
            return Some(buffer.slice(..));
        }
        None
    }
    pub fn index_buffer(&self) -> Option<wgpu::BufferSlice> {
        if let Some(buffer) = self.index_buffer.gpu_buffer() {
            return Some(buffer.slice(..));
        }
        None
    }
    pub fn add_mesh(&mut self, mesh_id: &MeshId, mesh: &mut Mesh) {
        let mesh_data = mesh.mesh_data();
        if mesh.is_initialized() || mesh_data.vertices.is_empty() {
            return;
        }
        self.vertex_buffer
            .add(mesh_id, mesh_data.vertices.as_slice());
        self.index_buffer.add(mesh_id, mesh_data.indices.as_slice());

        self.add_mesh_to_pipeline(mesh_id, mesh);
        mesh.init();
    }
    pub fn update_mesh(&mut self, mesh_id: &MeshId, mesh: &Mesh) {
        if !mesh.is_initialized() || mesh.mesh_data().vertices.is_empty() {
            return;
        }
        self.add_mesh_to_pipeline(mesh_id, mesh);
    }
    fn add_mesh_to_pipeline(&mut self, mesh_id: &MeshId, mesh: &Mesh) {
        if let Some(material) = mesh.material() {
            if let Some(pipeline) = material.get().pipeline() {
                let vertex_data = self.vertex_buffer.get(mesh_id).unwrap();
                let index_data = self.index_buffer.get(mesh_id).unwrap();
                pipeline
                    .get_mut()
                    .add_mesh_to_instance_buffer(mesh_id, mesh);
                pipeline.get_mut().add_mesh_to_indirect_buffer(
                    mesh_id,
                    mesh,
                    vertex_data,
                    index_data,
                )
            }
        }
    }
    pub fn remove_mesh(&mut self, mesh_id: &MeshId) {
        self.vertex_buffer.remove(mesh_id);
        self.index_buffer.remove(mesh_id);

        if self.vertex_buffer.is_empty() {
            self.vertex_buffer.clear();
        }
        if self.index_buffer.is_empty() {
            self.index_buffer.clear();
        }
    }
    pub fn clear(&mut self) {
        self.vertex_buffer.clear();
        self.index_buffer.clear();
    }
    pub fn send_to_gpu(&mut self, context: &RenderContext) {
        self.vertex_buffer.send_to_gpu(context);
        self.index_buffer.send_to_gpu(context);
    }
}
