use super::{data_formats::VERTEX_BUFFER_BIND_ID, device::*};
use crate::common::data_formats::*;
use vulkan_bindings::*;

#[derive(Clone)]
pub struct Mesh {
    vertex_count: u32,
    vertex_buffer: VkBuffer,
    vertex_buffer_memory: VkDeviceMemory,
    indices_count: u32,
    index_buffer: VkBuffer,
    index_buffer_memory: VkDeviceMemory,
}

impl Default for Mesh {
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

impl Mesh {
    pub fn delete(&self, device: &Device) {
        device.destroy_buffer(&self.vertex_buffer, &self.vertex_buffer_memory);
        device.destroy_buffer(&self.index_buffer, &self.index_buffer_memory);
    }

    pub fn create_vertex_buffer(&mut self, device: &Device, vertices: &[VertexData]) {
        if self.vertex_buffer != ::std::ptr::null_mut() {
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

    pub fn create_index_buffer(&mut self, device: &Device, indices: &[u32]) {
        if self.index_buffer != ::std::ptr::null_mut() {
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
        device: &Device,
        vertices: &[VertexData],
        num_vertices: u32,
        indices: &[u32],
        num_indices: u32,
    ) {
        self.bind_vertices(device, vertices, num_vertices);
        self.bind_indices(device, indices, num_indices);

        unsafe {
            let command_buffer = device.get_current_command_buffer();
            if self.index_buffer != ::std::ptr::null_mut() && num_indices > 0 {
                vkCmdDrawIndexed.unwrap()(command_buffer, num_indices as _, 1, 0, 0, 0);
            } else {
                vkCmdDraw.unwrap()(command_buffer, num_vertices as _, 1, 0, 0);
            }
        }
    }

    pub fn bind_vertices(&mut self, device: &Device, vertices: &[VertexData], num_vertices: u32) {
        if num_vertices == 0 {
            return;
        }
        if num_vertices >= self.vertex_count as _ {
            panic!("Trying to render more vertices then allocated ones");
        } else {
            device.map_buffer_memory(&mut self.vertex_buffer_memory, &vertices);
        }
        if self.vertex_buffer != ::std::ptr::null_mut() {
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

    pub fn bind_indices(&mut self, device: &Device, indices: &[u32], num_indices: u32) {
        if num_indices == 0 {
            return;
        }
        if num_indices >= self.indices_count as _ {
            panic!("Trying to render more indices then allocated ones");
        } else {
            device.map_buffer_memory(&mut self.index_buffer_memory, &indices);
        }
        if self.index_buffer != ::std::ptr::null_mut() {
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
