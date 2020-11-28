 #![allow(dead_code)]

use vulkan_bindings::*;
use nrg_platform::*;
use super::utils::*;

#[test]
fn test_vulkan()
{
    use super::types::*;
    
    VK::initialize(&vulkan_bindings::Lib::new());
    
    let app_info = VkApplicationInfo {
        sType: VkStructureType_VK_STRUCTURE_TYPE_APPLICATION_INFO,
        pNext: ::std::ptr::null_mut(),
        pApplicationName: ::std::ffi::CString::new("NRG").unwrap().as_ptr(),
        applicationVersion: VK_API_VERSION_1_1,
        pEngineName: ::std::ffi::CString::new("NRGEngine").unwrap().as_ptr(),
        engineVersion: VK_API_VERSION_1_1,
        apiVersion: VK_API_VERSION_1_1,
    };

    let mut create_info = VkInstanceCreateInfo {
        sType: VkStructureType_VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
        pNext: ::std::ptr::null_mut(),
        flags: 0,
        pApplicationInfo: &app_info,
        enabledLayerCount: 0,
        ppEnabledLayerNames: ::std::ptr::null_mut(),
        enabledExtensionCount: 0,
        ppEnabledExtensionNames: ::std::ptr::null_mut(), 
    };

    //Layers

    let mut layers_count = 0;
    unsafe {
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkEnumerateInstanceLayerProperties.unwrap()(&mut layers_count, ::std::ptr::null_mut())
        );
    }
    assert_ne!(layers_count, 0);    

    let mut available_layers: Vec<VkLayerProperties> = Vec::with_capacity(layers_count as usize);
    unsafe {
        available_layers.set_len(layers_count as usize);
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkEnumerateInstanceLayerProperties.unwrap()(&mut layers_count, available_layers.as_mut_ptr())
        );
    }    
    assert_ne!(available_layers.len(), 0);
    assert_eq!(available_layers.len(), layers_count as usize);
    
    let layer_names_str = available_layers.iter()
                                        .map(|layer| unsafe {::std::ffi::CStr::from_ptr(layer.layerName.as_ptr())}.to_owned())
                                        .collect::<Vec<::std::ffi::CString>>();
    let layer_names_ptr = layer_names_str.iter()
                                            .map(|e| e.as_ptr())
                                            .collect::<Vec<*const i8>>();

    assert_eq!(layer_names_str.len(), available_layers.len());
    
    create_info.enabledLayerCount = layer_names_ptr.len() as u32;
    create_info.ppEnabledLayerNames = layer_names_ptr.as_ptr();

    //Extensions
    
    let mut extension_count = 0;
    unsafe {
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkEnumerateInstanceExtensionProperties.unwrap()(::std::ptr::null_mut(), &mut extension_count, ::std::ptr::null_mut())
        );
    }
    assert_ne!(extension_count, 0);

    let mut supported_extensions: Vec<VkExtensionProperties> = Vec::with_capacity(extension_count as usize);
    unsafe {
        supported_extensions.set_len(extension_count as usize);
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkEnumerateInstanceExtensionProperties.unwrap()(::std::ptr::null_mut(), &mut extension_count, supported_extensions.as_mut_ptr())
        );
    }    
    assert_ne!(supported_extensions.len(), 0);
    assert_eq!(supported_extensions.len(), extension_count as usize);

    let extension_names_str = supported_extensions.iter()
                                            .map(|ext| unsafe {::std::ffi::CStr::from_ptr(ext.extensionName.as_ptr())}.to_owned())
                                            .collect::<Vec<::std::ffi::CString>>();
    let extension_names_ptr = extension_names_str.iter()
                                            .map(|e| e.as_ptr())
                                            .collect::<Vec<*const i8>>();

    assert_eq!(extension_names_str.len(), supported_extensions.len());

    create_info.enabledExtensionCount = extension_names_ptr.len() as u32;
    create_info.ppEnabledExtensionNames = extension_names_ptr.as_ptr();

    //Create Instance
   
    let mut instance:VkInstance = ::std::ptr::null_mut();
    unsafe {        
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkCreateInstance.unwrap()(&create_info, ::std::ptr::null_mut(), &mut instance)
        );
    }

    //Physical Device

    let mut physical_device_count = 0;
    unsafe {
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkEnumeratePhysicalDevices.unwrap()(instance, &mut physical_device_count, ::std::ptr::null_mut())
        );
    }
    assert_ne!(physical_device_count, 0);
    
    let mut physical_devices: Vec<VkPhysicalDevice> = Vec::with_capacity(physical_device_count as usize);
    unsafe {
        physical_devices.set_len(physical_device_count as usize);
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkEnumeratePhysicalDevices.unwrap()(instance, &mut physical_device_count, physical_devices.as_mut_ptr())
        );
    }    
    assert_ne!(physical_devices.len(), 0);
    assert_eq!(physical_devices.len(), physical_device_count as usize);

    for physical_device in physical_devices.into_iter() {

        let surface:VkSurfaceKHR = test_vulkan_create_win32_display_surface(&mut instance);
        //surface = test_vulkan_create_khr_display_surface(&lib, &mut physical_device, &mut instance);

        assert_eq!(is_device_suitable(&physical_device, &surface), true);

        let queue_family_indices = find_queue_family_indices(&physical_device, &surface);
        let swap_chain_details = find_swap_chain_support(&physical_device, &surface);

        let queue_priority:f32 = 1.0;
        let queue_create_info = VkDeviceQueueCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            queueFamilyIndex: queue_family_indices.graphics_family_index as u32,
            queueCount: 1,
            pQueuePriorities: &queue_priority,
        };

        let device_features: VkPhysicalDeviceFeatures = unsafe {
            let mut output = ::std::mem::MaybeUninit::uninit();
            vkGetPhysicalDeviceFeatures.unwrap()(physical_device, output.as_mut_ptr());
            output.assume_init()
        };
    
        let supported_device_extensions = get_device_extensions(&physical_device);
        assert_ne!(supported_device_extensions.len(), 0);

        let device_extension_names_str = supported_device_extensions.iter()
                                                        .map(|ext| unsafe {::std::ffi::CStr::from_ptr(ext.extensionName.as_ptr())}.to_owned())
                                                        .collect::<Vec<::std::ffi::CString>>();
        let device_extension_names_ptr = device_extension_names_str.iter()
                                                        .map(|e| e.as_ptr())
                                                        .collect::<Vec<*const i8>>();

        assert_eq!(device_extension_names_str.len(), device_extension_names_ptr.len());
        assert_ne!(device_extension_names_ptr.len(), 0);


        let device_create_info = VkDeviceCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            queueCreateInfoCount: 1,
            pQueueCreateInfos: &queue_create_info,
            enabledLayerCount: layers_count,
            ppEnabledLayerNames: layer_names_ptr.as_ptr(),
            enabledExtensionCount: device_extension_names_ptr.len() as u32,
            ppEnabledExtensionNames: device_extension_names_ptr.as_ptr(),
            pEnabledFeatures: &device_features,
        };

        let mut device: VkDevice = ::std::ptr::null_mut();
        unsafe {
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkCreateDevice.unwrap()(physical_device, &device_create_info, ::std::ptr::null_mut(), &mut device)
            );
        }
        assert_ne!(device, ::std::ptr::null_mut());

        let graphics_queue: VkQueue = unsafe {
            let mut output = ::std::mem::MaybeUninit::uninit();
            vkGetDeviceQueue.unwrap()(device, queue_family_indices.graphics_family_index as u32, 0, output.as_mut_ptr());
            output.assume_init()
        };
        assert_ne!(graphics_queue, ::std::ptr::null_mut());

        let swap_chain_create_info = VkSwapchainCreateInfoKHR{
            sType: VkStructureType_VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            surface: surface,
            minImageCount: ::std::cmp::max(swap_chain_details.capabilities.minImageCount + 1, swap_chain_details.capabilities.maxImageCount),
            imageFormat: swap_chain_details.formats[0].format,
            imageColorSpace: swap_chain_details.formats[0].colorSpace,
            imageExtent: swap_chain_details.capabilities.currentExtent,
            imageArrayLayers: 1,
            imageUsage: VkImageUsageFlagBits_VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT as u32,
            imageSharingMode: VkSharingMode_VK_SHARING_MODE_EXCLUSIVE,
            queueFamilyIndexCount: 0,
            pQueueFamilyIndices: ::std::ptr::null_mut(),
            preTransform: swap_chain_details.capabilities.currentTransform,
            compositeAlpha: VkCompositeAlphaFlagBitsKHR_VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR,
            presentMode: swap_chain_details.present_modes[0],
            clipped: VK_TRUE,
            oldSwapchain: ::std::ptr::null_mut(),
        };

        let swap_chain = unsafe {
            let mut output = ::std::mem::MaybeUninit::uninit();
            vkCreateSwapchainKHR.unwrap()(device, &swap_chain_create_info, ::std::ptr::null_mut(), output.as_mut_ptr());
            output.assume_init()
        };
        assert_ne!(swap_chain, ::std::ptr::null_mut());

        let mut swapchain_images_count = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            vkGetSwapchainImagesKHR.unwrap()(device, swap_chain, option.as_mut_ptr(), ::std::ptr::null_mut());
            option.assume_init()
        };
        assert_ne!(swapchain_images_count, 0);
            
        let mut swapchain_images: Vec<VkImage> = Vec::with_capacity(swapchain_images_count as usize);
        unsafe {
            swapchain_images.set_len(swapchain_images_count as usize);
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkGetSwapchainImagesKHR.unwrap()(device, swap_chain, &mut swapchain_images_count, swapchain_images.as_mut_ptr())
            );
        }     
        assert_eq!(swapchain_images.len(), swapchain_images_count as usize);
        assert_ne!(swapchain_images.len(), 0);
        
        let mut swapchain_image_views: Vec<VkImageView> = Vec::with_capacity(swapchain_images_count as usize);
        unsafe {
            swapchain_image_views.set_len(swapchain_images_count as usize);
        }

        let selected_format = swap_chain_details.formats[0].format;

        for (i, image) in swapchain_images.iter().enumerate() {
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
                    vkCreateImageView.unwrap()(device, &mut image_view_create_info, ::std::ptr::null_mut(), &mut swapchain_image_views[i])
                );
            }
        }

        let mut vert_shader_file = std::fs::File::open("../data/vert.spv").unwrap();
        let mut frag_shader_file = std::fs::File::open("../data/frag.spv").unwrap();
        let vert_shader_code = read_spirv_from_bytes(&mut vert_shader_file);
        let frag_shader_code = read_spirv_from_bytes(&mut frag_shader_file);
        let mut vertex_shader = create_shader_module(&mut device, &vert_shader_code);
        let mut fragment_shader = create_shader_module(&mut device, &frag_shader_code);
        
        let vertex_shader_stage_info = VkPipelineShaderStageCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            stage: VkShaderStageFlagBits_VK_SHADER_STAGE_VERTEX_BIT,
            module: vertex_shader,
            pName: "main".as_bytes().as_ptr() as *const _,
            pSpecializationInfo: ::std::ptr::null_mut(),
        };
        
        let frag_shader_stage_info = VkPipelineShaderStageCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            stage: VkShaderStageFlagBits_VK_SHADER_STAGE_FRAGMENT_BIT,
            module: fragment_shader,
            pName: "main".as_bytes().as_ptr() as *const _,
            pSpecializationInfo: ::std::ptr::null_mut(),
        };

        let mut shader_stages: Vec<VkPipelineShaderStageCreateInfo> = Vec::new();
        shader_stages.push(vertex_shader_stage_info);
        shader_stages.push(frag_shader_stage_info);

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
            width: swap_chain_details.capabilities.currentExtent.width as f32,
            height: swap_chain_details.capabilities.currentExtent.height as f32,
            minDepth: 0.0,
            maxDepth: 1.0,
        };

        let scissors = VkRect2D {
            offset: VkOffset2D {x: 0, y: 0},
            extent: swap_chain_details.capabilities.currentExtent,
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

        let pipeline_layout = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkCreatePipelineLayout.unwrap()(device, &pipeline_layout_info, ::std::ptr::null_mut(), option.as_mut_ptr())
            );
            option.assume_init()
        };

        let color_attachment = VkAttachmentDescription {
            flags: 0,
            format: swap_chain_details.formats[0].format,
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
        
        let render_pass = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkCreateRenderPass.unwrap()(device, &render_pass_create_info, ::std::ptr::null_mut(), option.as_mut_ptr())
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
            layout: pipeline_layout,
            renderPass: render_pass,
            subpass: 0,
            basePipelineHandle: ::std::ptr::null_mut(),
            basePipelineIndex: -1,
        };
        
        let graphics_pipeline = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkCreateGraphicsPipelines.unwrap()(device, ::std::ptr::null_mut(), 1, &pipeline_info, ::std::ptr::null_mut(), option.as_mut_ptr())
            );
            option.assume_init()
        };
        

        let mut framebuffers = Vec::<VkFramebuffer>::with_capacity(swapchain_image_views.len());
        unsafe {
            framebuffers.set_len(swapchain_images_count as usize);
        }
        
        for (i, imageview) in swapchain_image_views.iter().enumerate() {
            
            let framebuffer_create_info = VkFramebufferCreateInfo {
                sType: VkStructureType_VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO,
                pNext: ::std::ptr::null_mut(),
                flags: 0,
                renderPass: render_pass,
                attachmentCount: 1,
                pAttachments: imageview,
                width: swap_chain_details.capabilities.currentExtent.width,
                height: swap_chain_details.capabilities.currentExtent.height,
                layers: 1,
            };

            unsafe {
                assert_eq!(
                    VkResult_VK_SUCCESS,
                    vkCreateFramebuffer.unwrap()(device, &framebuffer_create_info, ::std::ptr::null_mut(), &mut framebuffers[i])
                );
            }
        }

        let command_pool_info = VkCommandPoolCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            queueFamilyIndex: queue_family_indices.graphics_family_index as u32,
        };
        
        let mut command_pool = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkCreateCommandPool.unwrap()(device, &command_pool_info, ::std::ptr::null_mut(), option.as_mut_ptr())
            );
            option.assume_init()
        };        

        let mut command_buffers = Vec::<VkCommandBuffer>::with_capacity(framebuffers.len());
        unsafe {
            command_buffers.set_len(framebuffers.len());
        }

        let command_alloc_info = VkCommandBufferAllocateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
            pNext: ::std::ptr::null_mut(),
            commandPool: command_pool,
            level: VkCommandBufferLevel_VK_COMMAND_BUFFER_LEVEL_PRIMARY,
            commandBufferCount: command_buffers.len() as u32,
        };

        unsafe {
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkAllocateCommandBuffers.unwrap()(device, &command_alloc_info, command_buffers.as_mut_ptr())
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
                renderPass: render_pass,
                framebuffer: framebuffers[i],
                renderArea: VkRect2D { 
                    offset: VkOffset2D {x: 0, y: 0},
                    extent: swap_chain_details.capabilities.currentExtent,
                },
                clearValueCount: 1,
                pClearValues: &clear_value,
            };

            let vertex_count = 3;

            unsafe {
                vkCmdBeginRenderPass.unwrap()(*command_buffer, &render_pass_begin_info, VkSubpassContents_VK_SUBPASS_CONTENTS_INLINE);

                vkCmdBindPipeline.unwrap()(*command_buffer, VkPipelineBindPoint_VK_PIPELINE_BIND_POINT_GRAPHICS, graphics_pipeline);

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

        const MAX_FRAMES_IN_FLIGHT:u32 = 2;

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

        for i in 0..MAX_FRAMES_IN_FLIGHT {
            unsafe {
                assert_eq!(
                    VkResult_VK_SUCCESS,
                    vkCreateSemaphore.unwrap()(device, &semaphore_create_info, ::std::ptr::null_mut(), &mut image_available_semaphores[i as usize])
                );    
                assert_eq!(
                    VkResult_VK_SUCCESS,
                    vkCreateSemaphore.unwrap()(device, &semaphore_create_info, ::std::ptr::null_mut(), &mut render_finished_semaphores[i as usize])
                ); 
                assert_eq!(
                    VkResult_VK_SUCCESS,
                    vkCreateFence.unwrap()(device, &fence_create_info, ::std::ptr::null_mut(), &mut inflight_fences[i as usize])
                );
                inflight_images[i as usize] = ::std::ptr::null_mut();
            }
        }

        let mut current_frame_index = 0;

        
        let image_index = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkAcquireNextImageKHR.unwrap()(device, swap_chain, ::std::u64::MAX, image_available_semaphores[current_frame_index], ::std::ptr::null_mut(), option.as_mut_ptr())
            );
            option.assume_init()
        };        

        let wait_stages = [VkPipelineStageFlagBits_VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT];
        let submit_info = VkSubmitInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_SUBMIT_INFO,
            pNext: ::std::ptr::null_mut(),
            waitSemaphoreCount: 1,
            pWaitSemaphores: &image_available_semaphores[current_frame_index as usize],
            pWaitDstStageMask: wait_stages.as_ptr() as *const _,
            commandBufferCount: 1,
            pCommandBuffers: command_buffers.as_mut_ptr(),
            signalSemaphoreCount: 1,
            pSignalSemaphores: &render_finished_semaphores[current_frame_index as usize],
        };

        let present_info = VkPresentInfoKHR {
            sType: VkStructureType_VK_STRUCTURE_TYPE_PRESENT_INFO_KHR,
            pNext: ::std::ptr::null_mut(),
            waitSemaphoreCount: 1,
            pWaitSemaphores: &render_finished_semaphores[current_frame_index as usize],
            swapchainCount: 1,
            pSwapchains: &swap_chain,
            pImageIndices: &image_index,
            pResults: ::std::ptr::null_mut(),
        };

        unsafe {
            // Check if a previous frame is using this image (i.e. there is its fence to wait on)
            if inflight_images[image_index as usize] != ::std::ptr::null_mut() {
                vkWaitForFences.unwrap()(device, 1, &inflight_images[image_index as usize], VK_TRUE, std::u64::MAX);
            }
            // Mark the image as now being in use by this frame
            inflight_images[image_index as usize] = inflight_fences[current_frame_index as usize];

            vkResetFences.unwrap()(device, 1, &inflight_fences[current_frame_index as usize]);
            
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkQueueSubmit.unwrap()(graphics_queue, 1, &submit_info, inflight_fences[current_frame_index as usize])
            );

            vkQueuePresentKHR.unwrap()(graphics_queue, &present_info);
            vkQueueWaitIdle.unwrap()(graphics_queue);

            current_frame_index = (current_frame_index + 1) % MAX_FRAMES_IN_FLIGHT as usize;
        }


        //wait device and start destroy

        unsafe {
            vkDeviceWaitIdle.unwrap()(device);
        }
        
        for i in 0..MAX_FRAMES_IN_FLIGHT {
            unsafe {        
                vkDestroySemaphore.unwrap()(device, render_finished_semaphores[i as usize], ::std::ptr::null_mut());
                vkDestroySemaphore.unwrap()(device, image_available_semaphores[i as usize], ::std::ptr::null_mut());
                vkDestroyFence.unwrap()(device, inflight_fences[i as usize], ::std::ptr::null_mut());
            }
        }

        unsafe {        
            vkDestroyCommandPool.unwrap()(device, command_pool, ::std::ptr::null_mut());
        }

        for framebuffer in framebuffers.iter() {
            unsafe {        
                vkDestroyFramebuffer.unwrap()(device, *framebuffer, ::std::ptr::null_mut());
            }
        }

        destroy_shader_module(&mut device, &mut fragment_shader);
        destroy_shader_module(&mut device, &mut vertex_shader);  

        unsafe {        
            vkDestroyPipeline.unwrap()(device, graphics_pipeline, ::std::ptr::null_mut());
        }

        unsafe {        
            vkDestroyPipelineLayout.unwrap()(device, pipeline_layout, ::std::ptr::null_mut());
        }

        unsafe {        
            vkDestroyRenderPass.unwrap()(device, render_pass, ::std::ptr::null_mut());
        }
        
        for imageview in swapchain_image_views.iter() {
            unsafe {        
                vkDestroyImageView.unwrap()(device, *imageview, ::std::ptr::null_mut());
            }
        }

        unsafe {        
            vkDestroySwapchainKHR.unwrap()(device, swap_chain, ::std::ptr::null_mut());
        }

        unsafe {        
            vkDestroySurfaceKHR.unwrap()(instance, surface, ::std::ptr::null_mut());
        }

        unsafe {        
            vkDestroyDevice.unwrap()(device, ::std::ptr::null_mut());
        }
    }    

    //Destroy Instance

    unsafe {        
        vkDestroyInstance.unwrap()(instance, ::std::ptr::null_mut());
    }
}

#[allow(non_snake_case)]
fn test_vulkan_create_win32_display_surface(instance:&mut VkInstance) -> VkSurfaceKHR
{
    let window =  Window::create( String::from("Test Window"),
                    String::from("Test Window"),
                    100, 100,
                    1024, 768 );

    let surface_create_info = VkWin32SurfaceCreateInfoKHR {
        sType: VkStructureType_VK_STRUCTURE_TYPE_WIN32_SURFACE_CREATE_INFO_KHR,
        pNext: ::std::ptr::null_mut(),
        flags: 0,
        hinstance: unsafe {::std::mem::transmute(window.handle.handle_impl.hinstance)},
        hwnd: unsafe {::std::mem::transmute(window.handle.handle_impl.hwnd)},
    };
    
    let surface:VkSurfaceKHR = unsafe {
        let mut output = ::std::mem::MaybeUninit::uninit();
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkCreateWin32SurfaceKHR.unwrap()(*instance, &surface_create_info, ::std::ptr::null_mut(), output.as_mut_ptr())
        );
        output.assume_init()
    };
    
    surface
}


fn test_vulkan_create_khr_display_surface(physical_device:&mut VkPhysicalDevice, instance:&mut VkInstance) -> VkSurfaceKHR
{
    let mut display_count:u32 = 0;
    unsafe {
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkGetPhysicalDeviceDisplayPropertiesKHR.unwrap()(*physical_device, &mut display_count, ::std::ptr::null_mut())
        );
    }
    assert_ne!(display_count, 0);

    let mut display_properties: Vec<VkDisplayPropertiesKHR> = Vec::with_capacity(display_count as usize);
    unsafe {
        display_properties.set_len(display_count as usize);
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkGetPhysicalDeviceDisplayPropertiesKHR.unwrap()(*physical_device, &mut display_count, display_properties.as_mut_ptr())
        );
    }  
    assert_ne!(display_properties.len(), 0);
    assert_eq!(display_properties.len(), display_count as usize);

    let display_selected = 0;
    let mut mode_count = 0;
    unsafe {
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkGetDisplayModePropertiesKHR.unwrap()(*physical_device, display_properties[display_selected].display, &mut mode_count, ::std::ptr::null_mut())
        );
    }
    assert_ne!(mode_count, 0);
    
    let mut display_modes: Vec<VkDisplayModePropertiesKHR> = Vec::with_capacity(mode_count as usize);
    unsafe {
        display_modes.set_len(mode_count as usize);
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkGetDisplayModePropertiesKHR.unwrap()(*physical_device, display_properties[display_selected].display, &mut mode_count, display_modes.as_mut_ptr())
        );
    }  
    assert_ne!(display_modes.len(), 0);
    assert_eq!(display_modes.len(), mode_count as usize);
    
    let mode_selected = 0;
    let mut plane_count = 0;
    unsafe {
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkGetPhysicalDeviceDisplayPlanePropertiesKHR.unwrap()(*physical_device, &mut plane_count, ::std::ptr::null_mut())
        );
    }
    assert_ne!(plane_count, 0);
            
    let mut display_planes: Vec<VkDisplayPlanePropertiesKHR> = Vec::with_capacity(plane_count as usize);
    unsafe {
        display_planes.set_len(plane_count as usize);
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkGetPhysicalDeviceDisplayPlanePropertiesKHR.unwrap()(*physical_device, &mut plane_count, display_planes.as_mut_ptr())
        );
    }  
    assert_ne!(display_planes.len(), 0);
    assert_eq!(display_planes.len(), plane_count as usize);

    let plane_selected = find_plane_for_display(physical_device, &display_properties[display_selected].display, &display_planes);
    assert_ne!(plane_selected, -1);

    let display_plane_capabilities = unsafe {
        let mut output = ::std::mem::MaybeUninit::uninit();
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkGetDisplayPlaneCapabilitiesKHR.unwrap()(*physical_device, display_modes[mode_selected].displayMode, plane_selected as u32, output.as_mut_ptr())
        );
        output.assume_init()
    };        
    
    let mut surface:VkSurfaceKHR = ::std::ptr::null_mut();

    let surface_info = VkDisplaySurfaceCreateInfoKHR {
        sType: VkStructureType_VK_STRUCTURE_TYPE_DISPLAY_SURFACE_CREATE_INFO_KHR,
        pNext: ::std::ptr::null_mut(),
        flags: 0,
        displayMode: display_modes[mode_selected].displayMode,
        planeIndex: plane_selected as u32,
        planeStackIndex: display_planes[plane_selected as usize].currentStackIndex,
        transform: VkSurfaceTransformFlagBitsKHR_VK_SURFACE_TRANSFORM_IDENTITY_BIT_KHR,
        globalAlpha: 1.0,
        alphaMode: get_supported_alpha_mode(&display_plane_capabilities),
        imageExtent: VkExtent2D { 
            width: display_modes[mode_selected].parameters.visibleRegion.width,
            height: display_modes[mode_selected].parameters.visibleRegion.height,
        },
    };

    unsafe {
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkCreateDisplayPlaneSurfaceKHR.unwrap()(*instance, &surface_info, ::std::ptr::null(), &mut surface)
        );
    }  

    surface
}

