use nrg_math::*;
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

    pub fn create_quad(rect: Vector4f, tex_coords: Vector4f, index_start: Option<usize>) -> (Vec<VertexData>, Vec<u32>) {    
        let vertices: [VertexData; 4] = [
            VertexData { pos: [rect.x, rect.y, 0.].into(), normal: [0., 0., 1.].into(), color: [1., 1., 1.].into(), tex_coord: [tex_coords.z, tex_coords.y].into()},
            VertexData { pos: [rect.z, rect.y, 0.].into(), normal: [0., 0., 1.].into(), color: [1., 1., 1.].into(), tex_coord: [tex_coords.x, tex_coords.y].into()},
            VertexData { pos: [rect.z, rect.w, 0.].into(), normal: [0., 0., 1.].into(), color: [1., 1., 1.].into(), tex_coord: [tex_coords.x, tex_coords.w].into()},
            VertexData { pos: [rect.x, rect.w, 0.].into(), normal: [0., 0., 1.].into(), color: [1., 1., 1.].into(), tex_coord: [tex_coords.z, tex_coords.w].into()},
        ]; 
        let index_offset:u32 = index_start.unwrap_or(0) as _;
        let indices: [u32; 6] = [index_offset, 1 + index_offset, 2 + index_offset, 2 + index_offset, 3 + index_offset, index_offset];

        (vertices.to_vec(), indices.to_vec())
    }
}