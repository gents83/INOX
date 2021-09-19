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

    #[allow(deref_nullptr)]
    pub fn get_attributes_desc() -> Vec<VkVertexInputAttributeDescription> {
        let attr: Vec<VkVertexInputAttributeDescription> = vec![
            VkVertexInputAttributeDescription {
                binding: INSTANCE_BUFFER_BIND_ID as _,
                format: VkFormat_VK_FORMAT_R32G32B32_SFLOAT,
                location: 4,
                offset: unsafe { &(*(::std::ptr::null::<InstanceData>())).id as *const _ as _ },
            },
            VkVertexInputAttributeDescription {
                binding: INSTANCE_BUFFER_BIND_ID as _,
                format: VkFormat_VK_FORMAT_R32G32B32_SFLOAT,
                location: 5,
                offset: unsafe {
                    &(*(::std::ptr::null::<InstanceData>())).position as *const _ as _
                },
            },
            VkVertexInputAttributeDescription {
                binding: INSTANCE_BUFFER_BIND_ID as _,
                format: VkFormat_VK_FORMAT_R32G32B32_SFLOAT,
                location: 6,
                offset: unsafe {
                    &(*(::std::ptr::null::<InstanceData>())).rotation as *const _ as _
                },
            },
            VkVertexInputAttributeDescription {
                binding: INSTANCE_BUFFER_BIND_ID as _,
                format: VkFormat_VK_FORMAT_R32G32B32_SFLOAT,
                location: 7,
                offset: unsafe { &(*(::std::ptr::null::<InstanceData>())).scale as *const _ as _ },
            },
            VkVertexInputAttributeDescription {
                binding: INSTANCE_BUFFER_BIND_ID as _,
                format: VkFormat_VK_FORMAT_R32G32B32A32_SFLOAT,
                location: 8,
                offset: unsafe {
                    &(*(::std::ptr::null::<InstanceData>())).draw_area as *const _ as _
                },
            },
            VkVertexInputAttributeDescription {
                binding: INSTANCE_BUFFER_BIND_ID as _,
                format: VkFormat_VK_FORMAT_R32G32B32A32_SFLOAT,
                location: 9,
                offset: unsafe {
                    &(*(::std::ptr::null::<InstanceData>())).diffuse_color as *const _ as _
                },
            },
            VkVertexInputAttributeDescription {
                binding: INSTANCE_BUFFER_BIND_ID as _,
                format: VkFormat_VK_FORMAT_R32_SINT,
                location: 10,
                offset: unsafe {
                    &(*(::std::ptr::null::<InstanceData>())).diffuse_texture_index as *const _ as _
                },
            },
            VkVertexInputAttributeDescription {
                binding: INSTANCE_BUFFER_BIND_ID as _,
                format: VkFormat_VK_FORMAT_R32_SINT,
                location: 11,
                offset: unsafe {
                    &(*(::std::ptr::null::<InstanceData>())).diffuse_layer_index as *const _ as _
                },
            },
        ];
        attr
    }
}

impl Into<VkPolygonMode> for PolygonModeType {
    fn into(self) -> VkPolygonMode {
        match self {
            PolygonModeType::Line => VkPolygonMode_VK_POLYGON_MODE_LINE,
            PolygonModeType::Point => VkPolygonMode_VK_POLYGON_MODE_POINT,
            _ => VkPolygonMode_VK_POLYGON_MODE_FILL,
        }
    }
}

impl Into<VkCullModeFlags> for CullingModeType {
    fn into(self) -> VkCullModeFlags {
        match self {
            CullingModeType::Back => VkCullModeFlagBits_VK_CULL_MODE_BACK_BIT as VkCullModeFlags,
            CullingModeType::Front => VkCullModeFlagBits_VK_CULL_MODE_FRONT_BIT as VkCullModeFlags,
            CullingModeType::Both => {
                VkCullModeFlagBits_VK_CULL_MODE_FRONT_AND_BACK as VkCullModeFlags
            }
            _ => VkCullModeFlagBits_VK_CULL_MODE_NONE as VkCullModeFlags,
        }
    }
}

impl Into<VkBlendFactor> for BlendFactor {
    fn into(self) -> VkBlendFactor {
        match self {
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
