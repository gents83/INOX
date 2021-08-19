use super::render_pass::*;
use super::shader::*;
use super::{data_formats::INSTANCE_BUFFER_BIND_ID, device::*};
use crate::common::data_formats::*;
use crate::common::shader::*;
use crate::common::texture::MAX_TEXTURE_COUNT;
use crate::common::texture::*;
use crate::common::utils::*;

use nrg_filesystem::convert_from_local_path;
use nrg_math::matrix4_to_array;
use nrg_math::Matrix4;
use nrg_resources::DATA_FOLDER;
use std::path::PathBuf;
use std::{cell::RefCell, path::Path, rc::Rc};
use vulkan_bindings::*;

pub struct PipelineImmutable {
    constant_data: ConstantData,
    descriptor_set_layout: VkDescriptorSetLayout,
    descriptor_pool: VkDescriptorPool,
    descriptor_sets: Vec<VkDescriptorSet>,
    uniform_buffers_size: usize,
    uniform_buffers: Vec<VkBuffer>,
    uniform_buffers_memory: Vec<VkDeviceMemory>,
    shaders: Vec<Shader>,
    pipeline_layout: VkPipelineLayout,
    graphics_pipeline: VkPipeline,
    instance_buffer_count: usize,
    instance_buffer: VkBuffer,
    instance_buffer_memory: VkDeviceMemory,
    indirect_command_buffer_count: usize,
    indirect_command_buffer: VkBuffer,
    indirect_command_buffer_memory: VkDeviceMemory,
    indirect_commands: Vec<VkDrawIndexedIndirectCommand>,
}

#[derive(Clone)]
pub struct Pipeline {
    inner: Rc<RefCell<PipelineImmutable>>,
    device: Device,
}

impl Pipeline {
    pub fn create(device: &Device) -> Pipeline {
        let immutable = PipelineImmutable {
            constant_data: ConstantData::default(),
            descriptor_set_layout: ::std::ptr::null_mut(),
            descriptor_sets: Vec::new(),
            descriptor_pool: ::std::ptr::null_mut(),
            uniform_buffers_size: 0,
            uniform_buffers: Vec::new(),
            uniform_buffers_memory: Vec::new(),
            shaders: Vec::new(),
            pipeline_layout: ::std::ptr::null_mut(),
            graphics_pipeline: ::std::ptr::null_mut(),
            instance_buffer_count: 0,
            instance_buffer: ::std::ptr::null_mut(),
            instance_buffer_memory: ::std::ptr::null_mut(),
            indirect_command_buffer_count: 0,
            indirect_command_buffer: ::std::ptr::null_mut(),
            indirect_command_buffer_memory: ::std::ptr::null_mut(),
            indirect_commands: Vec::new(),
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

    pub fn set_shader(&mut self, shader_type: ShaderType, shader_filepath: &Path) -> &mut Self {
        let path = convert_from_local_path(PathBuf::from(DATA_FOLDER).as_path(), shader_filepath);
        if path.exists() && path.is_file() {
            self.inner
                .borrow_mut()
                .remove_shader(&self.device, shader_type);

            let mut shader_file = std::fs::File::open(path).unwrap();
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

    pub fn bind(&mut self, commands: &[InstanceCommand], instances: &[InstanceData]) -> &mut Self {
        self.inner
            .borrow_mut()
            .bind(&self.device, commands, instances);
        self
    }

    pub fn update_constant_data(
        &self,
        width: f32,
        height: f32,
        view: &Matrix4,
        proj: &Matrix4,
    ) -> &Self {
        self.inner
            .borrow_mut()
            .update_constant_data(&self.device, width, height, view, proj);
        self
    }

    pub fn update_uniform_buffer(&self, view: &Matrix4, proj: &Matrix4) -> &Self {
        self.inner
            .borrow_mut()
            .update_uniform_buffer(&self.device, view, proj);
        self
    }
    pub fn update_descriptor_sets(&self, textures: &[TextureAtlas]) -> &Self {
        self.inner
            .borrow_mut()
            .update_descriptor_sets(&self.device, textures);
        self
    }

    pub fn bind_descriptors(&self) -> &Self {
        self.inner.borrow_mut().bind_descriptors(&self.device);
        self
    }

    pub fn bind_indirect(&self) -> &Self {
        self.inner.borrow_mut().bind_indirect(&self.device);
        self
    }

    pub fn draw_indirect(&mut self, count: usize) -> &mut Self {
        self.inner.borrow_mut().draw_indirect(&self.device, count);
        self
    }

    pub fn build(&mut self, device: &Device, render_pass: &RenderPass) -> &mut Self {
        self.inner
            .borrow_mut()
            .create_descriptor_set_layout(&self.device)
            .create_uniform_buffers(device)
            .create_descriptor_pool(device)
            .create_descriptor_sets(device)
            .create(device, render_pass);
        self
    }
}

impl PipelineImmutable {
    fn create_uniform_buffers(&mut self, device: &Device) -> &mut Self {
        let mut uniform_buffers = Vec::<VkBuffer>::with_capacity(device.get_images_count());
        let mut uniform_buffers_memory =
            Vec::<VkDeviceMemory>::with_capacity(device.get_images_count());
        unsafe {
            uniform_buffers.set_len(device.get_images_count());
            uniform_buffers_memory.set_len(device.get_images_count());
        }

        let uniform_buffers_size = std::mem::size_of::<UniformData>();
        let flags = VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT
            | VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_HOST_COHERENT_BIT;
        for i in 0..uniform_buffers.len() {
            device.create_buffer(
                uniform_buffers_size as _,
                VkBufferUsageFlagBits_VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT as _,
                flags as _,
                &mut uniform_buffers[i],
                &mut uniform_buffers_memory[i],
            );
        }

        self.uniform_buffers_size = uniform_buffers_size;
        self.uniform_buffers = uniform_buffers;
        self.uniform_buffers_memory = uniform_buffers_memory;
        self
    }
    fn create_descriptor_pool(&mut self, device: &Device) -> &mut Self {
        let pool_sizes: Vec<VkDescriptorPoolSize> = vec![
            VkDescriptorPoolSize {
                type_: VkDescriptorType_VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
                descriptorCount: device.get_images_count() as u32,
            },
            VkDescriptorPoolSize {
                type_: VkDescriptorType_VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
                descriptorCount: MAX_TEXTURE_COUNT as u32 * device.get_images_count() as u32,
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
                vkCreateDescriptorPool.unwrap()(
                    device.get_device(),
                    &pool_info,
                    ::std::ptr::null_mut(),
                    option.as_mut_ptr()
                )
            );
            option.assume_init()
        };
        self
    }
    pub fn create_descriptor_sets(&mut self, device: &Device) -> &mut Self {
        let mut layouts = Vec::<VkDescriptorSetLayout>::with_capacity(device.get_images_count());
        unsafe {
            layouts.set_len(device.get_images_count());
        }
        for layout in layouts.iter_mut() {
            *layout = self.descriptor_set_layout;
        }

        let alloc_info = VkDescriptorSetAllocateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO,
            pNext: ::std::ptr::null_mut(),
            descriptorPool: self.descriptor_pool,
            descriptorSetCount: device.get_images_count() as _,
            pSetLayouts: layouts.as_mut_ptr(),
        };

        let mut descriptor_sets = Vec::<VkDescriptorSet>::with_capacity(device.get_images_count());
        unsafe {
            descriptor_sets.set_len(device.get_images_count());
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkAllocateDescriptorSets.unwrap()(
                    device.get_device(),
                    &alloc_info,
                    descriptor_sets.as_mut_ptr()
                )
            );
        }

        self.descriptor_sets = descriptor_sets;
        self
    }
    fn delete(&self, device: &Device) {
        self.destroy_shader_modules(device);
        device.destroy_buffer(
            &self.indirect_command_buffer,
            &self.indirect_command_buffer_memory,
        );
        device.destroy_buffer(&self.instance_buffer, &self.instance_buffer_memory);
        for i in 0..self.uniform_buffers.len() {
            device.destroy_buffer(&self.uniform_buffers[i], &self.uniform_buffers_memory[i]);
        }
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

    fn create(&mut self, device: &Device, render_pass: &RenderPass) -> &mut Self {
        let mut shader_stages: Vec<VkPipelineShaderStageCreateInfo> = Vec::new();
        for shader in self.shaders.iter() {
            shader_stages.push(shader.stage_info());
        }

        let vertex_data_binding_info = VertexData::get_binding_desc();
        let vertex_data_attr_info = VertexData::get_attributes_desc();

        let instance_data_binding_info = InstanceData::get_binding_desc();
        let instance_data_attr_info = InstanceData::get_attributes_desc();

        let binding_info = [vertex_data_binding_info, instance_data_binding_info];
        let attr_info = [vertex_data_attr_info, instance_data_attr_info].concat();

        let vertex_input_info = VkPipelineVertexInputStateCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            vertexBindingDescriptionCount: binding_info.len() as _,
            pVertexBindingDescriptions: binding_info.as_ptr(),
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
            width: render_pass.get_framebuffer_width() as _,
            height: render_pass.get_framebuffer_height() as _,
            minDepth: 0.0,
            maxDepth: 1.0,
        };

        let scissors = VkRect2D {
            offset: VkOffset2D { x: 0, y: 0 },
            extent: render_pass.get_extent(),
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
            depthClampEnable: VK_TRUE,
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
            minSampleShading: 0.0,
            pSampleMask: ::std::ptr::null_mut(),
            alphaToCoverageEnable: VK_FALSE,
            alphaToOneEnable: VK_FALSE,
        };

        let stencil_state = VkStencilOpState {
            failOp: VkStencilOp_VK_STENCIL_OP_KEEP,
            passOp: VkStencilOp_VK_STENCIL_OP_KEEP,
            depthFailOp: VkStencilOp_VK_STENCIL_OP_KEEP,
            compareOp: VkCompareOp_VK_COMPARE_OP_ALWAYS,
            compareMask: 0,
            writeMask: 0,
            reference: 0,
        };

        let depth_stencil = VkPipelineDepthStencilStateCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            depthTestEnable: VK_TRUE,
            depthWriteEnable: VK_TRUE,
            depthCompareOp: VkCompareOp_VK_COMPARE_OP_LESS_OR_EQUAL,
            depthBoundsTestEnable: VK_FALSE,
            stencilTestEnable: VK_FALSE,
            front: stencil_state,
            back: stencil_state,
            minDepthBounds: 0.0,
            maxDepthBounds: 1.0,
        };

        let color_blend_attachment = VkPipelineColorBlendAttachmentState {
            blendEnable: VK_TRUE,
            srcColorBlendFactor: VkBlendFactor_VK_BLEND_FACTOR_ONE,
            dstColorBlendFactor: VkBlendFactor_VK_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA,
            colorBlendOp: VkBlendOp_VK_BLEND_OP_ADD,
            srcAlphaBlendFactor: VkBlendFactor_VK_BLEND_FACTOR_ONE_MINUS_DST_ALPHA,
            dstAlphaBlendFactor: VkBlendFactor_VK_BLEND_FACTOR_ONE,
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
            logicOpEnable: VK_FALSE,
            logicOp: VkLogicOp_VK_LOGIC_OP_COPY,
            attachmentCount: 1,
            pAttachments: &color_blend_attachment,
            blendConstants: [0., 0., 0., 0.],
        };

        let push_constant_range = VkPushConstantRange {
            stageFlags: VkShaderStageFlagBits_VK_SHADER_STAGE_ALL_GRAPHICS as _,
            offset: 0,
            size: ::std::mem::size_of::<ConstantData>() as _,
        };

        let pipeline_layout_info = VkPipelineLayoutCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            setLayoutCount: 1,
            pSetLayouts: &self.descriptor_set_layout,
            pushConstantRangeCount: 1,
            pPushConstantRanges: &push_constant_range,
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
                    device.get_pipeline_cache(),
                    1,
                    &pipeline_info,
                    ::std::ptr::null_mut(),
                    option.as_mut_ptr()
                )
            );
            option.assume_init()
        };

        self
    }

    fn bind(&mut self, device: &Device, commands: &[InstanceCommand], instances: &[InstanceData]) {
        self.prepare_indirect_commands(device, commands)
            .fill_instance_buffer(device, instances);

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

    fn remove_shader(&mut self, device: &Device, shader_type: ShaderType) {
        self.shaders.retain(|s| {
            if s.get_type() == shader_type {
                s.destroy(device.get_device());
                false
            } else {
                true
            }
        })
    }

    fn destroy_shader_modules(&self, device: &Device) {
        for shader in self.shaders.iter() {
            shader.destroy(device.get_device());
        }
    }

    fn create_descriptor_set_layout(&mut self, device: &Device) -> &mut Self {
        let bindings = vec![
            VkDescriptorSetLayoutBinding {
                binding: 0,
                descriptorCount: 1,
                descriptorType: VkDescriptorType_VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
                pImmutableSamplers: ::std::ptr::null_mut(),
                stageFlags: (VkShaderStageFlagBits_VK_SHADER_STAGE_VERTEX_BIT
                    | VkShaderStageFlagBits_VK_SHADER_STAGE_FRAGMENT_BIT)
                    as _,
            },
            VkDescriptorSetLayoutBinding {
                binding: 1,
                descriptorCount: MAX_TEXTURE_COUNT as _,
                descriptorType: VkDescriptorType_VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
                pImmutableSamplers: ::std::ptr::null_mut(),
                stageFlags: VkShaderStageFlagBits_VK_SHADER_STAGE_FRAGMENT_BIT as _,
            },
        ];

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

        self
    }

    fn fill_instance_buffer(&mut self, device: &Device, instances: &[InstanceData]) -> &mut Self {
        if instances.len() > self.instance_buffer_count {
            device.destroy_buffer(&self.instance_buffer, &self.instance_buffer_memory);
            self.instance_buffer_count = instances.len() * 2;
            let buffer_size = std::mem::size_of::<InstanceData>() * self.instance_buffer_count;
            let flags = VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT
                | VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_HOST_COHERENT_BIT;
            let usage = VkBufferUsageFlagBits_VK_BUFFER_USAGE_STORAGE_BUFFER_BIT
                | VkBufferUsageFlagBits_VK_BUFFER_USAGE_VERTEX_BUFFER_BIT;
            device.create_buffer(
                buffer_size as _,
                usage as _,
                flags as _,
                &mut self.instance_buffer,
                &mut self.instance_buffer_memory,
            );
        }
        if !instances.is_empty() {
            device.map_buffer_memory(&mut self.instance_buffer_memory, 0, instances);
        }
        self
    }

    fn prepare_indirect_commands(
        &mut self,
        device: &Device,
        commands: &[InstanceCommand],
    ) -> &mut Self {
        self.indirect_commands.clear();
        if commands.is_empty() {
            return self;
        }

        for c in commands.iter() {
            let indirect_command = VkDrawIndexedIndirectCommand {
                indexCount: (c.mesh_data_ref.last_index - c.mesh_data_ref.first_index) as _,
                instanceCount: 1,
                firstIndex: c.mesh_data_ref.first_index as _,
                vertexOffset: c.mesh_data_ref.first_vertex as _,
                firstInstance: c.mesh_index as _,
            };
            self.indirect_commands.push(indirect_command);
        }

        if commands.len() > self.indirect_command_buffer_count {
            device.destroy_buffer(
                &self.indirect_command_buffer,
                &self.indirect_command_buffer_memory,
            );
            self.indirect_command_buffer_count = commands.len() * 2;
            let indirect_buffer_size = std::mem::size_of::<VkDrawIndexedIndirectCommand>()
                * self.indirect_command_buffer_count;
            let flags = VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT
                | VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_HOST_COHERENT_BIT;
            let usage = VkBufferUsageFlagBits_VK_BUFFER_USAGE_STORAGE_BUFFER_BIT
                | VkBufferUsageFlagBits_VK_BUFFER_USAGE_INDIRECT_BUFFER_BIT;
            device.create_buffer(
                indirect_buffer_size as _,
                usage as _,
                flags as _,
                &mut self.indirect_command_buffer,
                &mut self.indirect_command_buffer_memory,
            );
        }

        if !self.indirect_commands.is_empty() {
            device.map_buffer_memory(
                &mut self.indirect_command_buffer_memory,
                0,
                self.indirect_commands.as_slice(),
            );
        }

        self
    }

    fn bind_indirect(&mut self, device: &Device) -> &mut Self {
        unsafe {
            let offsets = [0_u64];
            vkCmdBindVertexBuffers.unwrap()(
                device.get_current_command_buffer(),
                INSTANCE_BUFFER_BIND_ID as _,
                1,
                &mut self.instance_buffer,
                offsets.as_ptr(),
            );
        }
        self
    }
    fn draw_indirect(&mut self, device: &Device, count: usize) {
        if count > 0 {
            unsafe {
                vkCmdDrawIndexedIndirect.unwrap()(
                    device.get_current_command_buffer(),
                    self.indirect_command_buffer,
                    0,
                    count as _,
                    std::mem::size_of::<VkDrawIndexedIndirectCommand>() as _,
                );
            }
        }
    }

    fn update_constant_data(
        &mut self,
        device: &Device,
        width: f32,
        height: f32,
        view: &Matrix4,
        proj: &Matrix4,
    ) {
        let details = device.get_instance().get_swap_chain_info();
        self.constant_data.view_width = width as _;
        self.constant_data.view_height = height as _;
        self.constant_data.screen_width = details.capabilities.currentExtent.width as _;
        self.constant_data.screen_height = details.capabilities.currentExtent.height as _;
        self.constant_data.view = matrix4_to_array(*view);
        self.constant_data.proj = matrix4_to_array(*proj);

        unsafe {
            vkCmdPushConstants.unwrap()(
                device.get_current_command_buffer(),
                self.pipeline_layout,
                VkShaderStageFlagBits_VK_SHADER_STAGE_ALL_GRAPHICS as _,
                0,
                ::std::mem::size_of::<ConstantData>() as _,
                &self.constant_data as *const ConstantData as _,
            );
        }
    }

    fn update_uniform_buffer(&mut self, device: &Device, view: &Matrix4, proj: &Matrix4) {
        let image_index = device.get_current_buffer_index();
        let uniform_data: [UniformData; 1] = [UniformData {
            view: *view,
            proj: *proj,
        }];

        let mut buffer_memory = self.uniform_buffers_memory[image_index];
        device.map_buffer_memory(&mut buffer_memory, 0, &uniform_data);
        self.uniform_buffers_memory[image_index] = buffer_memory;

        let buffer_info = VkDescriptorBufferInfo {
            buffer: self.uniform_buffers[image_index],
            offset: 0,
            range: self.uniform_buffers_size as _,
        };

        let descriptor_write: Vec<VkWriteDescriptorSet> = vec![VkWriteDescriptorSet {
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
        }];

        unsafe {
            vkUpdateDescriptorSets.unwrap()(
                device.get_device(),
                descriptor_write.len() as _,
                descriptor_write.as_ptr(),
                0,
                ::std::ptr::null_mut(),
            );
        }
    }

    pub fn update_descriptor_sets(&mut self, device: &Device, textures: &[TextureAtlas]) {
        debug_assert!(
            !textures.is_empty(),
            "At least one texture should be received"
        );

        let image_index = device.get_current_buffer_index();

        let mut descriptor_write: Vec<VkWriteDescriptorSet> = Vec::new();
        let mut descriptors = Vec::new();
        for t in textures.iter() {
            descriptors.push(t.get_texture().get_descriptor());
        }
        descriptor_write.push(VkWriteDescriptorSet {
            sType: VkStructureType_VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
            pNext: ::std::ptr::null_mut(),
            dstSet: self.descriptor_sets[image_index],
            dstBinding: 1,
            dstArrayElement: 0,
            descriptorCount: descriptors.len() as _,
            descriptorType: VkDescriptorType_VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
            pImageInfo: descriptors.as_ptr(),
            pBufferInfo: ::std::ptr::null_mut(),
            pTexelBufferView: ::std::ptr::null_mut(),
        });

        unsafe {
            vkUpdateDescriptorSets.unwrap()(
                device.get_device(),
                descriptor_write.len() as _,
                descriptor_write.as_ptr(),
                0,
                ::std::ptr::null_mut(),
            );
        }
    }

    pub fn bind_descriptors(&self, device: &Device) {
        let image_index = device.get_current_buffer_index();

        unsafe {
            vkCmdBindDescriptorSets.unwrap()(
                device.get_current_command_buffer(),
                VkPipelineBindPoint_VK_PIPELINE_BIND_POINT_GRAPHICS,
                self.pipeline_layout,
                0,
                1,
                &self.descriptor_sets[image_index],
                0,
                ::std::ptr::null_mut(),
            );
        }
    }
}
