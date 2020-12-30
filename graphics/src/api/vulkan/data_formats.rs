use vulkan_bindings::*;
use crate::data_formats::*;

impl VertexData {
    pub fn get_binding_desc() -> VkVertexInputBindingDescription {
        VkVertexInputBindingDescription {
            binding: 0,
            stride: ::std::mem::size_of::<VertexData>() as _,
            inputRate: VkVertexInputRate_VK_VERTEX_INPUT_RATE_VERTEX,
        }
    }

    pub fn get_attributes_desc() -> Vec<VkVertexInputAttributeDescription> {
        let mut attr: Vec<VkVertexInputAttributeDescription> = Vec::new();
        attr.push( VkVertexInputAttributeDescription {
            binding: 0,
            format: VkFormat_VK_FORMAT_R32G32B32_SFLOAT,
            location: 0,
            offset: unsafe { &(*(::std::ptr::null::<VertexData>())).pos as *const _ as _ },
        });
        attr.push( VkVertexInputAttributeDescription {
            binding: 0,
            format: VkFormat_VK_FORMAT_R32G32B32_SFLOAT,
            location: 1,
            offset: unsafe { &(*(::std::ptr::null::<VertexData>())).color as *const _ as _ },
        });
        attr.push( VkVertexInputAttributeDescription {
            binding: 0,
            format: VkFormat_VK_FORMAT_R32G32_SFLOAT,
            location: 2,
            offset: unsafe { &(*(::std::ptr::null::<VertexData>())).tex_coord as *const _ as _ },
        });
        attr.push( VkVertexInputAttributeDescription {
            binding: 0,
            format: VkFormat_VK_FORMAT_R32G32B32_SFLOAT,
            location: 3,
            offset: unsafe { &(*(::std::ptr::null::<VertexData>())).normal as *const _ as _ },
        });
        attr
    }
}