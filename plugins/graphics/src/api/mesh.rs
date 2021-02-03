use nrg_math::*;
use super::data_formats::*;
use super::device::*;

pub struct Mesh {
    pub inner: super::backend::mesh::Mesh,
    pub vertices: Vec<VertexData>,
    pub indices: Vec<u32>,
    device: Device,
}

impl Mesh {
    pub fn create(device:&Device) -> Mesh {
        Self {
            inner: super::backend::mesh::Mesh::default(),
            vertices: Vec::new(),
            indices: Vec::new(),
            device: device.clone(),
        }
    }

    pub fn set_vertices(&mut self, vertex_data:&[VertexData]) -> &mut Self {
        self.vertices.clear();
        self.vertices.extend_from_slice(vertex_data);
        self
    }

    pub fn set_indices(&mut self, indices_data:&[u32]) -> &mut Self {
        self.indices.clear();
        self.indices.extend_from_slice(indices_data);
        self
    }
    
    pub fn destroy(&mut self) {
        let inner_device = self.device.inner.borrow();
        self.vertices.clear();
        self.indices.clear();
        self.inner.delete(&inner_device);
    }

    pub fn finalize(&mut self) -> &mut Self {
        let inner_device = self.device.inner.borrow();
        self.inner.create_vertex_buffer(&inner_device, self.vertices.as_slice());
        self.inner.create_index_buffer(&inner_device, self.indices.as_slice());
        drop(inner_device);
        self
    }

    pub fn draw(&self) {
        let inner_device = self.device.inner.borrow();
        self.inner.draw(&inner_device);
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

    pub fn set_vertex_color(&mut self, color:Vector3f) -> &mut Self {
        for v in self.vertices.iter_mut() {
            v.color = color;
        }
        self
    } 
}