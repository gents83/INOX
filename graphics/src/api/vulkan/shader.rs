use vulkan_bindings::*;

pub enum ShaderType {
    Invalid,
    Vertex,
    Fragment,
}

pub struct Shader {
    shader_type: ShaderType,
    content: Vec<u32>,
    module: VkShaderModule,
    stage_info: ::std::option::Option<VkPipelineShaderStageCreateInfo>,
}

impl Default for Shader {
    fn default() -> Shader {
        Shader {
            shader_type: ShaderType::Invalid,
            content: Vec::new(),
            module: ::std::ptr::null_mut(),
            stage_info: None,
        }
    }
}

impl Shader {
    pub fn create<'a>(mut self, device: VkDevice, shader_type: ShaderType, shader_content: Vec<u32>, entry_point: &'a str) -> Self {
        self.shader_type = shader_type;
        self.content = shader_content;
        
        let shader_create_info = VkShaderModuleCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            codeSize: (self.content.len() * 4) as _,
            pCode: self.content.as_ptr() as *const _,
        };
    
        self.module = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            let result = vkCreateShaderModule.unwrap()(device, &shader_create_info, ::std::ptr::null_mut(), option.as_mut_ptr());
            if result != VkResult_VK_SUCCESS {
                eprintln!("Failed to create shader module")
            }
            option.assume_init()
        };    
                       
        self.stage_info = Some(VkPipelineShaderStageCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            stage: match self.shader_type {
                ShaderType::Vertex => VkShaderStageFlagBits_VK_SHADER_STAGE_VERTEX_BIT,
                ShaderType::Fragment => VkShaderStageFlagBits_VK_SHADER_STAGE_FRAGMENT_BIT,
                _ => VkShaderStageFlagBits_VK_SHADER_STAGE_VERTEX_BIT,
            },
            module: self.module,
            pName: entry_point.as_bytes().as_ptr() as *const _,
            pSpecializationInfo: ::std::ptr::null_mut(),
        });

        self
    }

    pub fn destroy(&self, device: VkDevice) {
        unsafe{
            vkDestroyShaderModule.unwrap()(device, self.module, ::std::ptr::null_mut());
        }
    }

    pub fn stage_info(&self) -> VkPipelineShaderStageCreateInfo {
        self.stage_info.unwrap()
    }
}