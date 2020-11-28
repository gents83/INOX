#![allow(dead_code)]

use vulkan_bindings::*;

pub struct QueueFamilyIndices {
    pub graphics_family_index: i32,
    pub present_family_index: i32,
}
pub struct SwapChainSupportDetails {
    pub capabilities: VkSurfaceCapabilitiesKHR,
    pub formats: Vec<VkSurfaceFormatKHR>,
    pub present_modes: Vec<VkPresentModeKHR>,
}

pub fn find_queue_family_indices(device: &VkPhysicalDevice, surface:&VkSurfaceKHR) -> QueueFamilyIndices {
 
    let mut queue_family_count: u32 = unsafe {
        let mut output = ::std::mem::MaybeUninit::uninit();
        vkGetPhysicalDeviceQueueFamilyProperties.unwrap()(*device, output.as_mut_ptr(), ::std::ptr::null_mut());
        output.assume_init()
    };
    
    let mut queue_family_properties: Vec<VkQueueFamilyProperties> = Vec::with_capacity(queue_family_count as usize);
    unsafe {
        queue_family_properties.set_len(queue_family_count as usize);
        vkGetPhysicalDeviceQueueFamilyProperties.unwrap()(*device, &mut queue_family_count, queue_family_properties.as_mut_ptr());
    }    

    let mut graphic_index = -1;
    let mut present_index = -1;
    
    for (index, q) in queue_family_properties.iter().enumerate() {
        let mut present_support:VkBool32 = VK_FALSE;
        unsafe {
            vkGetPhysicalDeviceSurfaceSupportKHR.unwrap()(*device, index as u32, *surface, &mut present_support);
        }
        let mut graphic_support:VkBool32 = VK_FALSE;
        if (q.queueFlags & VkQueueFlagBits_VK_QUEUE_GRAPHICS_BIT as u32) != 0 {
            graphic_support = VK_TRUE;
        }
        if present_support != VK_FALSE && graphic_support != VK_FALSE {
            graphic_index = index as i32;
            present_index = index as i32;
            break;
        }
    }

    QueueFamilyIndices {
        graphics_family_index: graphic_index,
        present_family_index: present_index,
    }
}


pub fn find_swap_chain_support(device: &VkPhysicalDevice, surface:&VkSurfaceKHR) -> SwapChainSupportDetails {

    let surface_capabilities = unsafe {
        let mut option = ::std::mem::MaybeUninit::uninit();
        vkGetPhysicalDeviceSurfaceCapabilitiesKHR.unwrap()(*device, *surface, option.as_mut_ptr());
        option.assume_init()
    };

    let mut format_count = unsafe {
        let mut option = ::std::mem::MaybeUninit::uninit();
        vkGetPhysicalDeviceSurfaceFormatsKHR.unwrap()(*device, *surface, option.as_mut_ptr(), ::std::ptr::null_mut());
        option.assume_init()
    };
        
    let mut supported_formats: Vec<VkSurfaceFormatKHR> = Vec::with_capacity(format_count as usize);
    unsafe {
        supported_formats.set_len(format_count as usize);
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkGetPhysicalDeviceSurfaceFormatsKHR.unwrap()(*device, *surface, &mut format_count, supported_formats.as_mut_ptr())
        );
    }       

    let mut present_mode_count = unsafe {
        let mut option = ::std::mem::MaybeUninit::uninit();
        vkGetPhysicalDeviceSurfacePresentModesKHR.unwrap()(*device, *surface, option.as_mut_ptr(), ::std::ptr::null_mut());
        option.assume_init()
    };
        
    let mut supported_present_modes: Vec<VkPresentModeKHR> = Vec::with_capacity(present_mode_count as usize);
    unsafe {
        supported_present_modes.set_len(present_mode_count as usize);
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkGetPhysicalDeviceSurfacePresentModesKHR.unwrap()(*device, *surface, &mut present_mode_count, supported_present_modes.as_mut_ptr())
        );
    }    

    SwapChainSupportDetails {
        capabilities: surface_capabilities,
        formats: supported_formats,
        present_modes: supported_present_modes,
    }
}

pub fn get_device_extensions(device:&VkPhysicalDevice) -> Vec<VkExtensionProperties> {
    
    let mut device_extension_count = unsafe {
        let mut output = ::std::mem::MaybeUninit::uninit();
        vkEnumerateDeviceExtensionProperties.unwrap()(*device, ::std::ptr::null_mut(), output.as_mut_ptr(), ::std::ptr::null_mut());
        output.assume_init()
    };
    
    let mut supported_device_extensions: Vec<VkExtensionProperties> = Vec::with_capacity(device_extension_count as usize);
    unsafe {
        supported_device_extensions.set_len(device_extension_count as usize);
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkEnumerateDeviceExtensionProperties.unwrap()(*device, ::std::ptr::null_mut(), &mut device_extension_count, supported_device_extensions.as_mut_ptr())
        );
    }    

    supported_device_extensions
}

pub fn is_device_suitable(device:&VkPhysicalDevice, surface:&VkSurfaceKHR) -> bool {

    let device_properties: VkPhysicalDeviceProperties = unsafe {
        let mut output = ::std::mem::MaybeUninit::uninit();
        vkGetPhysicalDeviceProperties.unwrap()(*device, output.as_mut_ptr());
        output.assume_init()
    };
    
    let device_features: VkPhysicalDeviceFeatures = unsafe {
        let mut output = ::std::mem::MaybeUninit::uninit();
        vkGetPhysicalDeviceFeatures.unwrap()(*device, output.as_mut_ptr());
        output.assume_init()
    };

    let device_extensions = get_device_extensions(device);

    let device_extension_names_str = device_extensions.iter()
                                                .map(|ext| unsafe {::std::ffi::CStr::from_ptr(ext.extensionName.as_ptr())}.to_owned())
                                                .collect::<Vec<::std::ffi::CString>>();
    let has_required_ext = device_extension_names_str.iter()
                                            .find(|elem| {
                                                elem.to_owned().to_str() == unsafe{ ::std::ffi::CStr::from_ptr(VK_KHR_SWAPCHAIN_EXTENSION_NAME.as_ptr() as *const i8) }.to_str()
                                            })
                                            .map_or(false, |_| true);

    let queue_family_indices = find_queue_family_indices(device, surface);
    let swap_chain_details = find_swap_chain_support(device, surface);

    let has_swap_chain_support = has_required_ext && swap_chain_details.formats.len() > 0 && swap_chain_details.present_modes.len() > 0;

    if (device_properties.deviceType == VkPhysicalDeviceType_VK_PHYSICAL_DEVICE_TYPE_DISCRETE_GPU || 
        device_properties.deviceType == VkPhysicalDeviceType_VK_PHYSICAL_DEVICE_TYPE_INTEGRATED_GPU)
        && device_features.geometryShader != 0 
        && device_features.logicOp != 0
        && queue_family_indices.graphics_family_index >= 0
        && queue_family_indices.present_family_index >= 0
        && has_required_ext 
        && has_swap_chain_support
    {
        return true
    }
    return false
}

pub fn find_plane_for_display(device:&VkPhysicalDevice, display:&VkDisplayKHR, plane_properties:&Vec<VkDisplayPlanePropertiesKHR>) -> i32 {

    for (index, plane) in plane_properties.iter().enumerate() {
        if (plane.currentDisplay != ::std::ptr::null_mut()) &&
            (plane.currentDisplay != *display) {
            continue;
        }

        let mut supported_count: u32 = unsafe {
            let mut output = ::std::mem::MaybeUninit::uninit();
            vkGetDisplayPlaneSupportedDisplaysKHR.unwrap()(*device, index as u32, output.as_mut_ptr(), ::std::ptr::null_mut());
            output.assume_init()
        };

        if supported_count == 0 {
            continue;
        }            
        
        let mut supported_displays: Vec<VkDisplayKHR> = Vec::with_capacity(supported_count as usize);
        unsafe {
            supported_displays.set_len(supported_count as usize);
            vkGetDisplayPlaneSupportedDisplaysKHR.unwrap()(*device, index as u32, &mut supported_count, supported_displays.as_mut_ptr());
        }
        
        let found = match supported_displays.iter().find(|item| *item == display ) {
            Some(_) => true,
            None => false, 
        };
        
        if found {
            return index as i32
        }
    }
    return -1;
}

pub fn get_supported_alpha_mode(display_plane_capabilities:&VkDisplayPlaneCapabilitiesKHR) -> VkCompositeAlphaFlagBitsKHR {
    let alpha_mode_types : [u32; 4] = [
        VkDisplayPlaneAlphaFlagBitsKHR_VK_DISPLAY_PLANE_ALPHA_OPAQUE_BIT_KHR as u32,
        VkDisplayPlaneAlphaFlagBitsKHR_VK_DISPLAY_PLANE_ALPHA_GLOBAL_BIT_KHR as u32,
        VkDisplayPlaneAlphaFlagBitsKHR_VK_DISPLAY_PLANE_ALPHA_PER_PIXEL_BIT_KHR as u32,
        VkDisplayPlaneAlphaFlagBitsKHR_VK_DISPLAY_PLANE_ALPHA_PER_PIXEL_PREMULTIPLIED_BIT_KHR as u32
    ];

    for item in alpha_mode_types.iter() {
        if (display_plane_capabilities.supportedAlpha & *item) == 1{
            return *item as VkCompositeAlphaFlagBitsKHR
        }
    }
    return VkDisplayPlaneAlphaFlagBitsKHR_VK_DISPLAY_PLANE_ALPHA_OPAQUE_BIT_KHR
}

pub fn create_shader_module<'a>(device:&mut VkDevice, shader_content:&'a [u32]) -> VkShaderModule {
    let shader_create_info = VkShaderModuleCreateInfo {
        sType: VkStructureType_VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO,
        pNext: ::std::ptr::null_mut(),
        flags: 0,
        codeSize: (shader_content.len() * 4) as _,
        pCode: shader_content.as_ptr() as *const _,
    };

    let shader_module = unsafe {
        let mut option = ::std::mem::MaybeUninit::uninit();
        let result = vkCreateShaderModule.unwrap()(*device, &shader_create_info, ::std::ptr::null_mut(), option.as_mut_ptr());
        if result != VkResult_VK_SUCCESS {
            eprintln!("Failed to create shader module")
        }
        option.assume_init()
    };

    shader_module
}

pub fn destroy_shader_module(device:&mut VkDevice, shader_module:&mut VkShaderModule) {
    unsafe{
        vkDestroyShaderModule.unwrap()(*device, *shader_module, ::std::ptr::null_mut());
    }
}


pub fn read_spirv_from_bytes<Data: ::std::io::Read + ::std::io::Seek>(data: &mut Data) -> ::std::vec::Vec<u32> {
    let size = data.seek(::std::io::SeekFrom::End(0)).unwrap();
    if size % 4 != 0 {
        panic!("Input data length not divisible by 4");
    }
    if size > usize::max_value() as u64 {
        panic!("Input data too long");
    }
    let words = (size / 4) as usize;
    let mut result = Vec::<u32>::with_capacity(words);
    data.seek(::std::io::SeekFrom::Start(0)).unwrap();
    unsafe {
        data.read_exact(::std::slice::from_raw_parts_mut(
            result.as_mut_ptr() as *mut u8,
            words * 4,
        )).unwrap();
        result.set_len(words);
    }
    const MAGIC_NUMBER: u32 = 0x0723_0203;
    if !result.is_empty() && result[0] == MAGIC_NUMBER.swap_bytes() {
        for word in &mut result {
            *word = word.swap_bytes();
        }
    }
    if result.is_empty() || result[0] != MAGIC_NUMBER {
        panic!("Input data is missing SPIR-V magic number");
    }
    result
}