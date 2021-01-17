use vulkan_bindings::*;
use crate::data_formats::*;
use super::device::*;

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
        if self.vertex_buffer != std::ptr::null_mut() {
            device.destroy_buffer(&self.vertex_buffer, &self.vertex_buffer_memory);
            self.vertex_count = 0;
        }

        let length = ::std::mem::size_of::<VertexData>() * vertices.len();
        let flags = VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_HOST_COHERENT_BIT;
        
        let mut staging_buffer: VkBuffer = ::std::ptr::null_mut();
        let mut staging_buffer_memory : VkDeviceMemory = ::std::ptr::null_mut();
        device.create_buffer(length as _, 
                            VkBufferUsageFlagBits_VK_BUFFER_USAGE_TRANSFER_SRC_BIT as _, 
                            flags as _,
                            &mut staging_buffer,
                            &mut staging_buffer_memory);
        
        device.map_buffer_memory(&mut staging_buffer_memory, &vertices);

        let mut vertex_buffer: VkBuffer = ::std::ptr::null_mut();
        let mut vertex_buffer_memory : VkDeviceMemory = ::std::ptr::null_mut();
        let flags = VkBufferUsageFlagBits_VK_BUFFER_USAGE_TRANSFER_DST_BIT | VkBufferUsageFlagBits_VK_BUFFER_USAGE_VERTEX_BUFFER_BIT; 
        device.create_buffer(length as _, 
                            flags as _, 
                            VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT as _,
                            &mut vertex_buffer,
                            &mut vertex_buffer_memory);

        device.copy_buffer(&staging_buffer, &mut vertex_buffer, length as _);
        
        device.destroy_buffer(&staging_buffer, &staging_buffer_memory);

        self.vertex_count = vertices.len() as _;
        self.vertex_buffer = vertex_buffer;
        self.vertex_buffer_memory = vertex_buffer_memory;
    }

    pub fn create_index_buffer(&mut self, device: &Device, indices: &[u32]) {
        let length = ::std::mem::size_of::<u32>() * indices.len();
        let flags = VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_HOST_COHERENT_BIT;
        
        let mut staging_buffer: VkBuffer = ::std::ptr::null_mut();
        let mut staging_buffer_memory : VkDeviceMemory = ::std::ptr::null_mut();
        device.create_buffer(length as _, 
                            VkBufferUsageFlagBits_VK_BUFFER_USAGE_TRANSFER_SRC_BIT as _, 
                            flags as _,
                            &mut staging_buffer,
                            &mut staging_buffer_memory);
        
        device.map_buffer_memory(&mut staging_buffer_memory, &indices);

        let mut index_buffer: VkBuffer = ::std::ptr::null_mut();
        let mut index_buffer_memory : VkDeviceMemory = ::std::ptr::null_mut();
        let flags = VkBufferUsageFlagBits_VK_BUFFER_USAGE_TRANSFER_DST_BIT | VkBufferUsageFlagBits_VK_BUFFER_USAGE_INDEX_BUFFER_BIT;
        device.create_buffer(length as _, 
                            flags as _, 
                            VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT as _,
                            &mut index_buffer,
                            &mut index_buffer_memory);

        device.copy_buffer(&staging_buffer, &mut index_buffer, length as _);
        
        device.destroy_buffer(&staging_buffer, &staging_buffer_memory);

        self.indices_count = indices.len() as _;
        self.index_buffer = index_buffer;
        self.index_buffer_memory = index_buffer_memory;
    }    

    pub fn draw(&self, device:&Device) {      
        unsafe {  
            let vertex_buffers = [self.vertex_buffer];
            let offsets = [0_u64];
            vkCmdBindVertexBuffers.unwrap()(device.get_current_command_buffer(), 0, 1, vertex_buffers.as_ptr(), offsets.as_ptr());

            if self.index_buffer != std::ptr::null_mut() {
                vkCmdBindIndexBuffer.unwrap()(device.get_current_command_buffer(), self.index_buffer, 0, VkIndexType_VK_INDEX_TYPE_UINT32);
                vkCmdDrawIndexed.unwrap()(device.get_current_command_buffer(), self.indices_count as _, 1, 0, 0, 0);
            }
            else {
                vkCmdDraw.unwrap()(device.get_current_command_buffer(), self.vertex_count, 1, 0, 0);
            }
        }
    }
}
