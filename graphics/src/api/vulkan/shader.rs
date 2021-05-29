use crate::common::shader::*;
use vulkan_bindings::*;

pub struct Shader {
    shader_type: ShaderType,
    content: Vec<u32>,
    module: VkShaderModule,
    stage_info: ::std::option::Option<VkPipelineShaderStageCreateInfo>,
}

impl Shader {
    pub fn create(
        device: VkDevice,
        shader_type: ShaderType,
        shader_content: Vec<u32>,
        entry_point: &str,
    ) -> Self {
        let mut shader = Shader {
            shader_type: ShaderType::Invalid,
            content: Vec::new(),
            module: ::std::ptr::null_mut(),
            stage_info: None,
        };
        shader.shader_type = shader_type;
        shader.content = shader_content;

        let shader_create_info = VkShaderModuleCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            codeSize: (shader.content.len() * 4) as _,
            pCode: shader.content.as_ptr() as *const _,
        };

        shader.module = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            let result = vkCreateShaderModule.unwrap()(
                device,
                &shader_create_info,
                ::std::ptr::null_mut(),
                option.as_mut_ptr(),
            );
            if result != VkResult_VK_SUCCESS {
                eprintln!("Failed to create shader module")
            }
            option.assume_init()
        };

        shader.stage_info = Some(VkPipelineShaderStageCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            stage: match shader.shader_type {
                ShaderType::Vertex => VkShaderStageFlagBits_VK_SHADER_STAGE_VERTEX_BIT,
                ShaderType::Fragment => VkShaderStageFlagBits_VK_SHADER_STAGE_FRAGMENT_BIT,
                ShaderType::TessellationControl => {
                    VkShaderStageFlagBits_VK_SHADER_STAGE_TESSELLATION_CONTROL_BIT
                }
                ShaderType::TessellationEvaluation => {
                    VkShaderStageFlagBits_VK_SHADER_STAGE_TESSELLATION_EVALUATION_BIT
                }
                ShaderType::Geometry => {
                    VkShaderStageFlagBits_VK_SHADER_STAGE_GEOMETRY_BIT
                }
                _ => VkShaderStageFlagBits_VK_SHADER_STAGE_VERTEX_BIT,
            },
            module: shader.module,
            pName: entry_point.as_bytes().as_ptr() as *const _,
            pSpecializationInfo: ::std::ptr::null_mut(),
        });
        shader
    }

    pub fn destroy(&self, device: VkDevice) {
        unsafe {
            vkDestroyShaderModule.unwrap()(device, self.module, ::std::ptr::null_mut());
        }
    }

    pub fn stage_info(&self) -> VkPipelineShaderStageCreateInfo {
        self.stage_info.unwrap()
    }
}
