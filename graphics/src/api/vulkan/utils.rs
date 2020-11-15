#![allow(dead_code)]

use vulkan_bindings::*;

pub struct QueueFamilyIndices {
    pub graphics_family_index: i32,
}

pub fn find_queue_family_indices(lib:&VK, device: &VkPhysicalDevice) -> QueueFamilyIndices {
 
    let mut queue_family_count: u32 = unsafe {
        let mut output = ::std::mem::MaybeUninit::uninit();
        lib.vkGetPhysicalDeviceQueueFamilyProperties.unwrap()(*device, output.as_mut_ptr(), ::std::ptr::null_mut());
        output.assume_init()
    };
    
    let mut queue_family_properties: Vec<VkQueueFamilyProperties> = Vec::with_capacity(queue_family_count as usize);
    unsafe {
        queue_family_properties.set_len(queue_family_count as usize);
        lib.vkGetPhysicalDeviceQueueFamilyProperties.unwrap()(*device, &mut queue_family_count, queue_family_properties.as_mut_ptr());
    }    

    QueueFamilyIndices {
        graphics_family_index: {
            let mut index = -1;
            for q in queue_family_properties.iter() {
                index += 1;
                if (q.queueFlags & VkQueueFlagBits_VK_QUEUE_GRAPHICS_BIT as u32) != 0 {
                    break
                }
            }
            index
        },
    }
}

pub fn is_device_suitable(lib:&VK, device:&VkPhysicalDevice) -> bool {

    let device_properties: VkPhysicalDeviceProperties = unsafe {
        let mut output = ::std::mem::MaybeUninit::uninit();
        lib.vkGetPhysicalDeviceProperties.unwrap()(*device, output.as_mut_ptr());
        output.assume_init()
    };
    
    let device_features: VkPhysicalDeviceFeatures = unsafe {
        let mut output = ::std::mem::MaybeUninit::uninit();
        lib.vkGetPhysicalDeviceFeatures.unwrap()(*device, output.as_mut_ptr());
        output.assume_init()
    };

    let queue_family_indices = find_queue_family_indices(lib, device);

    if (device_properties.deviceType == VkPhysicalDeviceType_VK_PHYSICAL_DEVICE_TYPE_DISCRETE_GPU || 
        device_properties.deviceType == VkPhysicalDeviceType_VK_PHYSICAL_DEVICE_TYPE_INTEGRATED_GPU)
        && device_features.geometryShader != 0 
        && device_features.logicOp != 0
        && queue_family_indices.graphics_family_index >= 0
    {
        return true
    }
    return false
}

pub fn find_plane_for_display(lib:&VK, device:&VkPhysicalDevice, display:&VkDisplayKHR, plane_properties:&Vec<VkDisplayPlanePropertiesKHR>) -> i32 {

    for (index, plane) in plane_properties.iter().enumerate() {
        if (plane.currentDisplay != ::std::ptr::null_mut()) &&
            (plane.currentDisplay != *display) {
            continue;
        }

        let mut supported_count: u32 = unsafe {
            let mut output = ::std::mem::MaybeUninit::uninit();
            lib.vkGetDisplayPlaneSupportedDisplaysKHR.unwrap()(*device, index as u32, output.as_mut_ptr(), ::std::ptr::null_mut());
            output.assume_init()
        };

        if supported_count == 0 {
            continue;
        }            
        
        let mut supported_displays: Vec<VkDisplayKHR> = Vec::with_capacity(supported_count as usize);
        unsafe {
            supported_displays.set_len(supported_count as usize);
            lib.vkGetDisplayPlaneSupportedDisplaysKHR.unwrap()(*device, index as u32, &mut supported_count, supported_displays.as_mut_ptr());
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