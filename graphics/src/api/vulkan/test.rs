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

        let mut surface:VkSurfaceKHR = ::std::ptr::null_mut(); 
        surface = test_vulkan_create_win32_display_surface(&mut instance);
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

            let mut vertex_shader = create_shader_module(&mut device, test_shader_vert());
            let mut fragment_shader = create_shader_module(&mut device, test_shader_frag());
            
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

            let render_pass_info = VkRenderPassCreateInfo{
                sType: VkStructureType_VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO,
                pNext: ::std::ptr::null_mut(),
                flags: 0,
                attachmentCount: 1,
                pAttachments: &color_attachment,
                subpassCount: 1,
                pSubpasses: &subpass1,
                dependencyCount: 0,
                pDependencies: ::std::ptr::null_mut(),
            };
            
            let render_pass = unsafe {
                let mut option = ::std::mem::MaybeUninit::uninit();
                assert_eq!(
                    VkResult_VK_SUCCESS,
                    vkCreateRenderPass.unwrap()(device, &render_pass_info, ::std::ptr::null_mut(), option.as_mut_ptr())
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


pub fn test_shader_vert<'a>() -> &'a str {
    let shader_code = r#"	
    0302 2307 0000 0100 0a00 0d00 3a00 0000
    0000 0000 1100 0200 0100 0000 0b00 0600
    0100 0000 474c 534c 2e73 7464 2e34 3530
    0000 0000 0e00 0300 0000 0000 0100 0000
    0f00 0800 0000 0000 0400 0000 6d61 696e
    0000 0000 2200 0000 2600 0000 3100 0000
    4800 0500 2000 0000 0000 0000 0b00 0000
    0000 0000 4800 0500 2000 0000 0100 0000
    0b00 0000 0100 0000 4800 0500 2000 0000
    0200 0000 0b00 0000 0300 0000 4800 0500
    2000 0000 0300 0000 0b00 0000 0400 0000
    4700 0300 2000 0000 0200 0000 4700 0400
    2600 0000 0b00 0000 2a00 0000 4700 0400
    3100 0000 1e00 0000 0000 0000 1300 0200
    0200 0000 2100 0300 0300 0000 0200 0000
    1600 0300 0600 0000 2000 0000 1700 0400
    0700 0000 0600 0000 0200 0000 1500 0400
    0800 0000 2000 0000 0000 0000 2b00 0400
    0800 0000 0900 0000 0300 0000 1c00 0400
    0a00 0000 0700 0000 0900 0000 2b00 0400
    0600 0000 0d00 0000 0000 0000 2b00 0400
    0600 0000 0e00 0000 0000 00bf 2c00 0500
    0700 0000 0f00 0000 0d00 0000 0e00 0000
    2b00 0400 0600 0000 1000 0000 0000 003f
    2c00 0500 0700 0000 1100 0000 1000 0000
    1000 0000 2c00 0500 0700 0000 1200 0000
    0e00 0000 1000 0000 2c00 0600 0a00 0000
    1300 0000 0f00 0000 1100 0000 1200 0000
    1700 0400 1400 0000 0600 0000 0300 0000
    1c00 0400 1500 0000 1400 0000 0900 0000
    2b00 0400 0600 0000 1800 0000 0000 803f
    2c00 0600 1400 0000 1900 0000 1800 0000
    0d00 0000 0d00 0000 2c00 0600 1400 0000
    1a00 0000 0d00 0000 1800 0000 0d00 0000
    2c00 0600 1400 0000 1b00 0000 0d00 0000
    0d00 0000 1800 0000 2c00 0600 1500 0000
    1c00 0000 1900 0000 1a00 0000 1b00 0000
    1700 0400 1d00 0000 0600 0000 0400 0000
    2b00 0400 0800 0000 1e00 0000 0100 0000
    1c00 0400 1f00 0000 0600 0000 1e00 0000
    1e00 0600 2000 0000 1d00 0000 0600 0000
    1f00 0000 1f00 0000 2000 0400 2100 0000
    0300 0000 2000 0000 3b00 0400 2100 0000
    2200 0000 0300 0000 1500 0400 2300 0000
    2000 0000 0100 0000 2b00 0400 2300 0000
    2400 0000 0000 0000 2000 0400 2500 0000
    0100 0000 2300 0000 3b00 0400 2500 0000
    2600 0000 0100 0000 2000 0400 2e00 0000
    0300 0000 1d00 0000 2000 0400 3000 0000
    0300 0000 1400 0000 3b00 0400 3000 0000
    3100 0000 0300 0000 2000 0400 3600 0000
    0700 0000 0a00 0000 2000 0400 3700 0000
    0700 0000 0700 0000 2000 0400 3800 0000
    0700 0000 1500 0000 2000 0400 3900 0000
    0700 0000 1400 0000 3600 0500 0200 0000
    0400 0000 0000 0000 0300 0000 f800 0200
    0500 0000 3b00 0400 3800 0000 1700 0000
    0700 0000 3b00 0400 3600 0000 0c00 0000
    0700 0000 3e00 0300 0c00 0000 1300 0000
    3e00 0300 1700 0000 1c00 0000 3d00 0400
    2300 0000 2700 0000 2600 0000 4100 0500
    3700 0000 2900 0000 0c00 0000 2700 0000
    3d00 0400 0700 0000 2a00 0000 2900 0000
    5100 0500 0600 0000 2b00 0000 2a00 0000
    0000 0000 5100 0500 0600 0000 2c00 0000
    2a00 0000 0100 0000 5000 0700 1d00 0000
    2d00 0000 2b00 0000 2c00 0000 0d00 0000
    1800 0000 4100 0500 2e00 0000 2f00 0000
    2200 0000 2400 0000 3e00 0300 2f00 0000
    2d00 0000 4100 0500 3900 0000 3400 0000
    1700 0000 2700 0000 3d00 0400 1400 0000
    3500 0000 3400 0000 3e00 0300 3100 0000
    3500 0000 fd00 0100 3800 0100 
    "#;
    shader_code
}

pub fn test_shader_frag<'a>() -> &'a str {
    let shader_code = r#"
    0302 2307 0000 0100 0a00 0d00 1300 0000
    0000 0000 1100 0200 0100 0000 0b00 0600
    0100 0000 474c 534c 2e73 7464 2e34 3530
    0000 0000 0e00 0300 0000 0000 0100 0000
    0f00 0700 0400 0000 0400 0000 6d61 696e
    0000 0000 0900 0000 0c00 0000 1000 0300
    0400 0000 0700 0000 4700 0400 0900 0000
    1e00 0000 0000 0000 4700 0400 0c00 0000
    1e00 0000 0000 0000 1300 0200 0200 0000
    2100 0300 0300 0000 0200 0000 1600 0300
    0600 0000 2000 0000 1700 0400 0700 0000
    0600 0000 0400 0000 2000 0400 0800 0000
    0300 0000 0700 0000 3b00 0400 0800 0000
    0900 0000 0300 0000 1700 0400 0a00 0000
    0600 0000 0300 0000 2000 0400 0b00 0000
    0100 0000 0a00 0000 3b00 0400 0b00 0000
    0c00 0000 0100 0000 2b00 0400 0600 0000
    0e00 0000 0000 803f 3600 0500 0200 0000
    0400 0000 0000 0000 0300 0000 f800 0200
    0500 0000 3d00 0400 0a00 0000 0d00 0000
    0c00 0000 5100 0500 0600 0000 0f00 0000
    0d00 0000 0000 0000 5100 0500 0600 0000
    1000 0000 0d00 0000 0100 0000 5100 0500
    0600 0000 1100 0000 0d00 0000 0200 0000
    5000 0700 0700 0000 1200 0000 0f00 0000
    1000 0000 1100 0000 0e00 0000 3e00 0300
    0900 0000 1200 0000 fd00 0100 3800 0100
    "#;
    shader_code
}