#![allow(dead_code)]

use vulkan_bindings::*;
use super::types::*;

pub fn get_available_layers_count() -> u32 {
    let layers_count = unsafe {
        let mut option = ::std::mem::MaybeUninit::uninit();
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkEnumerateInstanceLayerProperties.unwrap()(option.as_mut_ptr(), ::std::ptr::null_mut())
        );
        option.assume_init()
    };
    layers_count
}

pub fn enumerate_available_layers() -> Vec<VkLayerProperties> {
    let mut layers_count = get_available_layers_count();
    
    let mut available_layers: Vec<VkLayerProperties> = Vec::with_capacity(layers_count as usize);
    unsafe {
        available_layers.set_len(layers_count as usize);
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkEnumerateInstanceLayerProperties.unwrap()(&mut layers_count, available_layers.as_mut_ptr())
        );
    }    
    available_layers
}


pub fn get_available_layers_names(supported_layers: &Vec<VkLayerProperties>) -> Vec<::std::ffi::CString>{
    supported_layers
        .iter()
        .map(|layer| unsafe {::std::ffi::CStr::from_ptr(layer.layerName.as_ptr())}.to_owned())
        .collect::<Vec<::std::ffi::CString>>()
}

pub fn get_available_extensions_count() -> u32 {
    let extension_count = unsafe {
        let mut option = ::std::mem::MaybeUninit::uninit();
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkEnumerateInstanceExtensionProperties.unwrap()(::std::ptr::null_mut(), option.as_mut_ptr(), ::std::ptr::null_mut())
        );
        option.assume_init()
    };
    extension_count

}

pub fn enumerate_available_extensions() -> Vec<VkExtensionProperties> {
    let mut extensions_count = get_available_extensions_count();
    let mut supported_extensions: Vec<VkExtensionProperties> = Vec::with_capacity(extensions_count as usize);
    unsafe {
        supported_extensions.set_len(extensions_count as usize);
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkEnumerateInstanceExtensionProperties.unwrap()(::std::ptr::null_mut(), &mut extensions_count, supported_extensions.as_mut_ptr())
        );
    }    
    supported_extensions
}

pub fn get_available_extensions_names(supported_extensions: &Vec<VkExtensionProperties>) -> Vec<::std::ffi::CString>{
    supported_extensions
        .iter()
        .map(|ext| unsafe {::std::ffi::CStr::from_ptr(ext.extensionName.as_ptr())}.to_owned())
        .collect::<Vec<::std::ffi::CString>>()
}

pub fn get_physical_devices_count(instance: VkInstance) -> u32 {            
    let physical_device_count =  unsafe {
        let mut option = ::std::mem::MaybeUninit::uninit();
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkEnumeratePhysicalDevices.unwrap()(instance, option.as_mut_ptr(), ::std::ptr::null_mut())
        );
        option.assume_init()
    };
    physical_device_count
}

pub fn enumerate_physical_devices(instance: VkInstance) -> Vec<VkPhysicalDevice> {
    let mut physical_device_count = get_physical_devices_count(instance);            
    let mut physical_devices: Vec<VkPhysicalDevice> = Vec::with_capacity(physical_device_count as usize);
    unsafe {
        physical_devices.set_len(physical_device_count as usize);
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkEnumeratePhysicalDevices.unwrap()(instance, &mut physical_device_count, physical_devices.as_mut_ptr())
        );
    }   
    physical_devices 
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

pub fn find_available_memory_type(physical_device: VkPhysicalDevice, filter: u32, properties: VkMemoryPropertyFlags) -> u32 {
    let mem_properties = unsafe {
        let mut option = ::std::mem::MaybeUninit::uninit();            
        vkGetPhysicalDeviceMemoryProperties.unwrap()(physical_device, option.as_mut_ptr());
        option.assume_init()
    };  
    for i in 0..mem_properties.memoryTypeCount {
        let is_correct_type = (filter & (1 << i)) != 0;
        let mem_type = mem_properties.memoryTypes[i as usize];
        let has_needed_properties = (mem_type.propertyFlags & properties) == properties;
        if  is_correct_type && has_needed_properties {
            return i;
        }
    }
    eprintln!("Unable to find suitable memory type with filter {} and flags {}", filter, properties);
    return VK_INVALID_ID as _;
}
