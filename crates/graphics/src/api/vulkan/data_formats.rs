use vulkan_bindings::*;

use crate::{BlendFactor, CullingModeType, InstanceData, PolygonModeType, VertexData};

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

    #[allow(deref_nullptr)]
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
                format: VkFormat_VK_FORMAT_R32G32B32_SFLOAT,
                location: 1,
                offset: unsafe { &(*(::std::ptr::null::<VertexData>())).normal as *const _ as _ },
            },
            VkVertexInputAttributeDescription {
                binding: VERTEX_BUFFER_BIND_ID as _,
                format: VkFormat_VK_FORMAT_R32G32B32_SFLOAT,
                location: 2,
                offset: unsafe { &(*(::std::ptr::null::<VertexData>())).tangent as *const _ as _ },
            },
            VkVertexInputAttributeDescription {
                binding: VERTEX_BUFFER_BIND_ID as _,
                format: VkFormat_VK_FORMAT_R32G32B32A32_SFLOAT,
                location: 3,
                offset: unsafe { &(*(::std::ptr::null::<VertexData>())).color as *const _ as _ },
            },
            VkVertexInputAttributeDescription {
                binding: VERTEX_BUFFER_BIND_ID as _,
                format: VkFormat_VK_FORMAT_R32G32_SFLOAT,
                location: 4,
                offset: unsafe {
                    &(*(::std::ptr::null::<VertexData>())).tex_coord[0] as *const _ as _
                },
            },
            VkVertexInputAttributeDescription {
                binding: VERTEX_BUFFER_BIND_ID as _,
                format: VkFormat_VK_FORMAT_R32G32_SFLOAT,
                location: 5,
                offset: unsafe {
                    &(*(::std::ptr::null::<VertexData>())).tex_coord[1] as *const _ as _
                },
            },
            VkVertexInputAttributeDescription {
                binding: VERTEX_BUFFER_BIND_ID as _,
                format: VkFormat_VK_FORMAT_R32G32_SFLOAT,
                location: 6,
                offset: unsafe {
                    &(*(::std::ptr::null::<VertexData>())).tex_coord[2] as *const _ as _
                },
            },
            VkVertexInputAttributeDescription {
                binding: VERTEX_BUFFER_BIND_ID as _,
                format: VkFormat_VK_FORMAT_R32G32_SFLOAT,
                location: 7,
                offset: unsafe {
                    &(*(::std::ptr::null::<VertexData>())).tex_coord[3] as *const _ as _
                },
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

    #[allow(deref_nullptr)]
    pub fn get_attributes_desc() -> Vec<VkVertexInputAttributeDescription> {
        let attr: Vec<VkVertexInputAttributeDescription> = vec![
            VkVertexInputAttributeDescription {
                binding: INSTANCE_BUFFER_BIND_ID as _,
                format: VkFormat_VK_FORMAT_R32G32B32A32_SFLOAT,
                location: 8,
                offset: unsafe { &(*(::std::ptr::null::<InstanceData>())).id as *const _ as _ },
            },
            VkVertexInputAttributeDescription {
                binding: INSTANCE_BUFFER_BIND_ID as _,
                format: VkFormat_VK_FORMAT_R32G32B32A32_SFLOAT,
                location: 9,
                offset: unsafe {
                    &(*(::std::ptr::null::<InstanceData>())).matrix[0][0] as *const _ as _
                },
            },
            VkVertexInputAttributeDescription {
                binding: INSTANCE_BUFFER_BIND_ID as _,
                format: VkFormat_VK_FORMAT_R32G32B32A32_SFLOAT,
                location: 10,
                offset: unsafe {
                    &(*(::std::ptr::null::<InstanceData>())).matrix[1][0] as *const _ as _
                },
            },
            VkVertexInputAttributeDescription {
                binding: INSTANCE_BUFFER_BIND_ID as _,
                format: VkFormat_VK_FORMAT_R32G32B32A32_SFLOAT,
                location: 11,
                offset: unsafe {
                    &(*(::std::ptr::null::<InstanceData>())).matrix[2][0] as *const _ as _
                },
            },
            VkVertexInputAttributeDescription {
                binding: INSTANCE_BUFFER_BIND_ID as _,
                format: VkFormat_VK_FORMAT_R32G32B32A32_SFLOAT,
                location: 12,
                offset: unsafe {
                    &(*(::std::ptr::null::<InstanceData>())).matrix[3][0] as *const _ as _
                },
            },
            VkVertexInputAttributeDescription {
                binding: INSTANCE_BUFFER_BIND_ID as _,
                format: VkFormat_VK_FORMAT_R32G32B32A32_SFLOAT,
                location: 13,
                offset: unsafe {
                    &(*(::std::ptr::null::<InstanceData>())).draw_area as *const _ as _
                },
            },
            VkVertexInputAttributeDescription {
                binding: INSTANCE_BUFFER_BIND_ID as _,
                format: VkFormat_VK_FORMAT_R32_SINT,
                location: 14,
                offset: unsafe {
                    &(*(::std::ptr::null::<InstanceData>())).material_index as *const _ as _
                },
            },
        ];
        attr
    }
}

impl From<PolygonModeType> for VkPolygonMode {
    fn from(val: PolygonModeType) -> Self {
        match val {
            PolygonModeType::Line => VkPolygonMode_VK_POLYGON_MODE_LINE,
            PolygonModeType::Point => VkPolygonMode_VK_POLYGON_MODE_POINT,
            _ => VkPolygonMode_VK_POLYGON_MODE_FILL,
        }
    }
}

impl From<CullingModeType> for VkCullModeFlags {
    fn from(val: CullingModeType) -> Self {
        match val {
            CullingModeType::Back => VkCullModeFlagBits_VK_CULL_MODE_BACK_BIT as VkCullModeFlags,
            CullingModeType::Front => VkCullModeFlagBits_VK_CULL_MODE_FRONT_BIT as VkCullModeFlags,
            CullingModeType::Both => {
                VkCullModeFlagBits_VK_CULL_MODE_FRONT_AND_BACK as VkCullModeFlags
            }
            _ => VkCullModeFlagBits_VK_CULL_MODE_NONE as VkCullModeFlags,
        }
    }
}

impl From<BlendFactor> for VkBlendFactor {
    fn from(val: BlendFactor) -> Self {
        match val {
            BlendFactor::Zero => VkBlendFactor_VK_BLEND_FACTOR_ZERO,
            BlendFactor::One => VkBlendFactor_VK_BLEND_FACTOR_ONE,
            BlendFactor::SrcColor => VkBlendFactor_VK_BLEND_FACTOR_ONE,
            BlendFactor::OneMinusSrcColor => VkBlendFactor_VK_BLEND_FACTOR_ONE_MINUS_SRC_COLOR,
            BlendFactor::DstColor => VkBlendFactor_VK_BLEND_FACTOR_DST_COLOR,
            BlendFactor::OneMinusDstColor => VkBlendFactor_VK_BLEND_FACTOR_ONE_MINUS_DST_COLOR,
            BlendFactor::SrcAlpha => VkBlendFactor_VK_BLEND_FACTOR_SRC_ALPHA,
            BlendFactor::OneMinusSrcAlpha => VkBlendFactor_VK_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA,
            BlendFactor::DstAlpha => VkBlendFactor_VK_BLEND_FACTOR_DST_ALPHA,
            BlendFactor::OneMinusDstAlpha => VkBlendFactor_VK_BLEND_FACTOR_ONE_MINUS_DST_ALPHA,
            BlendFactor::ConstantColor => VkBlendFactor_VK_BLEND_FACTOR_CONSTANT_COLOR,
            BlendFactor::OneMinusConstantColor => {
                VkBlendFactor_VK_BLEND_FACTOR_ONE_MINUS_CONSTANT_COLOR
            }
            BlendFactor::ConstantAlpha => VkBlendFactor_VK_BLEND_FACTOR_CONSTANT_ALPHA,
            BlendFactor::OneMinusConstantAlpha => {
                VkBlendFactor_VK_BLEND_FACTOR_ONE_MINUS_CONSTANT_ALPHA
            }
            BlendFactor::SrcAlphaSaturate => VkBlendFactor_VK_BLEND_FACTOR_SRC_ALPHA_SATURATE,
        }
    }
}

pub struct VkImageBarrierData {
    pub image: VkImage,
    pub old_layout: VkImageLayout,
    pub new_layout: VkImageLayout,
    pub src_access_mask: VkAccessFlags,
    pub dst_access_mask: VkAccessFlags,
    pub src_stage_mask: VkPipelineStageFlags,
    pub dst_stage_mask: VkPipelineStageFlags,
    pub layer_index: u32,
    pub layers_count: u32,
}
