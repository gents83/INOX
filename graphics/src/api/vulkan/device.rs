
use std::os::raw::c_char;
use vulkan_bindings::*;
use super::instance::*;
use super::shader::*;
use super::types::*;
use super::utils::*;

pub struct SwapChain<'a> {
    ptr: VkSwapchainKHR,
    details: &'a SwapChainSupportDetails,
    images: Vec<VkImage>,
    image_views: Vec<VkImageView>,
}

impl From<&SwapChain<'_>> for VkSwapchainKHR {
    fn from(swap_chain: &SwapChain<'_>) -> VkSwapchainKHR {
        swap_chain.ptr
    }
}

pub struct Device<'a> {
    instance: &'a Instance,
    device: VkDevice,
    graphics_queue: VkQueue,
    present_queue: VkQueue,
    swap_chain: SwapChain<'a>,
    shaders: Vec<Shader>,
    render_pass: VkRenderPass,
    pipeline_layout: VkPipelineLayout,
    graphics_pipeline: VkPipeline,
    framebuffers: Vec::<VkFramebuffer>,
    command_pool: VkCommandPool,
    command_buffers: Vec::<VkCommandBuffer>,
    image_available_semaphores: Vec<VkSemaphore>,
    render_finished_semaphores: Vec<VkSemaphore>,
    inflight_fences: Vec<VkFence>,
    inflight_images: Vec<VkFence>,
}

impl<'a> Device<'a> {
    pub fn new(instance: &'a Instance) -> Device {        
        let queue_priority:f32 = 1.0;
        let mut hash_family_indices: ::std::collections::HashSet<u32> = ::std::collections::HashSet::new();
        hash_family_indices.insert(instance.get_physical_device().get_queue_family_info().graphics_family_index as _);
        hash_family_indices.insert(instance.get_physical_device().get_queue_family_info().present_family_index as _);

        let mut queue_infos: Vec<VkDeviceQueueCreateInfo> = Vec::new();
        for (i, family_index) in hash_family_indices.into_iter().enumerate() {
            let queue_create_info = VkDeviceQueueCreateInfo {
                sType: VkStructureType_VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
                pNext: ::std::ptr::null_mut(),
                flags: 0,
                queueFamilyIndex: family_index,
                queueCount: 1,
                pQueuePriorities: &queue_priority,
            };
            queue_infos.push(queue_create_info);
        }

        let layer_names_str = get_available_layers_names(instance.get_supported_layers());
        let layer_names_ptr = layer_names_str
                                            .iter()
                                            .map(|e| e.as_ptr())
                                            .collect::<Vec<*const c_char>>();

        let device_extension_names_str = get_available_extensions_names(instance.get_physical_device().get_available_extensions());
        let device_extension_names_ptr = device_extension_names_str
                                                        .iter()
                                                        .map(|e| e.as_ptr())
                                                        .collect::<Vec<*const c_char>>();

        let device_create_info = VkDeviceCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            queueCreateInfoCount: queue_infos.len() as _,
            pQueueCreateInfos: queue_infos.as_ptr(),
            enabledLayerCount: layer_names_str.len() as _,
            ppEnabledLayerNames: layer_names_ptr.as_ptr(),
            enabledExtensionCount: device_extension_names_ptr.len() as u32,
            ppEnabledExtensionNames: device_extension_names_ptr.as_ptr(),
            pEnabledFeatures: instance.get_physical_device().get_available_features(),
        };

        let mut device: VkDevice = ::std::ptr::null_mut();
        unsafe {
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkCreateDevice.unwrap()(instance.get_physical_device().into(), &device_create_info, ::std::ptr::null_mut(), &mut device)
            );
        }

        let graphics_queue: VkQueue = unsafe {
            let mut output = ::std::mem::MaybeUninit::uninit();
            vkGetDeviceQueue.unwrap()(device, instance.get_physical_device().get_queue_family_info().graphics_family_index as _, 0, output.as_mut_ptr());
            output.assume_init()
        };
        let present_queue: VkQueue = unsafe {
            let mut output = ::std::mem::MaybeUninit::uninit();
            vkGetDeviceQueue.unwrap()(device, instance.get_physical_device().get_queue_family_info().present_family_index as _, 0, output.as_mut_ptr());
            output.assume_init()
        };

        Self {
            instance: instance,
            device: device,
            graphics_queue: graphics_queue,
            present_queue: present_queue,
            swap_chain: SwapChain {
                ptr: ::std::ptr::null_mut(),
                details: instance.get_physical_device().get_swap_chain_info(),
                images: Vec::new(),
                image_views: Vec::new(),
            },
            shaders: Vec::new(),
            render_pass: ::std::ptr::null_mut(),
            pipeline_layout: ::std::ptr::null_mut(),
            graphics_pipeline: ::std::ptr::null_mut(),
            framebuffers: Vec::new(),
            command_pool: ::std::ptr::null_mut(),
            command_buffers: Vec::new(),
            image_available_semaphores: Vec::new(),
            render_finished_semaphores: Vec::new(),
            inflight_fences: Vec::new(),
            inflight_images: Vec::new(),
        }
    }

    pub fn create_swap_chain(&mut self) -> &mut Self {       
        self.swap_chain.details = self.instance.get_physical_device().get_swap_chain_info();
        let queue_family_info = self.instance.get_physical_device().get_queue_family_info();
        let mut family_indices: Vec<u32> = Vec::new();

        let mut swap_chain_create_info = VkSwapchainCreateInfoKHR{
            sType: VkStructureType_VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            surface: self.instance.get_surface(),
            minImageCount: ::std::cmp::max(self.swap_chain.details.capabilities.minImageCount + 1, self.swap_chain.details.capabilities.maxImageCount),
            imageFormat: self.swap_chain.details.formats[0].format,
            imageColorSpace: self.swap_chain.details.formats[0].colorSpace,
            imageExtent: self.swap_chain.details.capabilities.currentExtent,
            imageArrayLayers: 1,
            imageUsage: VkImageUsageFlagBits_VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT as u32,
            imageSharingMode: VkSharingMode_VK_SHARING_MODE_EXCLUSIVE,
            queueFamilyIndexCount: 0,
            pQueueFamilyIndices: family_indices.as_mut_ptr(),
            preTransform: self.swap_chain.details.capabilities.currentTransform,
            compositeAlpha: VkCompositeAlphaFlagBitsKHR_VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR,
            presentMode: self.swap_chain.details.present_modes[0],
            clipped: VK_TRUE,
            oldSwapchain: ::std::ptr::null_mut(),
        };

        if  queue_family_info.graphics_family_index != queue_family_info.present_family_index {
            family_indices.push(queue_family_info.graphics_family_index as _);
            family_indices.push(queue_family_info.present_family_index as _);
            swap_chain_create_info.imageSharingMode = VkSharingMode_VK_SHARING_MODE_CONCURRENT;
            swap_chain_create_info.queueFamilyIndexCount = 2;
            swap_chain_create_info.pQueueFamilyIndices = family_indices.as_mut_ptr();
        }

        self.swap_chain.ptr = unsafe {
            let mut output = ::std::mem::MaybeUninit::uninit();
            vkCreateSwapchainKHR.unwrap()(self.device, &swap_chain_create_info, ::std::ptr::null_mut(), output.as_mut_ptr());
            output.assume_init()
        };

        let mut swapchain_images_count = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            vkGetSwapchainImagesKHR.unwrap()(self.device, self.get_swap_chain().into(), option.as_mut_ptr(), ::std::ptr::null_mut());
            option.assume_init()
        };
        
        let mut swap_chain_images = Vec::with_capacity(swapchain_images_count as usize);
        unsafe {
            swap_chain_images.set_len(swapchain_images_count as usize);
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkGetSwapchainImagesKHR.unwrap()(self.device, self.get_swap_chain().into(), &mut swapchain_images_count, swap_chain_images.as_mut_ptr())
            );
        }     

        self.swap_chain.images = swap_chain_images;
        self
    }

    pub fn create_image_views(&mut self) -> &mut  Self {              
        let mut swap_chain_image_views = Vec::with_capacity(self.swap_chain.images.len());
        unsafe {
            swap_chain_image_views.set_len(self.swap_chain.images.len());
        }

        let selected_format = self.swap_chain.details.formats[0].format;

        for (i, image) in self.swap_chain.images.iter().enumerate() {
            let mut image_view_create_info = VkImageViewCreateInfo {
                sType: VkStructureType_VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
                pNext: ::std::ptr::null_mut(),
                flags: 0,
                image: *image,
                viewType: VkImageViewType_VK_IMAGE_VIEW_TYPE_2D,
                format: selected_format,
                components: VkComponentMapping {
                    r: VkComponentSwizzle_VK_COMPONENT_SWIZZLE_IDENTITY,
                    g: VkComponentSwizzle_VK_COMPONENT_SWIZZLE_IDENTITY,
                    b: VkComponentSwizzle_VK_COMPONENT_SWIZZLE_IDENTITY,
                    a: VkComponentSwizzle_VK_COMPONENT_SWIZZLE_IDENTITY,
                },
                subresourceRange : VkImageSubresourceRange {
                    aspectMask: VkImageAspectFlagBits_VK_IMAGE_ASPECT_COLOR_BIT as VkImageAspectFlags,
                    baseMipLevel: 0,
                    levelCount: 1,
                    baseArrayLayer: 0,
                    layerCount: 1
                },
            };
            unsafe {
                assert_eq!(
                    VkResult_VK_SUCCESS,
                    vkCreateImageView.unwrap()(self.device, &mut image_view_create_info, ::std::ptr::null_mut(), &mut swap_chain_image_views[i])
                );
            }
        }
        self.swap_chain.image_views = swap_chain_image_views;
        self
    }

    pub fn create_render_pass(&mut self) -> &mut Self {        
        let color_attachment = VkAttachmentDescription {
            flags: 0,
            format: self.swap_chain.details.formats[0].format,
            samples: VkSampleCountFlagBits_VK_SAMPLE_COUNT_1_BIT,
            loadOp: VkAttachmentLoadOp_VK_ATTACHMENT_LOAD_OP_CLEAR,
            storeOp: VkAttachmentStoreOp_VK_ATTACHMENT_STORE_OP_STORE,
            stencilLoadOp: VkAttachmentLoadOp_VK_ATTACHMENT_LOAD_OP_DONT_CARE,
            stencilStoreOp: VkAttachmentStoreOp_VK_ATTACHMENT_STORE_OP_DONT_CARE,
            initialLayout: VkImageLayout_VK_IMAGE_LAYOUT_UNDEFINED,
            finalLayout: VkImageLayout_VK_IMAGE_LAYOUT_PRESENT_SRC_KHR,
        };

        let color_attachment_ref = VkAttachmentReference {
            attachment: 0,
            layout: VkImageLayout_VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL,
        };

        let subpass1 = VkSubpassDescription {
            flags: 0,
            pipelineBindPoint: VkPipelineBindPoint_VK_PIPELINE_BIND_POINT_GRAPHICS,
            inputAttachmentCount: 0,
            pInputAttachments: ::std::ptr::null_mut(),
            colorAttachmentCount: 1,
            pColorAttachments: &color_attachment_ref,
            pResolveAttachments: ::std::ptr::null_mut(),
            pDepthStencilAttachment: ::std::ptr::null_mut(),
            preserveAttachmentCount: 0,
            pPreserveAttachments: ::std::ptr::null_mut(),
        };

        let subpass_dependency = VkSubpassDependency { 
            srcSubpass: VK_SUBPASS_EXTERNAL as _,
            dstSubpass: 0,
            srcStageMask: VkPipelineStageFlagBits_VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT as _,
            dstStageMask: VkPipelineStageFlagBits_VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT as _,
            srcAccessMask: 0,
            dstAccessMask: VkAccessFlagBits_VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT as _,
            dependencyFlags: 0,
        };

        let render_pass_create_info = VkRenderPassCreateInfo{
            sType: VkStructureType_VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            attachmentCount: 1,
            pAttachments: &color_attachment,
            subpassCount: 1,
            pSubpasses: &subpass1,
            dependencyCount: 1,
            pDependencies: &subpass_dependency,
        };
        
        self.render_pass = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkCreateRenderPass.unwrap()(self.device, &render_pass_create_info, ::std::ptr::null_mut(), option.as_mut_ptr())
            );
            option.assume_init()
        };
        self
    }

    pub fn create_graphics_pipeline(&mut self) -> &mut Self {
        let mut vert_shader_file = std::fs::File::open("../data/vert.spv").unwrap();
        let mut frag_shader_file = std::fs::File::open("../data/frag.spv").unwrap();
        let vert_shader_code = read_spirv_from_bytes(&mut vert_shader_file);
        let frag_shader_code = read_spirv_from_bytes(&mut frag_shader_file);

        self.create_shader_module(ShaderType::Vertex, vert_shader_code, "main");
        self.create_shader_module(ShaderType::Fragment, frag_shader_code, "main");
        
        let mut shader_stages: Vec<VkPipelineShaderStageCreateInfo> = Vec::new();
        for shader in self.shaders.iter() {
            shader_stages.push(shader.stage_info());
        }

        let vertex_input_info = VkPipelineVertexInputStateCreateInfo{
            sType: VkStructureType_VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            vertexBindingDescriptionCount: 0,
            pVertexBindingDescriptions: ::std::ptr::null_mut(),
            vertexAttributeDescriptionCount: 0,
            pVertexAttributeDescriptions: ::std::ptr::null_mut(),
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
            width: self.swap_chain.details.capabilities.currentExtent.width as f32,
            height: self.swap_chain.details.capabilities.currentExtent.height as f32,
            minDepth: 0.0,
            maxDepth: 1.0,
        };

        let scissors = VkRect2D {
            offset: VkOffset2D {x: 0, y: 0},
            extent: self.swap_chain.details.capabilities.currentExtent,
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
            setLayoutCount: 0,
            pSetLayouts: ::std::ptr::null_mut(),
            pushConstantRangeCount: 0,
            pPushConstantRanges: ::std::ptr::null_mut(),
        };

        self.pipeline_layout = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkCreatePipelineLayout.unwrap()(self.device, &pipeline_layout_info, ::std::ptr::null_mut(), option.as_mut_ptr())
            );
            option.assume_init()
        };

        let pipeline_info = VkGraphicsPipelineCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            stageCount: 2,
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
            renderPass: self.render_pass,
            subpass: 0,
            basePipelineHandle: ::std::ptr::null_mut(),
            basePipelineIndex: -1,
        };
        
        self.graphics_pipeline = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkCreateGraphicsPipelines.unwrap()(self.device, ::std::ptr::null_mut(), 1, &pipeline_info, ::std::ptr::null_mut(), option.as_mut_ptr())
            );
            option.assume_init()
        };

        self
    }

    pub fn create_framebuffers(&mut self) -> &mut Self {
        let mut framebuffers = Vec::<VkFramebuffer>::with_capacity(self.swap_chain.image_views.len());
        unsafe {
            framebuffers.set_len(self.swap_chain.image_views.len());
        }
        
        for (i, imageview) in self.swap_chain.image_views.iter().enumerate() {            
            let framebuffer_create_info = VkFramebufferCreateInfo {
                sType: VkStructureType_VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO,
                pNext: ::std::ptr::null_mut(),
                flags: 0,
                renderPass: self.render_pass,
                attachmentCount: 1,
                pAttachments: imageview,
                width: self.swap_chain.details.capabilities.currentExtent.width,
                height: self.swap_chain.details.capabilities.currentExtent.height,
                layers: 1,
            };

            unsafe {
                assert_eq!(
                    VkResult_VK_SUCCESS,
                    vkCreateFramebuffer.unwrap()(self.device, &framebuffer_create_info, ::std::ptr::null_mut(), &mut framebuffers[i])
                );
            }
        }
        self.framebuffers = framebuffers;
        self
    }

    pub fn create_command_pool(&mut self) -> &mut Self {
        let command_pool_info = VkCommandPoolCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            queueFamilyIndex: self.instance.get_physical_device().get_queue_family_info().graphics_family_index as _,
        };
        
        let command_pool = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkCreateCommandPool.unwrap()(self.device, &command_pool_info, ::std::ptr::null_mut(), option.as_mut_ptr())
            );
            option.assume_init()
        };  
        self.command_pool = command_pool;  
        self
    }

    pub fn create_command_buffers(&mut self) -> &mut Self {        
        let mut command_buffers = Vec::<VkCommandBuffer>::with_capacity(self.framebuffers.len());
        unsafe {
            command_buffers.set_len(self.framebuffers.len());
        }

        let command_alloc_info = VkCommandBufferAllocateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
            pNext: ::std::ptr::null_mut(),
            commandPool: self.command_pool,
            level: VkCommandBufferLevel_VK_COMMAND_BUFFER_LEVEL_PRIMARY,
            commandBufferCount: command_buffers.len() as _,
        };

        unsafe {
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkAllocateCommandBuffers.unwrap()(self.device, &command_alloc_info, command_buffers.as_mut_ptr())
            );
        }

        for (i, command_buffer) in command_buffers.iter().enumerate() {
            let begin_info = VkCommandBufferBeginInfo {
                sType: VkStructureType_VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
                flags: 0,
                pNext: ::std::ptr::null_mut(),
                pInheritanceInfo: ::std::ptr::null_mut(),
            };
                
            unsafe {
                assert_eq!(
                    VkResult_VK_SUCCESS,
                    vkBeginCommandBuffer.unwrap()(*command_buffer, &begin_info)
                );
            }

            let clear_value = VkClearValue{ color: VkClearColorValue { float32: [ 0.0, 0.0, 0.0, 1.0 ] } };
            let render_pass_begin_info = VkRenderPassBeginInfo {
                sType: VkStructureType_VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO,
                pNext: ::std::ptr::null_mut(),
                renderPass: self.render_pass,
                framebuffer: self.framebuffers[i],
                renderArea: VkRect2D { 
                    offset: VkOffset2D {x: 0, y: 0},
                    extent: self.swap_chain.details.capabilities.currentExtent,
                },
                clearValueCount: 1,
                pClearValues: &clear_value,
            };

            let vertex_count = 3;

            unsafe {
                vkCmdBeginRenderPass.unwrap()(*command_buffer, &render_pass_begin_info, VkSubpassContents_VK_SUBPASS_CONTENTS_INLINE);

                vkCmdBindPipeline.unwrap()(*command_buffer, VkPipelineBindPoint_VK_PIPELINE_BIND_POINT_GRAPHICS, self.graphics_pipeline);

                vkCmdDraw.unwrap()(*command_buffer, vertex_count, 1, 0, 0);

                vkCmdEndRenderPass.unwrap()(*command_buffer);
            }
              
            unsafe {
                assert_eq!(
                    VkResult_VK_SUCCESS,
                    vkEndCommandBuffer.unwrap()(*command_buffer)
                );
            }
        }

        self.command_buffers = command_buffers;
        self
    }

    pub fn create_sync_objects(&mut self) -> &mut Self {
        let mut image_available_semaphores: Vec<VkSemaphore> = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT as usize);
        let mut render_finished_semaphores: Vec<VkSemaphore> = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT as usize);
        let mut inflight_fences: Vec<VkFence> = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT as usize); 
        let mut inflight_images: Vec<VkFence> = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT as usize); 
        unsafe {
            image_available_semaphores.set_len(MAX_FRAMES_IN_FLIGHT as usize);
            render_finished_semaphores.set_len(MAX_FRAMES_IN_FLIGHT as usize);
            inflight_fences.set_len(MAX_FRAMES_IN_FLIGHT as usize);
            inflight_images.set_len(MAX_FRAMES_IN_FLIGHT as usize);
        }
        let semaphore_create_info = VkSemaphoreCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
        };

        let fence_create_info = VkFenceCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_FENCE_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: VkFenceCreateFlagBits_VK_FENCE_CREATE_SIGNALED_BIT as _,
        };

        for i in 0..MAX_FRAMES_IN_FLIGHT {
            unsafe {
                assert_eq!(
                    VkResult_VK_SUCCESS,
                    vkCreateSemaphore.unwrap()(self.device, &semaphore_create_info, ::std::ptr::null_mut(), &mut image_available_semaphores[i as usize])
                );    
                assert_eq!(
                    VkResult_VK_SUCCESS,
                    vkCreateSemaphore.unwrap()(self.device, &semaphore_create_info, ::std::ptr::null_mut(), &mut render_finished_semaphores[i as usize])
                ); 
                assert_eq!(
                    VkResult_VK_SUCCESS,
                    vkCreateFence.unwrap()(self.device, &fence_create_info, ::std::ptr::null_mut(), &mut inflight_fences[i as usize])
                );
                inflight_images[i as usize] = ::std::ptr::null_mut();
            }
        }

        self.image_available_semaphores = image_available_semaphores;
        self.render_finished_semaphores = render_finished_semaphores;
        self.inflight_fences = inflight_fences;
        self.inflight_images = inflight_images;
        self
    }

    pub fn destroy_image_views(&self) {        
        for imageview in self.swap_chain.image_views.iter() {
            unsafe {        
                vkDestroyImageView.unwrap()(self.device, *imageview, ::std::ptr::null_mut());
            }
        }
    }

    pub fn create_shader_module<'s>(&mut self, shader_type: ShaderType, shader_content: Vec<u32>, entry_point: &'s str) {
        self.shaders.push( Shader::default().create(self.device, shader_type, shader_content, entry_point) );
    }
    
    pub fn destroy_shader_modules(&self) {
        for shader in self.shaders.iter() {
            shader.destroy(self.device);
        }
    }

    fn cleanup_swap_chain(&self) {             
        unsafe {    
            for framebuffer in self.framebuffers.iter() {
                vkDestroyFramebuffer.unwrap()(self.device, *framebuffer, ::std::ptr::null_mut());
            }

            vkFreeCommandBuffers.unwrap()(self.device, self.command_pool, self.command_buffers.len() as _, self.command_buffers.as_ptr());

            vkDestroyPipeline.unwrap()(self.device, self.graphics_pipeline, ::std::ptr::null_mut());
            vkDestroyPipelineLayout.unwrap()(self.device, self.pipeline_layout, ::std::ptr::null_mut());
            vkDestroyRenderPass.unwrap()(self.device, self.render_pass, ::std::ptr::null_mut());
            
            self.destroy_image_views();

            vkDestroySwapchainKHR.unwrap()(self.device, self.get_swap_chain().into(), ::std::ptr::null_mut());
        }
    }

    fn recreate_swap_chain(&mut self) {
        unsafe {            
            vkDeviceWaitIdle.unwrap()(self.device);
        }
        self.cleanup_swap_chain();

        self.create_swap_chain()
        .create_image_views()
        .create_render_pass()
        .create_graphics_pipeline()
        .create_framebuffers()
        .create_command_buffers();
    }

    pub fn delete(&self) {       
        unsafe {    
            self.cleanup_swap_chain();
            self.destroy_shader_modules();
                    
            for i in 0..MAX_FRAMES_IN_FLIGHT {
                unsafe {        
                    vkDestroySemaphore.unwrap()(self.device, self.render_finished_semaphores[i as usize], ::std::ptr::null_mut());
                    vkDestroySemaphore.unwrap()(self.device, self.image_available_semaphores[i as usize], ::std::ptr::null_mut());
                    vkDestroyFence.unwrap()(self.device, self.inflight_fences[i as usize], ::std::ptr::null_mut());
                }
            }

            vkDestroyCommandPool.unwrap()(self.device, self.command_pool, ::std::ptr::null_mut());
            vkDestroyDevice.unwrap()(self.device, ::std::ptr::null_mut());
        }
    }

    pub fn get_swap_chain(&self) -> &SwapChain {
        &self.swap_chain
    }

    pub fn temp_draw_frame(&mut self) {
        unsafe {
            let mut current_frame_index = 0;

            vkWaitForFences.unwrap()(self.device, 1, &self.inflight_fences[current_frame_index as usize], VK_TRUE, std::u64::MAX);
                
            let mut image_index: u32 = 0;
            let mut result = vkAcquireNextImageKHR.unwrap()(self.device, self.swap_chain.ptr, ::std::u64::MAX, self.image_available_semaphores[current_frame_index], ::std::ptr::null_mut(), &mut image_index);
            
            if result == VkResult_VK_ERROR_OUT_OF_DATE_KHR {
                self.recreate_swap_chain();
                return;
            } 
            else if result != VkResult_VK_SUCCESS && result != VkResult_VK_SUBOPTIMAL_KHR {
                eprintln!("Failed to acquire swap chain image");
            }

            if self.inflight_images[image_index as usize] != ::std::ptr::null_mut() {
                vkWaitForFences.unwrap()(self.device, 1, &self.inflight_images[image_index as usize], VK_TRUE, std::u64::MAX);
            }
            self.inflight_images[image_index as usize] = self.inflight_fences[current_frame_index as usize];
            
            
            let wait_stages = [VkPipelineStageFlagBits_VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT];
            let submit_info = VkSubmitInfo {
                sType: VkStructureType_VK_STRUCTURE_TYPE_SUBMIT_INFO,
                pNext: ::std::ptr::null_mut(),
                waitSemaphoreCount: 1,
                pWaitSemaphores: &self.image_available_semaphores[current_frame_index as usize],
                pWaitDstStageMask: wait_stages.as_ptr() as *const _,
                commandBufferCount: 1,
                pCommandBuffers: self.command_buffers.as_mut_ptr(),
                signalSemaphoreCount: 1,
                pSignalSemaphores: &self.render_finished_semaphores[current_frame_index as usize],
            };
            
            vkResetFences.unwrap()(self.device, 1, &self.inflight_fences[current_frame_index as usize]);
            
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkQueueSubmit.unwrap()(self.graphics_queue, 1, &submit_info, self.inflight_fences[current_frame_index as usize])
            );
                
            let present_info = VkPresentInfoKHR {
                sType: VkStructureType_VK_STRUCTURE_TYPE_PRESENT_INFO_KHR,
                pNext: ::std::ptr::null_mut(),
                waitSemaphoreCount: 1,
                pWaitSemaphores: &self.render_finished_semaphores[current_frame_index as usize],
                swapchainCount: 1,
                pSwapchains: &self.swap_chain.ptr,
                pImageIndices: &image_index,
                pResults: ::std::ptr::null_mut(),
            };

            result = vkQueuePresentKHR.unwrap()(self.graphics_queue, &present_info);
            
            if result == VkResult_VK_ERROR_OUT_OF_DATE_KHR || result == VkResult_VK_SUBOPTIMAL_KHR {
                self.recreate_swap_chain();
            } 
            else if result != VkResult_VK_SUCCESS {
                eprintln!("Failed to present swap chain image!");
            }
            
            vkQueueWaitIdle.unwrap()(self.graphics_queue);

            current_frame_index = (current_frame_index + 1) % MAX_FRAMES_IN_FLIGHT as usize;
        }
    }
}