use crate::common::data_formats::*;
use vulkan_bindings::*;

pub const VERTEX_BUFFER_BIND_ID: usize = 0;
pub const INSTANCE_BUFFER_BIND_ID: usize = 1;

impl VertexData {
    pub fn get_binding_desc() -> VkVertexInputBindingDescription {
        VkVertexInputBindingDescription {
            binding: VERTEX_BUFFER_BIND_ID as _,
            stride: ::std::mem::size_of::<VertexData>() as _,
            inputRate: VkVertexInputRate_VK_VERTEX_INPUT_RATE_VERTEX,
        }
    }

    pub fn get_attributes_desc() -> Vec<VkVertexInputAttributeDescription> {
        let attr: Vec<VkVertexInputAttributeDescription> = vec![
            VkVertexInputAttributeDescription {
                binding: VERTEX_BUFFER_BIND_ID as _,
                format: VkFormat_VK_FORMAT_R32G32B32_SFLOAT,
                location: 0,
                offset: unsafe { &(*(::std::ptr::null::<VertexData>())).pos as *const _ as _ },
            },
            VkVertexInputAttributeDescription {
                binding: VERTEX_BUFFER_BIND_ID as _,
                format: VkFormat_VK_FORMAT_R32G32B32A32_SFLOAT,
                location: 1,
                offset: unsafe { &(*(::std::ptr::null::<VertexData>())).color as *const _ as _ },
            },
            VkVertexInputAttributeDescription {
                binding: VERTEX_BUFFER_BIND_ID as _,
                format: VkFormat_VK_FORMAT_R32G32_SFLOAT,
                location: 2,
                offset: unsafe {
                    &(*(::std::ptr::null::<VertexData>())).tex_coord as *const _ as _
                },
            },
            VkVertexInputAttributeDescription {
                binding: VERTEX_BUFFER_BIND_ID as _,
                format: VkFormat_VK_FORMAT_R32G32B32_SFLOAT,
                location: 3,
                offset: unsafe { &(*(::std::ptr::null::<VertexData>())).normal as *const _ as _ },
            },
        ];
        attr
    }
}

impl InstanceData {
    pub fn get_binding_desc() -> VkVertexInputBindingDescription {
        VkVertexInputBindingDescription {
            binding: INSTANCE_BUFFER_BIND_ID as _,
            stride: ::std::mem::size_of::<InstanceData>() as _,
            inputRate: VkVertexInputRate_VK_VERTEX_INPUT_RATE_INSTANCE,
        }
    }

    pub fn get_attributes_desc() -> Vec<VkVertexInputAttributeDescription> {
        let attr: Vec<VkVertexInputAttributeDescription> = vec![
            VkVertexInputAttributeDescription {
                binding: INSTANCE_BUFFER_BIND_ID as _,
                format: VkFormat_VK_FORMAT_R32G32B32A32_SFLOAT,
                location: 4,
                offset: unsafe {
                    &(*(::std::ptr::null::<InstanceData>())).transform.x as *const _ as _
                },
            },
            VkVertexInputAttributeDescription {
                binding: INSTANCE_BUFFER_BIND_ID as _,
                format: VkFormat_VK_FORMAT_R32G32B32A32_SFLOAT,
                location: 5,
                offset: unsafe {
                    &(*(::std::ptr::null::<InstanceData>())).transform.y as *const _ as _
                },
            },
            VkVertexInputAttributeDescription {
                binding: INSTANCE_BUFFER_BIND_ID as _,
                format: VkFormat_VK_FORMAT_R32G32B32A32_SFLOAT,
                location: 6,
                offset: unsafe {
                    &(*(::std::ptr::null::<InstanceData>())).transform.z as *const _ as _
                },
            },
            VkVertexInputAttributeDescription {
                binding: INSTANCE_BUFFER_BIND_ID as _,
                format: VkFormat_VK_FORMAT_R32G32B32A32_SFLOAT,
                location: 7,
                offset: unsafe {
                    &(*(::std::ptr::null::<InstanceData>())).transform.w as *const _ as _
                },
            },
            VkVertexInputAttributeDescription {
                binding: INSTANCE_BUFFER_BIND_ID as _,
                format: VkFormat_VK_FORMAT_R32_SINT,
                location: 8,
                offset: unsafe {
                    &(*(::std::ptr::null::<InstanceData>())).diffuse_texture_index as *const _ as _
                },
            },
            VkVertexInputAttributeDescription {
                binding: INSTANCE_BUFFER_BIND_ID as _,
                format: VkFormat_VK_FORMAT_R32_SINT,
                location: 9,
                offset: unsafe {
                    &(*(::std::ptr::null::<InstanceData>())).diffuse_layer_index as *const _ as _
                },
            },
        ];
        attr
    }
}
