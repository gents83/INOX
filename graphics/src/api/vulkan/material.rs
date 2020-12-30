use nrg_common::*;
use nrg_math::*;
use vulkan_bindings::*;
use crate::data_formats::*;
use crate::material::*;
use crate::utils::*;
use super::device::*;
use super::shader::*;
use super::texture::*;
use super::render_pass::*;



pub struct Material {
    descriptor_set_layout: VkDescriptorSetLayout,
    descriptor_pool: VkDescriptorPool,
    uniform_buffers: Vec<VkBuffer>,
    uniform_buffers_memory: Vec<VkDeviceMemory>,
    shaders: Vec<Shader>,
    pipeline_layout: VkPipelineLayout,
    graphics_pipeline: VkPipeline,
}

#[derive(PartialEq)]
pub struct MaterialInstance {
    textures: Vec<Texture>,
    descriptor_sets: Vec<VkDescriptorSet>,
}


impl Default for Material {
    fn default() -> Material {
        Self {
            descriptor_set_layout: ::std::ptr::null_mut(),
            descriptor_pool: ::std::ptr::null_mut(),
            uniform_buffers: Vec::new(),
            uniform_buffers_memory: Vec::new(),
            shaders: Vec::new(),
            pipeline_layout: ::std::ptr::null_mut(),
            graphics_pipeline: ::std::ptr::null_mut(),
        }
    }
}

impl Material {
    pub fn get_descriptor_set_layout(&self) -> &VkDescriptorSetLayout {
        &self.descriptor_set_layout
    }

    pub fn get_descriptor_pool(&self) -> &VkDescriptorPool {
        &self.descriptor_pool
    }

    pub fn get_pipeline_layout(&self) -> &VkPipelineLayout {
        &self.pipeline_layout
    }

    pub fn get_uniform_buffer(&self, index: usize) -> &VkBuffer {
        &self.uniform_buffers[index]
    }

    pub fn delete(&self, device:&Device) {
        self.destroy_shader_modules(device);
        unsafe {
            vkDestroyDescriptorSetLayout.unwrap()(device.get_device(), self.descriptor_set_layout, ::std::ptr::null_mut());
        
            vkDestroyPipeline.unwrap()(device.get_device(), self.graphics_pipeline, ::std::ptr::null_mut());
            vkDestroyPipelineLayout.unwrap()(device.get_device(), self.pipeline_layout, ::std::ptr::null_mut());
        }
    }


    pub fn set_shader(&mut self, device:&Device, shader_type: ShaderType, shader_filepath: &str) {     
        let mut shader_file = std::fs::File::open(shader_filepath).unwrap();
        let shader_code = read_spirv_from_bytes(&mut shader_file);
        
        self.create_shader_module(device, shader_type, shader_code, "main");
    }

    pub fn build_material(&mut self, device: &Device) {
        self.create_descriptor_set_layout(device);
        self.create_uniform_buffers(device);
        self.create_descriptor_pool(device);
    }
    

    pub fn create_graphics_pipeline(&mut self, device: &Device, render_pass: &RenderPass) {  
        let details = device.get_instance().get_swap_chain_info();

        let mut shader_stages: Vec<VkPipelineShaderStageCreateInfo> = Vec::new();
        for shader in self.shaders.iter() {
            shader_stages.push(shader.stage_info());
        }

        let binding_info = VertexData::get_binding_desc();
        let attr_info = VertexData::get_attributes_desc();

        let vertex_input_info = VkPipelineVertexInputStateCreateInfo{
            sType: VkStructureType_VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            vertexBindingDescriptionCount: 1,
            pVertexBindingDescriptions: &binding_info,
            vertexAttributeDescriptionCount: attr_info.len() as _,
            pVertexAttributeDescriptions: attr_info.as_ptr(),
        };
        

        let input_assembly = VkPipelineInputAssemblyStateCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            topology: VkPrimitiveTopology_VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST,
            primitiveRestartEnable: VK_FALSE,
        };

        let viewport = VkViewport {
            x: 0.0,
            y: 0.0,
            width: details.capabilities.currentExtent.width as _,
            height: details.capabilities.currentExtent.height as _,
            minDepth: 0.0,
            maxDepth: 1.0,
        };

        let scissors = VkRect2D {
            offset: VkOffset2D {x: 0, y: 0},
            extent: details.capabilities.currentExtent,
        };

        let viewport_state = VkPipelineViewportStateCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            viewportCount: 1,
            pViewports: &viewport,
            scissorCount: 1,
            pScissors: &scissors,
        };

        let rasterizer = VkPipelineRasterizationStateCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            depthClampEnable: VK_FALSE,
            rasterizerDiscardEnable: VK_FALSE,
            polygonMode: VkPolygonMode_VK_POLYGON_MODE_FILL,
            cullMode: VkCullModeFlagBits_VK_CULL_MODE_NONE as VkCullModeFlags,
            frontFace: VkFrontFace_VK_FRONT_FACE_CLOCKWISE,
            depthBiasEnable: VK_FALSE,
            depthBiasConstantFactor: 0.0,
            depthBiasClamp: 0.0,
            depthBiasSlopeFactor: 0.0,
            lineWidth: 1.0,
        };

        let multisampling = VkPipelineMultisampleStateCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            rasterizationSamples: VkSampleCountFlagBits_VK_SAMPLE_COUNT_1_BIT,
            sampleShadingEnable: VK_FALSE,
            minSampleShading: 1.0,
            pSampleMask: ::std::ptr::null_mut(),
            alphaToCoverageEnable: VK_FALSE,
            alphaToOneEnable: VK_FALSE,
        };

        let color_blend_attachment = VkPipelineColorBlendAttachmentState {
            blendEnable: VK_TRUE,
            srcColorBlendFactor: VkBlendFactor_VK_BLEND_FACTOR_SRC_ALPHA,
            dstColorBlendFactor: VkBlendFactor_VK_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA,
            colorBlendOp: VkBlendOp_VK_BLEND_OP_ADD,
            srcAlphaBlendFactor: VkBlendFactor_VK_BLEND_FACTOR_ONE,
            dstAlphaBlendFactor: VkBlendFactor_VK_BLEND_FACTOR_ZERO,
            alphaBlendOp: VkBlendOp_VK_BLEND_OP_ADD,
            colorWriteMask: (VkColorComponentFlagBits_VK_COLOR_COMPONENT_R_BIT | 
                            VkColorComponentFlagBits_VK_COLOR_COMPONENT_G_BIT |
                            VkColorComponentFlagBits_VK_COLOR_COMPONENT_B_BIT |
                            VkColorComponentFlagBits_VK_COLOR_COMPONENT_A_BIT) as VkColorComponentFlags,
        };

        let color_blending = VkPipelineColorBlendStateCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            logicOpEnable: VK_TRUE,
            logicOp: VkLogicOp_VK_LOGIC_OP_COPY,
            attachmentCount: 1,
            pAttachments: &color_blend_attachment,
            blendConstants: [0.0, 0.0, 0.0, 0.0],
        };

        let pipeline_layout_info = VkPipelineLayoutCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            setLayoutCount: 1,
            pSetLayouts: &self.descriptor_set_layout,
            pushConstantRangeCount: 0,
            pPushConstantRanges: ::std::ptr::null_mut(),
        };

        self.pipeline_layout = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkCreatePipelineLayout.unwrap()(device.get_device(), &pipeline_layout_info, ::std::ptr::null_mut(), option.as_mut_ptr())
            );
            option.assume_init()
        };

        let pipeline_info = VkGraphicsPipelineCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            stageCount: shader_stages.len() as _,
            pStages: shader_stages.as_ptr(),
            pVertexInputState: &vertex_input_info,
            pInputAssemblyState: &input_assembly,
            pViewportState: &viewport_state,
            pRasterizationState: &rasterizer,
            pTessellationState: ::std::ptr::null_mut(),
            pMultisampleState: &multisampling,
            pDepthStencilState: ::std::ptr::null_mut(),
            pColorBlendState: &color_blending,
            pDynamicState: ::std::ptr::null_mut(),
            layout: self.pipeline_layout,
            renderPass: render_pass.into(),
            subpass: 0,
            basePipelineHandle: ::std::ptr::null_mut(),
            basePipelineIndex: -1,
        };
        
        self.graphics_pipeline = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkCreateGraphicsPipelines.unwrap()(device.get_device(), ::std::ptr::null_mut(), 1, &pipeline_info, ::std::ptr::null_mut(), option.as_mut_ptr())
            );
            option.assume_init()
        };

        unsafe {
            vkCmdBindPipeline.unwrap()(device.get_current_command_buffer(), VkPipelineBindPoint_VK_PIPELINE_BIND_POINT_GRAPHICS, self.graphics_pipeline);
        }
    }


    pub fn update_uniform_buffer(&mut self, device:&Device, image_index: usize, model_transform: &Matrix4f, cam_pos: Vector3f) {
        let details = device.get_instance().get_swap_chain_info();
        let uniform_data: [UniformData; 1] = [
            UniformData {
                model: *model_transform,
                view: Matrix4::from_look_at(cam_pos.into(), 
                                            [0.0, 0.0, 0.0].into(),
                                            [0.0, 0.0, 1.0].into()),
                proj: Matrix4::create_perspective(Degree(45.0).into(), 
                                                    details.capabilities.currentExtent.width as f32 / details.capabilities.currentExtent.height as f32, 
                                                    0.1, 1000.0),
            }
        ];

        let mut buffer_memory = self.uniform_buffers_memory[image_index];
        device.map_buffer_memory(&mut buffer_memory, &uniform_data);
        self.uniform_buffers_memory[image_index] = buffer_memory;
    }

}

impl Material {    
    fn create_shader_module(&mut self, device:&Device, shader_type: ShaderType, shader_content: Vec<u32>, entry_point: &'static str) {
        let shader = Shader::create(device.get_device(), shader_type, shader_content, entry_point);
        self.shaders.push( shader );
    }

    fn destroy_shader_modules(&self, device: &Device) {
        for shader in self.shaders.iter() {
            shader.destroy(device.get_device());
        }
    }

    fn create_descriptor_set_layout(&mut self, device: &Device) {
        let uniform_buffer_layout_binding = VkDescriptorSetLayoutBinding {
            binding: 0,
            descriptorCount: 1,
            descriptorType: VkDescriptorType_VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
            pImmutableSamplers: ::std::ptr::null_mut(),
            stageFlags: VkShaderStageFlagBits_VK_SHADER_STAGE_VERTEX_BIT as _,
        };
        let sampler_layout_binding = VkDescriptorSetLayoutBinding {
            binding: 1,
            descriptorCount: 1,
            descriptorType: VkDescriptorType_VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
            pImmutableSamplers: ::std::ptr::null_mut(),
            stageFlags: VkShaderStageFlagBits_VK_SHADER_STAGE_FRAGMENT_BIT as _,
        };
        let bindings = [uniform_buffer_layout_binding, sampler_layout_binding];
        let layout_create_info = VkDescriptorSetLayoutCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            flags: 0,
            pNext: ::std::ptr::null_mut(),
            bindingCount: bindings.len() as _,
            pBindings: bindings.as_ptr(),
        };

        self.descriptor_set_layout = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkCreateDescriptorSetLayout.unwrap()(device.get_device(), &layout_create_info, ::std::ptr::null_mut(), option.as_mut_ptr())
            );
            option.assume_init()
        };  
    }

    fn create_descriptor_pool(&mut self, device: &Device) {
        let pool_sizes = [
            VkDescriptorPoolSize {
                type_: VkDescriptorType_VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
                descriptorCount: device.get_images_count() as _,
            },
            VkDescriptorPoolSize {
                type_: VkDescriptorType_VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
                descriptorCount: device.get_images_count() as _,
            },
        ];

        let pool_info = VkDescriptorPoolCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO,
            flags: 0,
            pNext: ::std::ptr::null_mut(),
            poolSizeCount: pool_sizes.len() as _,
            pPoolSizes: pool_sizes.as_ptr(),
            maxSets: device.get_images_count() as _,
        };

        self.descriptor_pool = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkCreateDescriptorPool.unwrap()(device.get_device(), &pool_info, ::std::ptr::null_mut(), option.as_mut_ptr())
            );
            option.assume_init()
        };  
    }

    fn create_uniform_buffers(&mut self, device: &Device) {  
        let mut uniform_buffers = Vec::<VkBuffer>::with_capacity(device.get_images_count());
        let mut uniform_buffers_memory = Vec::<VkDeviceMemory>::with_capacity(device.get_images_count());
        unsafe {
            uniform_buffers.set_len(device.get_images_count());
            uniform_buffers_memory.set_len(device.get_images_count());
        }

        let uniform_buffer_size = std::mem::size_of::<UniformData>();
        let flags = VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_HOST_COHERENT_BIT;
        for i in 0..uniform_buffers.len() {
            device.create_buffer(uniform_buffer_size as _, VkBufferUsageFlagBits_VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT as _, flags as _, &mut uniform_buffers[i], &mut uniform_buffers_memory[i]);
        }

        self.uniform_buffers = uniform_buffers;
        self.uniform_buffers_memory = uniform_buffers_memory;
    }
}

impl EventListener for MaterialInstance {
    fn on_event_received(&self, event_type: u32) {
        if event_type == DeviceEvent::OnSwapChainCreated as u32 {

        }
        else if event_type == DeviceEvent::OnSwapChainCleanUp as u32 {

        }
    }
}

impl PartialEq<dyn EventListener> for MaterialInstance {
    fn eq(&self, other: &dyn EventListener) -> bool {
        true
    }
}


impl MaterialInstance {
    pub fn create_from(device: &mut Device, material: &Material) -> Self {
        let mut instance = MaterialInstance {
            textures: Vec::new(),
            descriptor_sets: Vec::new(),
        };
        instance.create_descriptor_sets(&device, &material);
        //device.register_listener(|ev|{ /*instance.on_device_callback(ev);*/ } );
        instance       
    }

    pub fn on_device_callback(&mut self, event_type: u32) {

    }

    pub fn destroy(&self, device: &Device) {
        for texture in self.textures.iter() {
            texture.destroy(device);
        }
    }  

    pub fn add_texture(&mut self, device: &Device, filepath: &str) -> &mut Self {
        self.textures.push( Texture::create_from(device, filepath) );
        self
    }

    pub fn create_descriptor_sets(&mut self, device: &Device, material: &Material) {
        let mut layouts = Vec::<VkDescriptorSetLayout>::with_capacity(device.get_images_count());
        unsafe {
            layouts.set_len(device.get_images_count());
        }
        for layout in layouts.iter_mut() {
            *layout = *material.get_descriptor_set_layout();
        }

        let alloc_info = VkDescriptorSetAllocateInfo{
            sType: VkStructureType_VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO,
            pNext: ::std::ptr::null_mut(),
            descriptorPool: *material.get_descriptor_pool(),
            descriptorSetCount: device.get_images_count() as _,
            pSetLayouts: layouts.as_mut_ptr(),
        };

        let mut descriptor_sets = Vec::<VkDescriptorSet>::with_capacity(device.get_images_count());
        unsafe {
            descriptor_sets.set_len(device.get_images_count());
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkAllocateDescriptorSets.unwrap()(device.get_device(), &alloc_info, descriptor_sets.as_mut_ptr())
            );
        }
        
        self.descriptor_sets = descriptor_sets;
    }

    pub fn update_descriptor_sets(&mut self, device: &Device, material: &Material, image_index: usize) {
        let buffer_info = VkDescriptorBufferInfo {
            buffer: *material.get_uniform_buffer(image_index),
            offset: 0,
            range: ::std::mem::size_of::<UniformData>() as _,
        };

        let descriptor_write = [
            VkWriteDescriptorSet {
                sType: VkStructureType_VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
                pNext: ::std::ptr::null_mut(),
                dstSet: self.descriptor_sets[image_index],
                dstBinding: 0,
                dstArrayElement: 0,
                descriptorCount: 1,
                descriptorType: VkDescriptorType_VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
                pImageInfo: ::std::ptr::null_mut(),
                pBufferInfo: &buffer_info,
                pTexelBufferView: ::std::ptr::null_mut(),
            },
            VkWriteDescriptorSet {
                sType: VkStructureType_VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
                pNext: ::std::ptr::null_mut(),
                dstSet: self.descriptor_sets[image_index],
                dstBinding: 1,
                dstArrayElement: 0,
                descriptorCount: 1,
                descriptorType: VkDescriptorType_VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
                pImageInfo: &self.textures[0].get_descriptor(),
                pBufferInfo: ::std::ptr::null_mut(),
                pTexelBufferView: ::std::ptr::null_mut(),
            },
        ];

        unsafe {
            vkUpdateDescriptorSets.unwrap()(device.get_device(), descriptor_write.len() as _, descriptor_write.as_ptr(), 0, ::std::ptr::null_mut());

            vkCmdBindDescriptorSets.unwrap()(device.get_current_command_buffer(), VkPipelineBindPoint_VK_PIPELINE_BIND_POINT_GRAPHICS, *material.get_pipeline_layout(), 0, 1, &self.descriptor_sets[image_index], 0, ::std::ptr::null_mut());
        }
    }
}

