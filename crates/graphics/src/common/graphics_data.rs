use std::{collections::HashMap, mem::size_of, ops::Range};

use inox_math::matrix4_to_array;
use inox_messenger::MessageHubRc;
use inox_resources::{HashBuffer, ResourceId, ResourceTrait, SharedData, SharedDataRc};
use wgpu::util::DrawIndexedIndirect;

use crate::{
    AsBufferBinding, DataBuffer, GpuBuffer, InstanceData, Mesh, MeshData, MeshId, MeshletData,
    RenderContext, RenderCoreContext, RenderPipelineId, VertexFormatBits, INVALID_INDEX,
};

pub const GRAPHICS_DATA_UID: ResourceId =
    inox_uid::generate_static_uid_from_string(stringify!(GraphicsData));

#[repr(C)]
#[derive(Default, Clone)]
pub struct Meshlets {
    is_dirty: bool,
    pub data: Vec<MeshletData>,
}

impl AsBufferBinding for Meshlets {
    fn is_dirty(&self) -> bool {
        self.is_dirty
    }
    fn set_dirty(&mut self, is_dirty: bool) {
        self.is_dirty = is_dirty;
    }
    fn size(&self) -> u64 {
        (std::mem::size_of::<MeshletData>() * self.data.len()) as _
    }

    fn fill_buffer(&self, render_core_context: &RenderCoreContext, buffer: &mut DataBuffer) {
        buffer.add_to_gpu_buffer(render_core_context, self.data.as_slice());
    }
}

#[derive(Default)]
struct MeshBuffers {
    vertex_buffer: GpuBuffer<{ wgpu::BufferUsages::bits(&wgpu::BufferUsages::VERTEX) }>,
    index_buffer: GpuBuffer<{ wgpu::BufferUsages::bits(&wgpu::BufferUsages::INDEX) }>,
    meshlets: Meshlets,
}

#[derive(Default)]
struct MeshletsInfo {
    first_meshlet: usize,
    meshlet_count: usize,
}

#[derive(Default)]
struct PipelineBuffers {
    vertex_format: VertexFormatBits,
    instance_buffer: (HashBuffer<MeshId, InstanceData, 0>, DataBuffer),
    commands_buffer: (Vec<DrawIndexedIndirect>, DataBuffer),
    is_dirty: bool,
}

#[derive(Default)]
pub struct GraphicsData {
    mesh_buffers: HashMap<VertexFormatBits, MeshBuffers>,
    pipeline_buffers: HashMap<RenderPipelineId, PipelineBuffers>,
    meshlets_info: HashBuffer<MeshId, MeshletsInfo, 0>,
}

impl ResourceTrait for GraphicsData {
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

unsafe impl Send for GraphicsData {}
unsafe impl Sync for GraphicsData {}

impl GraphicsData {
    pub fn is_empty(&self, pipeline_id: &RenderPipelineId) -> bool {
        self.pipeline_buffers
            .get(pipeline_id)
            .map(|pb| {
                if pb.vertex_format == 0 {
                    return true;
                }
                self.mesh_buffers
                    .get(&pb.vertex_format)
                    .map(|mb| mb.vertex_buffer.is_empty())
                    .unwrap_or(true)
            })
            .unwrap_or(true)
    }
    pub fn vertex_count(&self, pipeline_id: &RenderPipelineId) -> usize {
        self.pipeline_buffers
            .get(pipeline_id)
            .map(|pb| {
                if pb.vertex_format == 0 {
                    return 0;
                }
                self.mesh_buffers
                    .get(&pb.vertex_format)
                    .map(|mb| mb.vertex_buffer.len())
                    .unwrap_or(0)
            })
            .unwrap_or(0)
    }
    pub fn index_count(&self, pipeline_id: &RenderPipelineId) -> usize {
        self.pipeline_buffers
            .get(pipeline_id)
            .map(|pb| {
                if pb.vertex_format == 0 {
                    return 0;
                }
                self.mesh_buffers
                    .get(&pb.vertex_format)
                    .map(|mb| mb.index_buffer.len())
                    .unwrap_or(0)
            })
            .unwrap_or(0)
    }
    pub fn total_vertex_count(&self) -> usize {
        let mut count = 0;
        self.mesh_buffers
            .iter()
            .for_each(|(_, mb)| count += mb.vertex_buffer.len());
        count
    }
    pub fn total_index_count(&self) -> usize {
        let mut count = 0;
        self.mesh_buffers
            .iter()
            .for_each(|(_, mb)| count += mb.index_buffer.len());
        count
    }
    pub fn vertex_buffer(&self, pipeline_id: &RenderPipelineId) -> Option<wgpu::BufferSlice> {
        self.pipeline_buffers
            .get(pipeline_id)
            .map(|pb| {
                if pb.vertex_format == 0 {
                    return None;
                }
                self.mesh_buffers
                    .get(&pb.vertex_format)
                    .map(|mb| mb.vertex_buffer.gpu_buffer().map(|buffer| buffer.slice(..)))
                    .unwrap_or(None)
            })
            .unwrap_or(None)
    }
    pub fn index_buffer(&self, pipeline_id: &RenderPipelineId) -> Option<wgpu::BufferSlice> {
        self.pipeline_buffers
            .get(pipeline_id)
            .map(|pb| {
                if pb.vertex_format == 0 {
                    return None;
                }
                self.mesh_buffers
                    .get(&pb.vertex_format)
                    .map(|mb| mb.index_buffer.gpu_buffer().map(|buffer| buffer.slice(..)))
                    .unwrap_or(None)
            })
            .unwrap_or(None)
    }
    pub fn add_vertices(
        &mut self,
        mesh_id: &MeshId,
        attributes_hash: VertexFormatBits,
        vertex_size: usize,
        vertices: &[u8],
    ) -> Range<usize> {
        let entry = self.mesh_buffers.entry(attributes_hash).or_default();
        entry
            .vertex_buffer
            .add_with_size(mesh_id, vertices, vertex_size)
    }
    pub fn add_indices(
        &mut self,
        mesh_id: &MeshId,
        attributes_hash: VertexFormatBits,
        indices: &[u32],
    ) -> Range<usize> {
        let entry = self.mesh_buffers.entry(attributes_hash).or_default();
        entry.index_buffer.add(mesh_id, indices)
    }
    fn add_meshlets(
        &mut self,
        mesh_id: &MeshId,
        attributes_hash: VertexFormatBits,
        meshlets: &[MeshletData],
    ) {
        let entry = self.mesh_buffers.entry(attributes_hash).or_default();
        let first_index = entry.meshlets.data.len();
        meshlets.iter().for_each(|meshlet| {
            entry.meshlets.data.push(meshlet.clone());
        });
        entry.meshlets.set_dirty(true);
        self.meshlets_info.insert(
            mesh_id,
            MeshletsInfo {
                first_meshlet: first_index,
                meshlet_count: meshlets.len(),
            },
        );
    }
    pub fn get_meshlets(&mut self, attributes_hash: &VertexFormatBits) -> Option<&mut Meshlets> {
        self.mesh_buffers
            .get_mut(attributes_hash)
            .map(|mb| &mut mb.meshlets)
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
        let vertices_range = self.add_vertices(
            mesh_id,
            mesh_data.vertex_format(),
            mesh_data.vertex_size(),
            mesh_data.vertices.as_slice(),
        );
        let indices_range = self.add_indices(
            mesh_id,
            mesh_data.vertex_format(),
            mesh_data.indices.as_slice(),
        );
        self.add_meshlets(
            mesh_id,
            mesh_data.vertex_format(),
            mesh_data.meshlets.as_slice(),
        );
        (vertices_range, indices_range)
    }
    fn remove_mesh_data(&mut self, mesh_id: &MeshId) {
        self.mesh_buffers.iter_mut().for_each(|(_, mb)| {
            mb.vertex_buffer.remove(mesh_id);
            mb.index_buffer.remove(mesh_id);
            let mut start = 0usize;
            let mut count = 0usize;
            if let Some(meshlet_info) = self.meshlets_info.remove(mesh_id) {
                start = meshlet_info.first_meshlet;
                count = meshlet_info.meshlet_count;
                mb.meshlets.data.drain(start..(start + count));
                mb.meshlets.set_dirty(true);
            }
            if count > 0 {
                self.meshlets_info.for_each_item_mut(|_id, _index, info| {
                    if info.first_meshlet > start {
                        info.first_meshlet -= count;
                    }
                });
            }
        });
    }
    pub fn set_pipeline_vertex_format(
        &mut self,
        pipeline_id: &RenderPipelineId,
        vertex_format: VertexFormatBits,
    ) {
        let pb = self.pipeline_buffers.entry(*pipeline_id).or_default();
        pb.vertex_format = vertex_format;
    }
    pub fn update_mesh(&mut self, mesh_id: &MeshId, mesh: &Mesh) {
        if let Some(material) = mesh.material() {
            if let Some(pipeline) = material.get().pipeline() {
                if let Some(pb) = self.pipeline_buffers.get(pipeline.id()) {
                    if pb.vertex_format == 0 {
                        inox_log::debug_log!("Pipeline not yet loaded");
                    }
                    if pb.vertex_format != mesh.vertex_format() {
                        panic!(
                            "Mesh vertex format mismatch - Pipeline is {:?}, mesh is {:?}",
                            pb.vertex_format,
                            mesh.vertex_format()
                        );
                    }
                } else {
                    return;
                }
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
        pipeline_id: &RenderPipelineId,
    ) -> usize {
        let pb = self.pipeline_buffers.get_mut(pipeline_id).unwrap();
        /*
        let euler_angles = mesh.matrix().rotation();
        let rotation = Quaternion::from_euler_angles(euler_angles);
        let normal_matrix = Matrix3::from(rotation);
        */
        let instance = InstanceData {
            matrix: matrix4_to_array(mesh.matrix()),
            draw_area: mesh.draw_area().into(),
            material_index: mesh
                .material()
                .as_ref()
                .map_or(INVALID_INDEX, |m| m.get().uniform_index()),
        };

        let mut index = pb.instance_buffer.0.insert(mesh_id, instance);
        if mesh.draw_index() >= 0 {
            index = mesh.draw_index() as usize;
            pb.instance_buffer.0.move_to(mesh_id, index);
        }
        pb.is_dirty = true;
        index
    }
    fn remove_mesh_from_instances(&mut self, mesh_id: &MeshId, pipeline_id: &RenderPipelineId) {
        if let Some(pb) = self.pipeline_buffers.get_mut(pipeline_id) {
            pb.instance_buffer.0.remove(mesh_id);
            pb.is_dirty = true;
        }
    }
    pub fn remove_mesh(&mut self, mesh_id: &MeshId) {
        self.remove_mesh_data(mesh_id);
        self.pipeline_buffers.iter_mut().for_each(|(_, pb)| {
            pb.instance_buffer.0.remove(mesh_id);
            pb.is_dirty = true;
        });
    }
    pub fn instance_buffer(&self, pipeline_id: &RenderPipelineId) -> Option<wgpu::BufferSlice> {
        self.pipeline_buffers
            .get(pipeline_id)
            .map(|pb| pb.instance_buffer.1.gpu_buffer().map(|b| b.slice(..)))
            .unwrap_or(None)
    }
    pub fn for_each_instance(
        &self,
        pipeline_id: &RenderPipelineId,
        mut f: impl FnMut(&MeshId, usize, &InstanceData, &Range<usize>, &Range<usize>),
    ) {
        if let Some(pb) = self.pipeline_buffers.get(pipeline_id) {
            if pb.vertex_format == 0 {
                return;
            }
            pb.instance_buffer
                .0
                .for_each_item(|mesh_id, index, instance_data| {
                    if let (Some(vertices_range), Some(indices_range)) = self
                        .mesh_buffers
                        .get(&pb.vertex_format)
                        .map(|mb| {
                            (
                                mb.vertex_buffer
                                    .get(mesh_id)
                                    .map(|buffer_data| buffer_data.item_range()),
                                mb.index_buffer
                                    .get(mesh_id)
                                    .map(|buffer_data| buffer_data.item_range()),
                            )
                        })
                        .unwrap_or((None, None))
                    {
                        f(
                            mesh_id,
                            index,
                            instance_data,
                            &vertices_range,
                            &indices_range,
                        );
                    }
                });
        }
    }
    pub fn instance_count(&self, pipeline_id: &RenderPipelineId) -> usize {
        self.pipeline_buffers
            .get(pipeline_id)
            .map(|pb| pb.instance_buffer.0.item_count())
            .unwrap_or(0)
    }
    pub fn commands_count(&self, pipeline_id: &RenderPipelineId) -> usize {
        self.pipeline_buffers
            .get(pipeline_id)
            .map(|pb| pb.commands_buffer.0.len())
            .unwrap_or(0)
    }
    pub fn commands_buffer(&self, pipeline_id: &RenderPipelineId) -> Option<&wgpu::Buffer> {
        self.pipeline_buffers
            .get(pipeline_id)
            .map(|pb| pb.commands_buffer.1.gpu_buffer())
            .unwrap_or(None)
    }

    pub fn fill_command_buffer(
        &mut self,
        render_context: &RenderContext,
        pipeline_id: &RenderPipelineId,
    ) -> u64 {
        inox_profiler::scoped_profile!("graphics_data::fill_command_buffer");
        if self.pipeline_buffers.get(pipeline_id).is_none() {
            return 0;
        }
        let pb = self.pipeline_buffers.get_mut(pipeline_id).unwrap();
        if pb.vertex_format == 0 || pb.instance_buffer.0.item_count() == 0 {
            return 0;
        }
        if !pb.is_dirty {
            return pb.commands_buffer.0.len() as _;
        }
        pb.commands_buffer.0.clear();
        pb.is_dirty = false;

        if let Some(mb) = self.mesh_buffers.get(&pb.vertex_format) {
            pb.commands_buffer
                .0
                .reserve(pb.instance_buffer.0.item_count());

            for i in 0..pb.instance_buffer.0.buffer_len() {
                if let Some(mesh_id) = pb.instance_buffer.0.id(i) {
                    if let Some(vertices_range) = mb
                        .vertex_buffer
                        .get(&mesh_id)
                        .as_ref()
                        .map(|buffer_data| buffer_data.item_range())
                    {
                        if let Some((base_index, vertex_count)) =
                            mb.index_buffer.get(&mesh_id).as_ref().map(|buffer_data| {
                                (
                                    buffer_data.item_range().start as u32,
                                    1 + buffer_data.item_count() as u32,
                                )
                            })
                        {
                            if let Some(meshlet_info) = self.meshlets_info.get(&mesh_id) {
                                let start = meshlet_info.first_meshlet;
                                let end = meshlet_info.first_meshlet + meshlet_info.meshlet_count;
                                mb.meshlets.data[start..end].iter().for_each(|meshlet| {
                                    pb.commands_buffer.0.push(wgpu::util::DrawIndexedIndirect {
                                        vertex_count: meshlet.indices_count as _,
                                        instance_count: 1,
                                        base_index: (base_index + meshlet.indices_offset),
                                        vertex_offset: vertices_range.start as _,
                                        base_instance: i as _,
                                    });
                                });
                            } else {
                                pb.commands_buffer.0.push(wgpu::util::DrawIndexedIndirect {
                                    vertex_count,
                                    instance_count: 1,
                                    base_index,
                                    vertex_offset: vertices_range.start as _,
                                    base_instance: i as _,
                                });
                            }
                        }
                    }
                }
            }
            let commands_count = pb.commands_buffer.0.len() as u64;
            if commands_count > 0 {
                let total_size =
                    size_of::<wgpu::util::DrawIndexedIndirect>() as u64 * commands_count;

                pb.commands_buffer
                    .1
                    .init_from_type::<wgpu::util::DrawIndexedIndirect>(
                        &render_context.core,
                        total_size,
                        wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
                    );
                pb.commands_buffer
                    .1
                    .add_to_gpu_buffer(&render_context.core, pb.commands_buffer.0.as_slice());
                return commands_count;
            }
        }
        0
    }

    pub fn for_each_vertex_buffer_data<F>(&self, pipeline_id: &RenderPipelineId, mut f: F)
    where
        F: FnMut(&MeshId, &Range<usize>),
    {
        if let Some(pb) = self.pipeline_buffers.get(pipeline_id) {
            if pb.vertex_format == 0 {
                return;
            }
            if let Some(mb) = self.mesh_buffers.get(&pb.vertex_format) {
                mb.vertex_buffer.for_each_occupied(&mut f);
                mb.vertex_buffer.for_each_free(&mut f);
            }
        }
    }

    pub fn send_to_gpu(&mut self, render_context: &RenderContext) {
        inox_profiler::scoped_profile!("graphics_data::send_to_gpu");

        self.mesh_buffers.iter_mut().for_each(|(_, mb)| {
            mb.vertex_buffer
                .send_to_gpu(&render_context.core.device, &render_context.core.queue);
            mb.index_buffer
                .send_to_gpu(&render_context.core.device, &render_context.core.queue);
        });

        self.pipeline_buffers.iter_mut().for_each(|(_, pb)| {
            let total_size =
                size_of::<InstanceData>() as u64 * pb.instance_buffer.0.buffer_len() as u64;
            if total_size > 0 {
                pb.instance_buffer.1.init_from_type::<InstanceData>(
                    &render_context.core,
                    total_size,
                    wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                );
                pb.instance_buffer
                    .1
                    .add_to_gpu_buffer(&render_context.core, pb.instance_buffer.0.data());
            }
        });
    }
}
