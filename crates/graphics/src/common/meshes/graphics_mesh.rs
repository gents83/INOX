use std::collections::HashMap;

use inox_math::{matrix3_to_array, matrix4_to_array, Mat4Ops, Matrix, Matrix3};
use inox_messenger::MessageHubRc;
use inox_resources::{BufferData, ResourceId, ResourceTrait, SharedData, SharedDataRc};

use crate::{GpuBuffer, InstanceData, Mesh, MeshId, PipelineId, VertexData, INVALID_INDEX};

pub const GRAPHIC_MESH_UID: ResourceId =
    inox_uid::generate_static_uid_from_string(stringify!(GraphicMesh));

#[derive(Default)]
pub struct GraphicsMesh {
    vertex_buffer: GpuBuffer<VertexData, { wgpu::BufferUsages::bits(&wgpu::BufferUsages::VERTEX) }>,
    index_buffer: GpuBuffer<u32, { wgpu::BufferUsages::bits(&wgpu::BufferUsages::INDEX) }>,

    instance_buffers: HashMap<
        PipelineId,
        GpuBuffer<InstanceData, { wgpu::BufferUsages::bits(&wgpu::BufferUsages::VERTEX) }>,
    >,
    indirect_buffers: HashMap<
        PipelineId,
        GpuBuffer<
            wgpu::util::DrawIndexedIndirect,
            { wgpu::BufferUsages::bits(&wgpu::BufferUsages::INDIRECT) },
        >,
    >,
}

impl ResourceTrait for GraphicsMesh {
    type OnCreateData = ();

    fn on_create(
        &mut self,
        _shared_data_rc: &SharedDataRc,
        _message_hub: &MessageHubRc,
        _id: &ResourceId,
        _on_create_data: Option<&<Self as ResourceTrait>::OnCreateData>,
    ) {
    }
    fn on_destroy(
        &mut self,
        _shared_data: &SharedData,
        _message_hub: &MessageHubRc,
        _id: &ResourceId,
    ) {
    }
    fn on_copy(&mut self, _other: &Self)
    where
        Self: Sized,
    {
        debug_assert!(false, "GraphicsMesh::on_copy should not be called");
    }
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
    pub fn has_mesh(&mut self, mesh_id: &MeshId) -> bool {
        self.vertex_buffer.get(mesh_id).is_some()
    }
    pub fn add_mesh(&mut self, mesh_id: &MeshId, mesh: &Mesh) -> bool {
        let mesh_data = mesh.mesh_data();
        if mesh_data.vertices.is_empty() || !mesh.is_visible() {
            self.remove_mesh(mesh_id);
            return false;
        }
        self.vertex_buffer
            .add(mesh_id, mesh_data.vertices.as_slice());
        self.index_buffer.add(mesh_id, mesh_data.indices.as_slice());

        self.add_mesh_to_pipeline(mesh_id, mesh)
    }

    fn add_mesh_to_pipeline(&mut self, mesh_id: &MeshId, mesh: &Mesh) -> bool {
        if let Some(material) = mesh.material() {
            if let Some(pipeline) = material.get().pipeline() {
                let vertex_data = self.vertex_buffer.get(mesh_id).unwrap();
                let index_data = self.index_buffer.get(mesh_id).unwrap();
                let instance_buffer = self.instance_buffers.entry(*pipeline.id()).or_default();
                let indirect_buffer = self.indirect_buffers.entry(*pipeline.id()).or_default();
                let instance_index =
                    Self::add_mesh_to_instance_buffer(mesh_id, mesh, instance_buffer);
                Self::add_mesh_to_indirect_buffer(
                    mesh_id,
                    instance_index as _,
                    vertex_data,
                    index_data,
                    indirect_buffer,
                );
                return true;
            }
        }
        false
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

        self.instance_buffers.iter_mut().for_each(|(_, b)| {
            b.remove(mesh_id);
        });
        self.indirect_buffers.iter_mut().for_each(|(_, b)| {
            b.remove(mesh_id);
        });
    }
    pub fn clear(&mut self) {
        self.vertex_buffer.clear();
        self.index_buffer.clear();
    }

    pub fn instance_buffer(&self, pipeline_id: &PipelineId) -> Option<wgpu::BufferSlice> {
        if let Some(buffer) = self.instance_buffers.get(pipeline_id) {
            if let Some(gpu_buffer) = buffer.gpu_buffer() {
                return Some(gpu_buffer.slice(..));
            }
        }
        None
    }
    pub fn instance_count(&self, pipeline_id: &PipelineId) -> usize {
        if let Some(buffer) = self.instance_buffers.get(pipeline_id) {
            return buffer.len();
        }
        0
    }
    pub fn for_each_instance<F>(&self, pipeline_id: &PipelineId, f: F)
    where
        F: FnMut(usize, &InstanceData),
    {
        if let Some(buffer) = self.instance_buffers.get(pipeline_id) {
            buffer.for_each_data(f);
        }
    }
    pub fn indirect(
        &self,
        index: u32,
        pipeline_id: &PipelineId,
    ) -> Option<&wgpu::util::DrawIndexedIndirect> {
        if let Some(buffer) = self.indirect_buffers.get(pipeline_id) {
            return Some(buffer.data_at_index(index));
        }
        None
    }
    pub fn indirect_buffer(&self, pipeline_id: &PipelineId) -> Option<&wgpu::Buffer> {
        if let Some(buffer) = self.indirect_buffers.get(pipeline_id) {
            return buffer.gpu_buffer();
        }
        None
    }

    fn add_mesh_to_instance_buffer(
        mesh_id: &MeshId,
        mesh: &Mesh,
        instance_buffer: &mut GpuBuffer<
            InstanceData,
            { wgpu::BufferUsages::bits(&wgpu::BufferUsages::VERTEX) },
        >,
    ) -> usize {
        let normal_matrix = mesh.matrix().inverse().transpose();
        let normal_matrix = Matrix3::from_cols(
            normal_matrix.x.xyz(),
            normal_matrix.y.xyz(),
            normal_matrix.z.xyz(),
        );
        let instance = InstanceData {
            matrix: matrix4_to_array(mesh.matrix()),
            normal_matrix: matrix3_to_array(normal_matrix),
            draw_area: mesh.draw_area().into(),
            material_index: mesh
                .material()
                .as_ref()
                .map_or(INVALID_INDEX, |m| m.get().uniform_index()),
        };

        let instance_range = instance_buffer.add(mesh_id, &[instance]);
        if mesh.draw_index() >= 0 && instance_range.start != mesh.draw_index() as usize {
            instance_buffer.swap(instance_range.start as _, mesh.draw_index() as _);
            return mesh.draw_index() as _;
        }
        instance_range.start
    }
    fn add_mesh_to_indirect_buffer(
        mesh_id: &MeshId,
        instance_index: u32,
        vertex_data: &BufferData,
        index_data: &BufferData,
        indirect_buffer: &mut GpuBuffer<
            wgpu::util::DrawIndexedIndirect,
            { wgpu::BufferUsages::bits(&wgpu::BufferUsages::INDIRECT) },
        >,
    ) {
        let old_range = indirect_buffer.add(
            mesh_id,
            &[wgpu::util::DrawIndexedIndirect {
                vertex_count: index_data.len() as _,
                instance_count: 1,
                base_index: index_data.range.start as _,
                vertex_offset: vertex_data.range.start as _,
                base_instance: instance_index as _,
            }],
        );
        if old_range.start != instance_index as usize {
            indirect_buffer.swap(instance_index as _, old_range.start as _);
        }
    }

    pub fn send_to_gpu(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.vertex_buffer.send_to_gpu(device, queue);
        self.index_buffer.send_to_gpu(device, queue);

        self.instance_buffers.iter_mut().for_each(|(_, b)| {
            b.send_to_gpu(device, queue);
        });
        self.indirect_buffers.iter_mut().for_each(|(_, b)| {
            b.send_to_gpu(device, queue);
        });
    }
}
