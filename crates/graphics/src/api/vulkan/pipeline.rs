use super::data_formats::INSTANCE_BUFFER_BIND_ID;
use super::shader::BackendShader;
use super::{BackendCommandBuffer, BackendDevice, BackendRenderPass};
use crate::api::backend::{
    copy_from_buffer, create_buffer, destroy_buffer, physical_device::BackendPhysicalDevice,
};

use crate::utils::read_spirv_from_bytes;
use crate::{
    ConstantData, InstanceCommand, InstanceData, LightData, PipelineData, ShaderData,
    ShaderMaterialData, ShaderTextureData, ShaderType, TextureAtlas, VertexData, MAX_NUM_LIGHTS,
    MAX_NUM_MATERIALS, MAX_NUM_TEXTURES, MAX_TEXTURE_ATLAS_COUNT,
};
use nrg_filesystem::convert_from_local_path;

use nrg_math::{matrix4_to_array, Matrix4};
use nrg_profiler::debug_log;
use nrg_resources::DATA_FOLDER;
use std::path::{Path, PathBuf};
use vulkan_bindings::*;

#[derive(Clone)]
pub struct BackendPipeline {
    shaders: Vec<BackendShader>,
    graphics_pipeline: VkPipeline,
    instance_buffer_count: usize,
    instance_buffer: VkBuffer,
    instance_buffer_memory: VkDeviceMemory,
    indirect_command_buffer_count: usize,
    indirect_command_buffer: VkBuffer,
    indirect_command_buffer_memory: VkDeviceMemory,
    indirect_commands: Vec<VkDrawIndexedIndirectCommand>,
    descriptor_set_layout: VkDescriptorSetLayout,
    descriptor_pool: VkDescriptorPool,
    descriptor_sets: Vec<VkDescriptorSet>,
    data_buffers_size: usize,
    data_buffers: Vec<VkBuffer>,
    data_buffers_memory: Vec<VkDeviceMemory>,
    pipeline_layout: VkPipelineLayout,
}
impl Default for BackendPipeline {
    fn default() -> Self {
        Self {
            shaders: Vec::new(),
            graphics_pipeline: ::std::ptr::null_mut(),
            instance_buffer_count: 0,
            instance_buffer: ::std::ptr::null_mut(),
            instance_buffer_memory: ::std::ptr::null_mut(),
            indirect_command_buffer_count: 0,
            indirect_command_buffer: ::std::ptr::null_mut(),
            indirect_command_buffer_memory: ::std::ptr::null_mut(),
            indirect_commands: Vec::new(),
            descriptor_set_layout: ::std::ptr::null_mut(),
            descriptor_sets: Vec::new(),
            descriptor_pool: ::std::ptr::null_mut(),
            data_buffers_size: 0,
            data_buffers: Vec::new(),
            data_buffers_memory: Vec::new(),
            pipeline_layout: ::std::ptr::null_mut(),
        }
    }
}

unsafe impl Send for BackendPipeline {}
unsafe impl Sync for BackendPipeline {}

impl BackendPipeline {
    pub fn set_shader(
        &mut self,
        device: &BackendDevice,
        shader_type: ShaderType,
        shader_filepath: &Path,
    ) -> &mut Self {
        let path = convert_from_local_path(PathBuf::from(DATA_FOLDER).as_path(), shader_filepath);
        if path.exists() && path.is_file() {
            self.remove_shader(device, shader_type);

            let mut shader_file = std::fs::File::open(path).unwrap();
            let shader_code = read_spirv_from_bytes(&mut shader_file);

            self.create_shader_module(device, shader_type, shader_code, "main");
        }
        self
    }

    pub fn destroy(&self, device: &BackendDevice) {
        self.destroy_shader_modules(device);
        destroy_buffer(
            device,
            &self.indirect_command_buffer,
            &self.indirect_command_buffer_memory,
        );
        destroy_buffer(device, &self.instance_buffer, &self.instance_buffer_memory);

        for i in 0..self.data_buffers.len() {
            destroy_buffer(device, &self.data_buffers[i], &self.data_buffers_memory[i]);
        }
        unsafe {
            let images_count = device.get_images_count();
            vkFreeDescriptorSets.unwrap()(
                **device,
                self.descriptor_pool,
                images_count as _,
                self.descriptor_sets.as_ptr(),
            );
            vkDestroyDescriptorSetLayout.unwrap()(
                **device,
                self.descriptor_set_layout,
                ::std::ptr::null_mut(),
            );
            vkDestroyPipelineLayout.unwrap()(
                **device,
                self.pipeline_layout,
                ::std::ptr::null_mut(),
            );
            vkDestroyDescriptorPool.unwrap()(
                **device,
                self.descriptor_pool,
                ::std::ptr::null_mut(),
            );

            vkDestroyPipeline.unwrap()(**device, self.graphics_pipeline, ::std::ptr::null_mut());
        }
    }

    pub fn build(
        &mut self,
        device: &BackendDevice,
        physical_device: &BackendPhysicalDevice,
        render_pass: &BackendRenderPass,
        data: &PipelineData,
    ) -> &mut Self {
        self.create_descriptor_set_layout(device)
            .create_data_buffer(device, physical_device)
            .create_descriptor_pool(device)
            .create_descriptor_sets(device)
            .create_pipeline_layout(device);

        let mut shader_stages: Vec<VkPipelineShaderStageCreateInfo> = Vec::new();
        for shader in self.shaders.iter() {
            shader_stages.push(shader.stage_info());
        }

        let mut vertex_data_binding_info = vec![VertexData::get_binding_desc()];
        let mut vertex_data_attr_info = VertexData::get_attributes_desc();

        let instance_data_binding_info = InstanceData::get_binding_desc();
        let mut instance_data_attr_info = InstanceData::get_attributes_desc();
        vertex_data_binding_info.push(instance_data_binding_info);
        vertex_data_attr_info.append(&mut instance_data_attr_info);

        let vertex_input_info = VkPipelineVertexInputStateCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            vertexBindingDescriptionCount: vertex_data_binding_info.len() as _,
            pVertexBindingDescriptions: vertex_data_binding_info.as_slice().as_ptr(),
            vertexAttributeDescriptionCount: vertex_data_attr_info.len() as _,
            pVertexAttributeDescriptions: vertex_data_attr_info.as_slice().as_ptr(),
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
            polygonMode: data.mode.into(),
            cullMode: data.culling.into(),
            frontFace: VkFrontFace_VK_FRONT_FACE_COUNTER_CLOCKWISE,
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
            srcColorBlendFactor: data.src_color_blend_factor.into(),
            dstColorBlendFactor: data.dst_color_blend_factor.into(),
            colorBlendOp: VkBlendOp_VK_BLEND_OP_ADD,
            srcAlphaBlendFactor: data.src_alpha_blend_factor.into(),
            dstAlphaBlendFactor: data.dst_alpha_blend_factor.into(),
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
            blendConstants: [1., 1., 1., 1.],
        };

        let dynamic_states = vec![
            VkDynamicState_VK_DYNAMIC_STATE_VIEWPORT,
            VkDynamicState_VK_DYNAMIC_STATE_SCISSOR,
        ];
        let dynamic_state = VkPipelineDynamicStateCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_PIPELINE_DYNAMIC_STATE_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            dynamicStateCount: dynamic_states.len() as _,
            pDynamicStates: dynamic_states.as_ptr(),
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
            pDynamicState: &dynamic_state,
            layout: self.pipeline_layout,
            renderPass: **render_pass,
            subpass: 0,
            basePipelineHandle: ::std::ptr::null_mut(),
            basePipelineIndex: -1,
        };

        self.graphics_pipeline = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkCreateGraphicsPipelines.unwrap()(
                    **device,
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

    pub fn bind_pipeline(&mut self, command_buffer: &BackendCommandBuffer) -> &mut Self {
        unsafe {
            vkCmdBindPipeline.unwrap()(
                command_buffer.get(),
                VkPipelineBindPoint_VK_PIPELINE_BIND_POINT_GRAPHICS,
                self.graphics_pipeline,
            );
        }
        self
    }

    pub fn bind_indirect(
        &mut self,
        device: &BackendDevice,
        physical_device: &BackendPhysicalDevice,
        commands: &[InstanceCommand],
        instances: &[InstanceData],
    ) -> &mut Self {
        self.prepare_indirect_commands(device, physical_device, commands)
            .fill_instance_buffer(device, physical_device, instances);
        self
    }

    fn create_shader_module(
        &mut self,
        device: &BackendDevice,
        shader_type: ShaderType,
        shader_content: Vec<u32>,
        entry_point: &'static str,
    ) -> &mut Self {
        let shader = BackendShader::create(device, shader_type, shader_content, entry_point);
        self.shaders.push(shader);
        self
    }

    fn remove_shader(&mut self, device: &BackendDevice, shader_type: ShaderType) {
        self.shaders.retain(|s| {
            if s.get_type() == shader_type {
                s.destroy(device);
                false
            } else {
                true
            }
        })
    }

    fn destroy_shader_modules(&self, device: &BackendDevice) {
        for shader in self.shaders.iter() {
            shader.destroy(device);
        }
    }

    pub fn fill_instance_buffer(
        &mut self,
        device: &BackendDevice,
        physical_device: &BackendPhysicalDevice,
        instances: &[InstanceData],
    ) -> &mut Self {
        if instances.len() > self.instance_buffer_count {
            destroy_buffer(device, &self.instance_buffer, &self.instance_buffer_memory);
            self.instance_buffer_count = instances.len() * 2;
            let buffer_size = std::mem::size_of::<InstanceData>() * self.instance_buffer_count;
            let flags = VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT
                | VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_HOST_COHERENT_BIT;
            let usage = VkBufferUsageFlagBits_VK_BUFFER_USAGE_STORAGE_BUFFER_BIT
                | VkBufferUsageFlagBits_VK_BUFFER_USAGE_VERTEX_BUFFER_BIT;
            create_buffer(
                device,
                physical_device,
                buffer_size as _,
                usage as _,
                flags as _,
                &mut self.instance_buffer,
                &mut self.instance_buffer_memory,
            );
        }
        if !instances.is_empty() {
            copy_from_buffer(device, &mut self.instance_buffer_memory, 0, instances);
        }
        self
    }

    pub fn prepare_indirect_commands(
        &mut self,
        device: &BackendDevice,
        physical_device: &BackendPhysicalDevice,
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
            destroy_buffer(
                device,
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
            create_buffer(
                device,
                physical_device,
                indirect_buffer_size as _,
                usage as _,
                flags as _,
                &mut self.indirect_command_buffer,
                &mut self.indirect_command_buffer_memory,
            );
        }

        if !self.indirect_commands.is_empty() {
            copy_from_buffer(
                device,
                &mut self.indirect_command_buffer_memory,
                0,
                self.indirect_commands.as_slice(),
            );
        }

        self
    }

    pub fn bind_instance_buffer(&mut self, command_buffer: &BackendCommandBuffer) -> &mut Self {
        if !self.instance_buffer.is_null() {
            unsafe {
                let offsets = [0_u64];
                vkCmdBindVertexBuffers.unwrap()(
                    command_buffer.get(),
                    INSTANCE_BUFFER_BIND_ID as _,
                    1,
                    &mut self.instance_buffer,
                    offsets.as_ptr(),
                );
            }
        }
        self
    }
    pub fn draw_indirect_batch(&self, command_buffer: &BackendCommandBuffer, count: usize) {
        if count > 0 {
            unsafe {
                vkCmdDrawIndexedIndirect.unwrap()(
                    command_buffer.get(),
                    self.indirect_command_buffer,
                    0,
                    count as _,
                    std::mem::size_of::<VkDrawIndexedIndirectCommand>() as _,
                );
            }
        }
    }
    pub fn draw_single(
        &self,
        command_buffer: &BackendCommandBuffer,
        instance_commands: &[InstanceCommand],
        instance_data: &[InstanceData],
        instance_count: usize,
    ) {
        unsafe {
            (0..instance_count).for_each(|i| {
                if let Some(c) = instance_commands.iter().find(|c| c.mesh_index == i) {
                    if c.mesh_index == i {
                        let instance_data_ref = &instance_data[i];
                        let scissors = VkRect2D {
                            offset: VkOffset2D {
                                x: instance_data_ref.draw_area.x.round() as _,
                                y: instance_data_ref.draw_area.y.round() as _,
                            },
                            extent: VkExtent2D {
                                width: ((instance_data_ref.draw_area.z
                                    - instance_data_ref.draw_area.x)
                                    .max(1.))
                                .round() as _,
                                height: ((instance_data_ref.draw_area.w
                                    - instance_data_ref.draw_area.y)
                                    .max(1.))
                                .round() as _,
                            },
                        };
                        vkCmdSetScissor.unwrap()(command_buffer.get(), 0, 1, &scissors);
                        let size = std::mem::size_of::<VkDrawIndexedIndirectCommand>();
                        vkCmdDrawIndexedIndirect.unwrap()(
                            command_buffer.get(),
                            self.indirect_command_buffer,
                            (i * size) as _,
                            1,
                            0,
                        );
                    }
                }
            });
        }
    }

    fn create_data_buffer(
        &mut self,
        device: &BackendDevice,
        physical_device: &BackendPhysicalDevice,
    ) -> &mut Self {
        let images_count = device.get_images_count();
        let mut data_buffers = Vec::<VkBuffer>::with_capacity(images_count);
        let mut data_buffers_memory = Vec::<VkDeviceMemory>::with_capacity(images_count);
        unsafe {
            data_buffers.set_len(images_count);
            data_buffers_memory.set_len(images_count);
        }

        let data_buffers_size = std::mem::size_of::<ShaderData>();
        //ShaderData::debug_size();

        let flags = VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT
            | VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_HOST_COHERENT_BIT;
        for i in 0..data_buffers.len() {
            create_buffer(
                device,
                physical_device,
                data_buffers_size as _,
                VkBufferUsageFlagBits_VK_BUFFER_USAGE_STORAGE_BUFFER_BIT as _,
                flags as _,
                &mut data_buffers[i],
                &mut data_buffers_memory[i],
            );
        }

        self.data_buffers_size = data_buffers_size;
        self.data_buffers = data_buffers;
        self.data_buffers_memory = data_buffers_memory;
        self
    }
    fn create_descriptor_pool(&mut self, device: &BackendDevice) -> &mut Self {
        if !self.descriptor_pool.is_null() {
            unsafe {
                vkDestroyDescriptorPool.unwrap()(
                    **device,
                    self.descriptor_pool,
                    ::std::ptr::null_mut(),
                );
            }
        }
        let images_count = device.get_images_count();
        let pool_sizes: Vec<VkDescriptorPoolSize> = vec![
            VkDescriptorPoolSize {
                type_: VkDescriptorType_VK_DESCRIPTOR_TYPE_STORAGE_BUFFER,
                descriptorCount: images_count as u32,
            },
            VkDescriptorPoolSize {
                type_: VkDescriptorType_VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
                descriptorCount: MAX_TEXTURE_ATLAS_COUNT as u32 * images_count as u32,
            },
        ];

        let pool_info = VkDescriptorPoolCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO,
            flags: VkDescriptorPoolCreateFlagBits_VK_DESCRIPTOR_POOL_CREATE_UPDATE_AFTER_BIND_BIT
                as _,
            pNext: ::std::ptr::null_mut(),
            poolSizeCount: pool_sizes.len() as _,
            pPoolSizes: pool_sizes.as_ptr(),
            maxSets: images_count as _,
        };

        self.descriptor_pool = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkCreateDescriptorPool.unwrap()(
                    **device,
                    &pool_info,
                    ::std::ptr::null_mut(),
                    option.as_mut_ptr()
                )
            );
            option.assume_init()
        };
        self
    }
    pub fn create_descriptor_sets(&mut self, device: &BackendDevice) -> &mut Self {
        let images_count = device.get_images_count();
        if !self.descriptor_sets.is_empty() {
            unsafe {
                vkFreeDescriptorSets.unwrap()(
                    **device,
                    self.descriptor_pool,
                    images_count as _,
                    self.descriptor_sets.as_ptr(),
                );
            }
        }

        let mut layouts = Vec::<VkDescriptorSetLayout>::with_capacity(images_count);
        unsafe {
            layouts.set_len(images_count);
        }
        for layout in layouts.iter_mut() {
            *layout = self.descriptor_set_layout;
        }

        let alloc_info = VkDescriptorSetAllocateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO,
            pNext: ::std::ptr::null_mut(),
            descriptorPool: self.descriptor_pool,
            descriptorSetCount: images_count as _,
            pSetLayouts: layouts.as_mut_ptr(),
        };

        let mut descriptor_sets = Vec::<VkDescriptorSet>::with_capacity(images_count);
        unsafe {
            descriptor_sets.set_len(images_count);
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkAllocateDescriptorSets.unwrap()(
                    **device,
                    &alloc_info,
                    descriptor_sets.as_mut_ptr()
                )
            );
        }

        self.descriptor_sets = descriptor_sets;
        self
    }

    fn create_pipeline_layout(&mut self, device: &BackendDevice) -> &mut Self {
        if !self.pipeline_layout.is_null() {
            unsafe {
                vkDestroyPipelineLayout.unwrap()(
                    **device,
                    self.pipeline_layout,
                    ::std::ptr::null_mut(),
                );
            }
        }

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
                    **device,
                    &pipeline_layout_info,
                    ::std::ptr::null_mut(),
                    option.as_mut_ptr()
                )
            );
            option.assume_init()
        };
        self
    }

    fn create_descriptor_set_layout(&mut self, device: &BackendDevice) -> &mut Self {
        if !self.descriptor_set_layout.is_null() {
            unsafe {
                vkDestroyDescriptorSetLayout.unwrap()(
                    **device,
                    self.descriptor_set_layout,
                    ::std::ptr::null_mut(),
                );
            }
        }
        let bindings = vec![
            VkDescriptorSetLayoutBinding {
                binding: 0,
                descriptorCount: 1,
                descriptorType: VkDescriptorType_VK_DESCRIPTOR_TYPE_STORAGE_BUFFER,
                pImmutableSamplers: ::std::ptr::null_mut(),
                stageFlags: (VkShaderStageFlagBits_VK_SHADER_STAGE_VERTEX_BIT
                    | VkShaderStageFlagBits_VK_SHADER_STAGE_FRAGMENT_BIT)
                    as _,
            },
            VkDescriptorSetLayoutBinding {
                binding: 1,
                descriptorCount: MAX_TEXTURE_ATLAS_COUNT as _,
                descriptorType: VkDescriptorType_VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
                pImmutableSamplers: ::std::ptr::null_mut(),
                stageFlags: VkShaderStageFlagBits_VK_SHADER_STAGE_FRAGMENT_BIT as _,
            },
        ];

        let layout_create_info = VkDescriptorSetLayoutCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            flags: VkDescriptorSetLayoutCreateFlagBits_VK_DESCRIPTOR_SET_LAYOUT_CREATE_UPDATE_AFTER_BIND_POOL_BIT as _,
            pNext: ::std::ptr::null_mut(),
            bindingCount: bindings.len() as _,
            pBindings: bindings.as_ptr(),
        };

        self.descriptor_set_layout = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkCreateDescriptorSetLayout.unwrap()(
                    **device,
                    &layout_create_info,
                    ::std::ptr::null_mut(),
                    option.as_mut_ptr()
                )
            );
            option.assume_init()
        };

        self
    }

    pub fn update_constant_data(
        &self,
        command_buffer: &BackendCommandBuffer,
        width: u32,
        height: u32,
        view: &Matrix4,
        proj: &Matrix4,
    ) -> &Self {
        #[rustfmt::skip]
        const LEFT_HANDED_VULKAN_MATRIX: Matrix4 = Matrix4::new(
             1.0,  0.0, 0.0, 0.0,
             0.0, -1.0, 0.0, 0.0,
             0.0,  0.0, 0.5, 0.0,
             0.0,  0.0, 0.5, 1.0,
        );
        let constant_data = ConstantData {
            view: matrix4_to_array(*view),
            proj: matrix4_to_array(LEFT_HANDED_VULKAN_MATRIX * *proj),
            screen_width: width as _,
            screen_height: height as _,
            ..Default::default()
        };

        unsafe {
            vkCmdPushConstants.unwrap()(
                command_buffer.get(),
                self.pipeline_layout,
                VkShaderStageFlagBits_VK_SHADER_STAGE_ALL_GRAPHICS as _,
                0,
                ::std::mem::size_of::<ConstantData>() as _,
                &constant_data as *const ConstantData as _,
            );
        }
        self
    }

    pub fn update_data_buffer(
        &self,
        device: &BackendDevice,
        light_data: &[LightData],
        texture_data: &[ShaderTextureData],
        material_data: &[ShaderMaterialData],
    ) -> &Self {
        let image_index = device.get_current_image_index();
        let mut uniform_data: [ShaderData; 1] = [ShaderData::default(); 1];
        if light_data.len() >= MAX_NUM_LIGHTS {
            debug_log(
                format!(
                    "Too many lights, max supported number is {} instead of {}",
                    MAX_NUM_LIGHTS,
                    light_data.len()
                )
                .as_str(),
            );
            uniform_data[0].num_lights = MAX_NUM_LIGHTS as _;
        } else {
            uniform_data[0].num_lights = light_data.len() as _;
        }
        if texture_data.len() >= MAX_NUM_TEXTURES {
            debug_log(
                format!(
                    "Too many textures, max supported number is {} instead of {}",
                    MAX_NUM_TEXTURES,
                    texture_data.len()
                )
                .as_str(),
            );
            uniform_data[0].num_textures = MAX_NUM_TEXTURES as _;
        } else {
            uniform_data[0].num_textures = texture_data.len() as _;
        }
        if material_data.len() >= MAX_NUM_MATERIALS {
            debug_log(
                format!(
                    "Too many materials, max supported number is {} instead of {}",
                    MAX_NUM_MATERIALS,
                    material_data.len()
                )
                .as_str(),
            );
            uniform_data[0].num_materials = MAX_NUM_MATERIALS as _;
        } else {
            uniform_data[0].num_materials = material_data.len() as _;
        }

        (0..uniform_data[0].num_lights as usize).for_each(|i| {
            uniform_data[0].light_data[i] = light_data[i];
        });
        (0..uniform_data[0].num_textures as usize).for_each(|i| {
            uniform_data[0].textures_data[i] = texture_data[i];
        });
        (0..uniform_data[0].num_materials as usize).for_each(|i| {
            uniform_data[0].materials_data[i] = material_data[i];
        });

        let mut buffer_memory = self.data_buffers_memory[image_index];
        copy_from_buffer(device, &mut buffer_memory, 0, &uniform_data);

        let buffer_info = VkDescriptorBufferInfo {
            buffer: self.data_buffers[image_index],
            offset: 0,
            range: self.data_buffers_size as _,
        };

        let descriptor_write: Vec<VkWriteDescriptorSet> = vec![VkWriteDescriptorSet {
            sType: VkStructureType_VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
            pNext: ::std::ptr::null_mut(),
            dstSet: self.descriptor_sets[image_index],
            dstBinding: 0,
            dstArrayElement: 0,
            descriptorCount: 1,
            descriptorType: VkDescriptorType_VK_DESCRIPTOR_TYPE_STORAGE_BUFFER,
            pImageInfo: ::std::ptr::null_mut(),
            pBufferInfo: &buffer_info,
            pTexelBufferView: ::std::ptr::null_mut(),
        }];

        unsafe {
            vkUpdateDescriptorSets.unwrap()(
                **device,
                descriptor_write.len() as _,
                descriptor_write.as_ptr(),
                0,
                ::std::ptr::null_mut(),
            );
        }
        self
    }

    pub fn update_descriptor_sets(
        &self,
        device: &BackendDevice,
        textures: &[TextureAtlas],
        used_textures: &[bool],
    ) -> &Self {
        debug_assert!(
            !textures.is_empty(),
            "At least one texture should be received"
        );
        debug_assert!(
            textures.len() <= MAX_TEXTURE_ATLAS_COUNT,
            "Max num textures exceeded"
        );
        debug_assert!(
            textures.len() == used_textures.len(),
            "Size of textures and used textures should be the same"
        );

        let image_index = device.get_current_image_index();

        let mut descriptor_write: Vec<VkWriteDescriptorSet> = Vec::new();
        let mut descriptors = Vec::new();
        (0..MAX_TEXTURE_ATLAS_COUNT).for_each(|i| {
            let index = if i < textures.len() && used_textures[i] {
                i
            } else {
                0
            };
            descriptors.push(textures[index].get_texture().get_descriptor());
        });
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
                **device,
                descriptor_write.len() as _,
                descriptor_write.as_ptr(),
                0,
                ::std::ptr::null_mut(),
            );
        }
        self
    }

    pub fn bind_descriptors(
        &self,
        device: &BackendDevice,
        command_buffer: &BackendCommandBuffer,
    ) -> usize {
        let image_index = device.get_current_image_index();

        unsafe {
            vkCmdBindDescriptorSets.unwrap()(
                command_buffer.get(),
                VkPipelineBindPoint_VK_PIPELINE_BIND_POINT_GRAPHICS,
                self.pipeline_layout,
                0,
                1,
                &self.descriptor_sets[image_index],
                0,
                ::std::ptr::null_mut(),
            );
        }
        image_index
    }
}
