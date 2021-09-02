use super::{data_formats::VERTEX_BUFFER_BIND_ID, device::*};
use crate::common::data_formats::*;
use vulkan_bindings::*;

#[derive(Clone)]
pub struct BackendMesh {
    vertex_count: u32,
    vertex_buffer: VkBuffer,
    vertex_buffer_memory: VkDeviceMemory,
    indices_count: u32,
    index_buffer: VkBuffer,
    index_buffer_memory: VkDeviceMemory,
}

impl Default for BackendMesh {
    fn default() -> Self {
        Self {
            vertex_count: 0,
            vertex_buffer: ::std::ptr::null_mut(),
            vertex_buffer_memory: ::std::ptr::null_mut(),
            indices_count: 0,
            index_buffer: ::std::ptr::null_mut(),
            index_buffer_memory: ::std::ptr::null_mut(),
        }
    }
}

impl BackendMesh {
    pub fn get_vertex_count(&self) -> u32 {
        self.vertex_count
    }
    pub fn get_indices_count(&self) -> u32 {
        self.indices_count
    }
    pub fn delete(&self, device: &BackendDevice) {
        device.destroy_buffer(&self.vertex_buffer, &self.vertex_buffer_memory);
        device.destroy_buffer(&self.index_buffer, &self.index_buffer_memory);
    }

    pub fn create_vertex_buffer(&mut self, device: &BackendDevice, vertices: &[VertexData]) {
        if !self.vertex_buffer.is_null() {
            device.destroy_buffer(&self.vertex_buffer, &self.vertex_buffer_memory);
            self.vertex_count = 0;
        }

        let length = ::std::mem::size_of::<VertexData>() * vertices.len();
        let flags = VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT
            | VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_HOST_COHERENT_BIT;
        let mut vertex_buffer: VkBuffer = ::std::ptr::null_mut();
        let mut vertex_buffer_memory: VkDeviceMemory = ::std::ptr::null_mut();
        device.create_buffer(
            length as _,
            VkBufferUsageFlagBits_VK_BUFFER_USAGE_VERTEX_BUFFER_BIT as _,
            flags as _,
            &mut vertex_buffer,
            &mut vertex_buffer_memory,
        );

        self.vertex_count = vertices.len() as _;
        self.vertex_buffer = vertex_buffer;
        self.vertex_buffer_memory = vertex_buffer_memory;
    }

    pub fn create_index_buffer(&mut self, device: &BackendDevice, indices: &[u32]) {
        if !self.index_buffer.is_null() {
            device.destroy_buffer(&self.index_buffer, &self.index_buffer_memory);
            self.indices_count = 0;
        }

        let length = ::std::mem::size_of::<u32>() * indices.len();
        let flags = VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT
            | VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_HOST_COHERENT_BIT;
        let mut index_buffer: VkBuffer = ::std::ptr::null_mut();
        let mut index_buffer_memory: VkDeviceMemory = ::std::ptr::null_mut();
        device.create_buffer(
            length as _,
            VkBufferUsageFlagBits_VK_BUFFER_USAGE_INDEX_BUFFER_BIT as _,
            flags as _,
            &mut index_buffer,
            &mut index_buffer_memory,
        );

        self.indices_count = indices.len() as _;
        self.index_buffer = index_buffer;
        self.index_buffer_memory = index_buffer_memory;
    }

    pub fn draw(
        &mut self,
        device: &mut BackendDevice,
        vertices: &[VertexData],
        num_vertices: u32,
        indices: &[u32],
        num_indices: u32,
    ) {
        self.bind_at_index(device, vertices, 0, indices, 0);
        self.bind_vertices(device);
        self.bind_indices(device);

        unsafe {
            let command_buffer = device.get_current_command_buffer();
            if !self.index_buffer.is_null() && num_indices > 0 {
                vkCmdDrawIndexed.unwrap()(command_buffer, num_indices as _, 1, 0, 0, 0);
            } else {
                vkCmdDraw.unwrap()(command_buffer, num_vertices as _, 1, 0, 0);
            }
        }
    }

    pub fn bind_at_index(
        &mut self,
        device: &BackendDevice,
        vertices: &[VertexData],
        first_vertex: u32,
        indices: &[u32],
        first_index: u32,
    ) {
        device.map_buffer_memory(&mut self.vertex_buffer_memory, first_vertex as _, vertices);
        device.map_buffer_memory(&mut self.index_buffer_memory, first_index as _, indices);
    }

    pub fn bind_vertices(&self, device: &mut BackendDevice) {
        if !self.vertex_buffer.is_null() {
            unsafe {
                let vertex_buffers = [self.vertex_buffer];
                let offsets = [0_u64];
                vkCmdBindVertexBuffers.unwrap()(
                    device.get_current_command_buffer(),
                    VERTEX_BUFFER_BIND_ID as _,
                    1,
                    vertex_buffers.as_ptr(),
                    offsets.as_ptr(),
                );
            }
        }
    }

    pub fn bind_indices(&self, device: &mut BackendDevice) {
        if !self.index_buffer.is_null() {
            unsafe {
                vkCmdBindIndexBuffer.unwrap()(
                    device.get_current_command_buffer(),
                    self.index_buffer,
                    0,
                    VkIndexType_VK_INDEX_TYPE_UINT32,
                );
            }
        }
    }
}
