use super::device::*;
use super::render_pass::*;
use super::shader::*;
use crate::common::data_formats::*;
use crate::common::shader::*;
use crate::common::utils::*;
use std::{cell::RefCell, path::PathBuf, rc::Rc};
use vulkan_bindings::*;

pub struct PipelineImmutable {
    descriptor_set_layout: VkDescriptorSetLayout,
    shaders: Vec<Shader>,
    pipeline_layout: VkPipelineLayout,
    graphics_pipeline: VkPipeline,
}

#[derive(Clone)]
pub struct Pipeline {
    inner: Rc<RefCell<PipelineImmutable>>,
    device: Device,
}

impl Pipeline {
    pub fn create(device: &Device) -> Pipeline {
        let immutable = PipelineImmutable {
            descriptor_set_layout: ::std::ptr::null_mut(),
            shaders: Vec::new(),
            pipeline_layout: ::std::ptr::null_mut(),
            graphics_pipeline: ::std::ptr::null_mut(),
        };
        let inner = Rc::new(RefCell::new(immutable));
        Pipeline {
            inner,
            device: device.clone(),
        }
    }

    pub fn get_pipeline_layout(&self) -> VkPipelineLayout {
        self.inner.borrow().pipeline_layout
    }

    pub fn get_descriptor_set_layout(&self) -> VkDescriptorSetLayout {
        self.inner.borrow().descriptor_set_layout
    }

    pub fn delete(&self) {
        let inner = self.inner.borrow();
        inner.delete(&self.device);
    }

    pub fn set_shader(&mut self, shader_type: ShaderType, shader_filepath: PathBuf) -> &mut Self {
        if shader_filepath.exists() {
            let mut shader_file = std::fs::File::open(shader_filepath).unwrap();
            let shader_code = read_spirv_from_bytes(&mut shader_file);

            self.inner.borrow_mut().create_shader_module(
                &self.device,
                shader_type,
                shader_code,
                "main",
            );
        }
        self
    }

    pub fn prepare(&mut self, render_pass: &RenderPass) -> &mut Self {
        self.inner.borrow_mut().prepare(&self.device, render_pass);
        self
    }

    pub fn build(&mut self) -> &mut Self {
        self.inner
            .borrow_mut()
            .create_descriptor_set_layout(&self.device);
        self
    }
}

impl PipelineImmutable {
    fn delete(&self, device: &Device) {
        self.destroy_shader_modules(&device);
        unsafe {
            vkDestroyDescriptorSetLayout.unwrap()(
                device.get_device(),
                self.descriptor_set_layout,
                ::std::ptr::null_mut(),
            );

            vkDestroyPipeline.unwrap()(
                device.get_device(),
                self.graphics_pipeline,
                ::std::ptr::null_mut(),
            );
            vkDestroyPipelineLayout.unwrap()(
                device.get_device(),
                self.pipeline_layout,
                ::std::ptr::null_mut(),
            );
        }
    }

    fn prepare(&mut self, device: &Device, render_pass: &RenderPass) {
        let details = device.get_instance().get_swap_chain_info();

        let mut shader_stages: Vec<VkPipelineShaderStageCreateInfo> = Vec::new();
        for shader in self.shaders.iter() {
            shader_stages.push(shader.stage_info());
        }

        let binding_info = VertexData::get_binding_desc();
        let attr_info = VertexData::get_attributes_desc();

        let vertex_input_info = VkPipelineVertexInputStateCreateInfo {
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
            offset: VkOffset2D { x: 0, y: 0 },
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
            cullMode: VkCullModeFlagBits_VK_CULL_MODE_BACK_BIT as VkCullModeFlags,
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

        let depth_stencil = VkPipelineDepthStencilStateCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            depthTestEnable: VK_TRUE,
            depthWriteEnable: VK_TRUE,
            depthCompareOp: VkCompareOp_VK_COMPARE_OP_LESS,
            depthBoundsTestEnable: VK_FALSE,
            stencilTestEnable: VK_FALSE,
            front: VkStencilOpState {
                failOp: VkStencilOp_VK_STENCIL_OP_KEEP,
                passOp: VkStencilOp_VK_STENCIL_OP_KEEP,
                depthFailOp: VkStencilOp_VK_STENCIL_OP_KEEP,
                compareOp: VkCompareOp_VK_COMPARE_OP_NEVER,
                compareMask: 0,
                writeMask: 0,
                reference: 0,
            },
            back: VkStencilOpState {
                failOp: VkStencilOp_VK_STENCIL_OP_KEEP,
                passOp: VkStencilOp_VK_STENCIL_OP_KEEP,
                depthFailOp: VkStencilOp_VK_STENCIL_OP_KEEP,
                compareOp: VkCompareOp_VK_COMPARE_OP_NEVER,
                compareMask: 0,
                writeMask: 0,
                reference: 0,
            },
            minDepthBounds: 0.0,
            maxDepthBounds: 1.0,
        };

        let color_blend_attachment = VkPipelineColorBlendAttachmentState {
            blendEnable: VK_TRUE,
            srcColorBlendFactor: VkBlendFactor_VK_BLEND_FACTOR_SRC_ALPHA,
            dstColorBlendFactor: VkBlendFactor_VK_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA,
            colorBlendOp: VkBlendOp_VK_BLEND_OP_ADD,
            srcAlphaBlendFactor: VkBlendFactor_VK_BLEND_FACTOR_ONE,
            dstAlphaBlendFactor: VkBlendFactor_VK_BLEND_FACTOR_ZERO,
            alphaBlendOp: VkBlendOp_VK_BLEND_OP_ADD,
            colorWriteMask: (VkColorComponentFlagBits_VK_COLOR_COMPONENT_R_BIT
                | VkColorComponentFlagBits_VK_COLOR_COMPONENT_G_BIT
                | VkColorComponentFlagBits_VK_COLOR_COMPONENT_B_BIT
                | VkColorComponentFlagBits_VK_COLOR_COMPONENT_A_BIT)
                as VkColorComponentFlags,
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
                vkCreatePipelineLayout.unwrap()(
                    device.get_device(),
                    &pipeline_layout_info,
                    ::std::ptr::null_mut(),
                    option.as_mut_ptr()
                )
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
            pDepthStencilState: &depth_stencil,
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
                vkCreateGraphicsPipelines.unwrap()(
                    device.get_device(),
                    ::std::ptr::null_mut(),
                    1,
                    &pipeline_info,
                    ::std::ptr::null_mut(),
                    option.as_mut_ptr()
                )
            );
            option.assume_init()
        };

        unsafe {
            vkCmdBindPipeline.unwrap()(
                device.get_current_command_buffer(),
                VkPipelineBindPoint_VK_PIPELINE_BIND_POINT_GRAPHICS,
                self.graphics_pipeline,
            );
        }
    }

    fn create_shader_module(
        &mut self,
        device: &Device,
        shader_type: ShaderType,
        shader_content: Vec<u32>,
        entry_point: &'static str,
    ) -> &mut Self {
        let shader = Shader::create(
            device.get_device(),
            shader_type,
            shader_content,
            entry_point,
        );
        self.shaders.push(shader);
        self
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
                vkCreateDescriptorSetLayout.unwrap()(
                    device.get_device(),
                    &layout_create_info,
                    ::std::ptr::null_mut(),
                    option.as_mut_ptr()
                )
            );
            option.assume_init()
        };
    }
}
