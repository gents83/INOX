use std::{collections::HashMap, mem::size_of, ops::Range};

use inox_math::{matrix3_to_array, matrix4_to_array, Mat4Ops, Matrix, Matrix3};
use inox_messenger::MessageHubRc;
use inox_resources::{HashBuffer, ResourceId, ResourceTrait, SharedData, SharedDataRc};
use wgpu::util::DrawIndexedIndirect;

use crate::{
    DataBuffer, GpuBuffer, InstanceData, Mesh, MeshData, MeshId, PipelineId, RenderContext,
    VertexData, INVALID_INDEX,
};

pub const GRAPHIC_MESH_UID: ResourceId =
    inox_uid::generate_static_uid_from_string(stringify!(GraphicMesh));

#[derive(Default)]
pub struct GraphicsMesh {
    vertex_buffer: GpuBuffer<VertexData, { wgpu::BufferUsages::bits(&wgpu::BufferUsages::VERTEX) }>,
    index_buffer: GpuBuffer<u32, { wgpu::BufferUsages::bits(&wgpu::BufferUsages::INDEX) }>,

    instance_buffers: HashMap<PipelineId, (HashBuffer<MeshId, InstanceData, 0>, DataBuffer)>,
    commands_buffers: HashMap<PipelineId, (Vec<DrawIndexedIndirect>, DataBuffer)>,
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
    pub fn vertices(&self) -> &[VertexData] {
        self.vertex_buffer.cpu_buffer()
    }
    pub fn vertices_range_of(&self, mesh_id: &MeshId) -> Option<&Range<usize>> {
        self.vertex_buffer
            .get(mesh_id)
            .as_ref()
            .map(|buffer_data| &buffer_data.range)
    }
    pub fn indices_range_of(&self, mesh_id: &MeshId) -> Option<&Range<usize>> {
        self.index_buffer
            .get(mesh_id)
            .as_ref()
            .map(|buffer_data| &buffer_data.range)
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
    pub fn get_vertex_mut(&mut self, index: usize) -> &mut VertexData {
        self.vertex_buffer.data_at_index_mut(index as _)
    }
    pub fn get_index_mut(&mut self, index: usize) -> &mut u32 {
        self.index_buffer.data_at_index_mut(index as _)
    }
    pub fn reserve_vertices(&mut self, mesh_id: &MeshId, count: usize) -> Range<usize> {
        self.vertex_buffer.reserve(mesh_id, count)
    }
    pub fn reserve_indices(&mut self, mesh_id: &MeshId, count: usize) -> Range<usize> {
        self.index_buffer.reserve(mesh_id, count)
    }
    pub fn set_vertices(&mut self, mesh_id: &MeshId, vertices: &[VertexData]) -> Range<usize> {
        self.vertex_buffer.add(mesh_id, vertices)
    }
    pub fn set_indices(&mut self, mesh_id: &MeshId, indices: &[u32]) -> Range<usize> {
        self.index_buffer.add(mesh_id, indices)
    }
    pub fn add_mesh_data(
        &mut self,
        mesh_id: &MeshId,
        mesh_data: &MeshData,
    ) -> (Range<usize>, Range<usize>) {
        if mesh_data.vertices.is_empty() {
            self.remove_mesh(mesh_id);
            return (0..0, 0..0);
        }
        let vertices_range = self.set_vertices(mesh_id, mesh_data.vertices.as_slice());
        let indices_range = self.set_indices(mesh_id, mesh_data.indices.as_slice());
        (vertices_range, indices_range)
    }
    fn remove_mesh_data(&mut self, mesh_id: &MeshId) {
        self.vertex_buffer.remove(mesh_id);
        self.index_buffer.remove(mesh_id);

        if self.vertex_buffer.is_empty() {
            self.vertex_buffer.clear();
        }
        if self.index_buffer.is_empty() {
            self.index_buffer.clear();
        }
    }
    pub fn update_mesh(&mut self, mesh_id: &MeshId, mesh: &Mesh) {
        if let Some(material) = mesh.material() {
            if let Some(pipeline) = material.get().pipeline() {
                if !mesh.is_visible() {
                    self.remove_mesh_from_instances(mesh_id, pipeline.id());
                } else {
                    self.add_mesh_to_instances(mesh_id, mesh, pipeline.id());
                }
            }
        }
    }
    fn add_mesh_to_instances(
        &mut self,
        mesh_id: &MeshId,
        mesh: &Mesh,
        pipeline_id: &PipelineId,
    ) -> usize {
        let instance_buffer = self.instance_buffers.entry(*pipeline_id).or_default();
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

        let mut index = instance_buffer.0.insert(mesh_id, instance);
        if mesh.draw_index() >= 0 {
            index = mesh.draw_index() as usize;
            instance_buffer.0.move_to(mesh_id, index);
        }
        index
    }
    fn remove_mesh_from_instances(&mut self, mesh_id: &MeshId, pipeline_id: &PipelineId) {
        if let Some(instance_buffer) = self.instance_buffers.get_mut(pipeline_id) {
            instance_buffer.0.remove(mesh_id);
        }
    }
    pub fn remove_mesh(&mut self, mesh_id: &MeshId) {
        self.remove_mesh_data(mesh_id);
        self.instance_buffers.iter_mut().for_each(|(_, b)| {
            b.0.remove(mesh_id);
        });
    }
    fn clear(&mut self) {
        self.vertex_buffer.clear();
        self.index_buffer.clear();
    }

    pub fn instance_buffer(&self, pipeline_id: &PipelineId) -> Option<wgpu::BufferSlice> {
        self.instance_buffers
            .get(pipeline_id)
            .map(|b| b.1.gpu_buffer().slice(..))
    }
    pub fn for_each_instance(
        &self,
        pipeline_id: &PipelineId,
        mut f: impl FnMut(&MeshId, usize, &InstanceData, &Range<usize>, &Range<usize>),
    ) {
        if let Some(buffer) = self.instance_buffers.get(pipeline_id) {
            buffer.0.for_each_item(|mesh_id, index, instance_data| {
                if let Some(vertices_range) = self
                    .vertex_buffer
                    .get(mesh_id)
                    .as_ref()
                    .map(|buffer_data| &buffer_data.range)
                {
                    if let Some(indices_range) = self
                        .index_buffer
                        .get(mesh_id)
                        .as_ref()
                        .map(|buffer_data| &buffer_data.range)
                    {
                        f(mesh_id, index, instance_data, vertices_range, indices_range);
                    }
                }
            });
        }
    }
    pub fn instance_count(&self, pipeline_id: &PipelineId) -> usize {
        if let Some(buffer) = self.instance_buffers.get(pipeline_id) {
            return buffer.0.item_count();
        }
        0
    }
    pub fn commands_count(&self, pipeline_id: &PipelineId) -> usize {
        self.commands_buffers
            .get(pipeline_id)
            .map(|(c, _)| c.len())
            .unwrap_or_default()
    }
    pub fn commands_buffer(&self, pipeline_id: &PipelineId) -> Option<&wgpu::Buffer> {
        self.commands_buffers
            .get(pipeline_id)
            .map(|(_, d)| d.gpu_buffer())
    }

    pub fn fill_command_buffer(
        &mut self,
        context: &RenderContext,
        pipeline_id: &PipelineId,
    ) -> u64 {
        let instances = self.instance_buffers.entry(*pipeline_id).or_default();
        let commands_buffer = self.commands_buffers.entry(*pipeline_id).or_default();

        commands_buffer.0.clear();
        commands_buffer.0.reserve(instances.0.item_count());
        for i in 0..instances.0.buffer_len() {
            if let Some(mesh_id) = instances.0.id(i) {
                if let Some(vertices_range) = self
                    .vertex_buffer
                    .get(&mesh_id)
                    .as_ref()
                    .map(|buffer_data| &buffer_data.range)
                {
                    if let Some(indices_range) = self
                        .index_buffer
                        .get(&mesh_id)
                        .as_ref()
                        .map(|buffer_data| &buffer_data.range)
                    {
                        commands_buffer.0.push(wgpu::util::DrawIndexedIndirect {
                            vertex_count: 1 + indices_range.len() as u32,
                            instance_count: 1,
                            base_index: indices_range.start as _,
                            vertex_offset: vertices_range.start as _,
                            base_instance: i as _,
                        });
                    }
                }
            }
        }
        let commands_count = commands_buffer.0.len() as u64;

        let total_size = size_of::<wgpu::util::DrawIndexedIndirect>() as u64 * commands_count;

        commands_buffer.1.init::<wgpu::util::DrawIndexedIndirect>(
            context,
            total_size,
            wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
        );
        commands_buffer
            .1
            .add_to_gpu_buffer(context, commands_buffer.0.as_slice());
        commands_count
    }

    pub fn for_each_vertex_buffer_data<F>(&self, mut f: F)
    where
        F: FnMut(&MeshId, &Range<usize>),
    {
        self.vertex_buffer.for_each_occupied(&mut f);
        self.vertex_buffer.for_each_free(&mut f);
    }

    pub fn send_to_gpu(&mut self, context: &RenderContext) {
        inox_profiler::scoped_profile!("graphics_mesh::send_to_gpu");

        self.vertex_buffer
            .send_to_gpu(&context.device, &context.queue);
        self.index_buffer
            .send_to_gpu(&context.device, &context.queue);

        self.instance_buffers
            .iter_mut()
            .for_each(|(_pipeline_id, b)| {
                let total_size = size_of::<InstanceData>() as u64 * b.0.buffer_len() as u64;
                b.1.init::<InstanceData>(
                    context,
                    total_size,
                    wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                );
                b.1.add_to_gpu_buffer(context, b.0.data());
            });
    }
}
