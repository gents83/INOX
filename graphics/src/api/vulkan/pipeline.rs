use super::data_formats::INSTANCE_BUFFER_BIND_ID;
use super::shader::BackendShader;
use super::{BackendCommandBuffer, BackendDevice, BackendRenderPass};
use crate::api::backend::{
    create_buffer, destroy_buffer, map_buffer_memory, physical_device::BackendPhysicalDevice,
};

use crate::utils::read_spirv_from_bytes;
use crate::{
    CullingModeType, InstanceCommand, InstanceData, PolygonModeType, ShaderType, VertexData,
};
use nrg_filesystem::convert_from_local_path;

use nrg_resources::DATA_FOLDER;
use std::path::{Path, PathBuf};
use vulkan_bindings::*;

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

        unsafe {
            vkDestroyPipeline.unwrap()(**device, self.graphics_pipeline, ::std::ptr::null_mut());
        }
    }

    pub fn build(
        &mut self,
        device: &BackendDevice,
        render_pass: &BackendRenderPass,
        culling: &CullingModeType,
        mode: &PolygonModeType,
    ) -> &mut Self {
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
            polygonMode: match *mode {
                PolygonModeType::Line => VkPolygonMode_VK_POLYGON_MODE_LINE,
                PolygonModeType::Point => VkPolygonMode_VK_POLYGON_MODE_POINT,
                _ => VkPolygonMode_VK_POLYGON_MODE_FILL,
            },
            cullMode: match *culling {
                CullingModeType::Back => {
                    VkCullModeFlagBits_VK_CULL_MODE_BACK_BIT as VkCullModeFlags
                }
                CullingModeType::Front => {
                    VkCullModeFlagBits_VK_CULL_MODE_FRONT_BIT as VkCullModeFlags
                }
                CullingModeType::Both => {
                    VkCullModeFlagBits_VK_CULL_MODE_FRONT_AND_BACK as VkCullModeFlags
                }
                _ => VkCullModeFlagBits_VK_CULL_MODE_NONE as VkCullModeFlags,
            },
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
            layout: device.get_pipeline_layout(),
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
        command_buffer: &BackendCommandBuffer,
        commands: &[InstanceCommand],
        instances: &[InstanceData],
    ) -> &mut Self {
        self.prepare_indirect_commands(device, physical_device, commands)
            .fill_instance_buffer(device, physical_device, instances)
            .bind_pipeline(command_buffer);
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
            map_buffer_memory(device, &mut self.instance_buffer_memory, 0, instances);
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
            map_buffer_memory(
                device,
                &mut self.indirect_command_buffer_memory,
                0,
                self.indirect_commands.as_slice(),
            );
        }

        self
    }

    pub fn bind_instance_buffer(&mut self, command_buffer: &BackendCommandBuffer) -> &mut Self {
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
        self
    }
    pub fn draw_indirect(&self, command_buffer: &BackendCommandBuffer, count: usize) {
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
    pub fn draw_indexed(
        &self,
        command_buffer: &BackendCommandBuffer,
        instance_commands: &[InstanceCommand],
        instance_count: usize,
    ) {
        unsafe {
            for i in 0..instance_count {
                if let Some(c) = instance_commands.iter().find(|c| c.mesh_index == i) {
                    if c.mesh_index == i {
                        vkCmdDrawIndexed.unwrap()(
                            command_buffer.get(),
                            c.mesh_data_ref.last_index - c.mesh_data_ref.first_index,
                            1,
                            c.mesh_data_ref.first_index,
                            c.mesh_data_ref.first_vertex as _,
                            0,
                        );
                    }
                }
            }
        }
    }
}
