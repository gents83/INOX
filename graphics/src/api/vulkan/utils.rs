#![allow(dead_code)]

use std::ffi::{CStr, CString};

use super::types::*;
use vulkan_bindings::*;

pub fn get_minimum_required_vulkan_extensions() -> Vec<CString> {
    vec![unsafe { CStr::from_ptr(VK_KHR_SWAPCHAIN_EXTENSION_NAME.as_ptr() as *const _) }.to_owned()]
}

pub fn get_available_layers_count() -> u32 {
    unsafe {
        let mut option = ::std::mem::MaybeUninit::uninit();
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkEnumerateInstanceLayerProperties.unwrap()(
                option.as_mut_ptr(),
                ::std::ptr::null_mut()
            )
        );
        option.assume_init()
    }
}

pub fn enumerate_available_layers() -> Vec<VkLayerProperties> {
    let mut layers_count = get_available_layers_count();

    let mut available_layers: Vec<VkLayerProperties> = Vec::with_capacity(layers_count as usize);
    unsafe {
        available_layers.set_len(layers_count as usize);
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkEnumerateInstanceLayerProperties.unwrap()(
                &mut layers_count,
                available_layers.as_mut_ptr()
            )
        );
    }
    available_layers
}

pub fn get_available_layers_names(
    supported_layers: &[VkLayerProperties],
) -> Vec<::std::ffi::CString> {
    supported_layers
        .iter()
        .map(|layer| unsafe { ::std::ffi::CStr::from_ptr(layer.layerName.as_ptr()) }.to_owned())
        .collect::<Vec<::std::ffi::CString>>()
}

pub fn get_available_extensions_count() -> u32 {
    unsafe {
        let mut option = ::std::mem::MaybeUninit::uninit();
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkEnumerateInstanceExtensionProperties.unwrap()(
                ::std::ptr::null_mut(),
                option.as_mut_ptr(),
                ::std::ptr::null_mut()
            )
        );
        option.assume_init()
    }
}

pub fn enumerate_available_extensions() -> Vec<VkExtensionProperties> {
    let mut extensions_count = get_available_extensions_count();
    let mut supported_extensions: Vec<VkExtensionProperties> =
        Vec::with_capacity(extensions_count as usize);
    unsafe {
        supported_extensions.set_len(extensions_count as usize);
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkEnumerateInstanceExtensionProperties.unwrap()(
                ::std::ptr::null_mut(),
                &mut extensions_count,
                supported_extensions.as_mut_ptr()
            )
        );
    }
    supported_extensions
}

pub fn get_available_extensions_names(
    supported_extensions: &[VkExtensionProperties],
) -> Vec<::std::ffi::CString> {
    supported_extensions
        .iter()
        .map(|ext| unsafe { ::std::ffi::CStr::from_ptr(ext.extensionName.as_ptr()) }.to_owned())
        .collect::<Vec<::std::ffi::CString>>()
}

pub fn get_physical_devices_count(instance: VkInstance) -> u32 {
    unsafe {
        let mut option = ::std::mem::MaybeUninit::uninit();
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkEnumeratePhysicalDevices.unwrap()(
                instance,
                option.as_mut_ptr(),
                ::std::ptr::null_mut()
            )
        );
        option.assume_init()
    }
}

pub fn enumerate_physical_devices(instance: VkInstance) -> Vec<VkPhysicalDevice> {
    let mut physical_device_count = get_physical_devices_count(instance);
    let mut physical_devices: Vec<VkPhysicalDevice> =
        Vec::with_capacity(physical_device_count as usize);
    unsafe {
        physical_devices.set_len(physical_device_count as usize);
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkEnumeratePhysicalDevices.unwrap()(
                instance,
                &mut physical_device_count,
                physical_devices.as_mut_ptr()
            )
        );
    }
    physical_devices
}

pub fn find_plane_for_display(
    device: &VkPhysicalDevice,
    display: &VkDisplayKHR,
    plane_properties: &[VkDisplayPlanePropertiesKHR],
) -> i32 {
    for (index, plane) in plane_properties.iter().enumerate() {
        if (plane.currentDisplay != ::std::ptr::null_mut()) && (plane.currentDisplay != *display) {
            continue;
        }

        let mut supported_count: u32 = unsafe {
            let mut output = ::std::mem::MaybeUninit::uninit();
            vkGetDisplayPlaneSupportedDisplaysKHR.unwrap()(
                *device,
                index as u32,
                output.as_mut_ptr(),
                ::std::ptr::null_mut(),
            );
            output.assume_init()
        };

        if supported_count == 0 {
            continue;
        }

        let mut supported_displays: Vec<VkDisplayKHR> =
            Vec::with_capacity(supported_count as usize);
        unsafe {
            supported_displays.set_len(supported_count as usize);
            vkGetDisplayPlaneSupportedDisplaysKHR.unwrap()(
                *device,
                index as u32,
                &mut supported_count,
                supported_displays.as_mut_ptr(),
            );
        }

        let found = supported_displays.iter().any(|item| item == display);

        if found {
            return index as i32;
        }
    }
    -1
}

pub fn get_supported_alpha_mode(
    display_plane_capabilities: &VkDisplayPlaneCapabilitiesKHR,
) -> VkCompositeAlphaFlagBitsKHR {
    let alpha_mode_types: [u32; 4] = [
        VkDisplayPlaneAlphaFlagBitsKHR_VK_DISPLAY_PLANE_ALPHA_OPAQUE_BIT_KHR as u32,
        VkDisplayPlaneAlphaFlagBitsKHR_VK_DISPLAY_PLANE_ALPHA_GLOBAL_BIT_KHR as u32,
        VkDisplayPlaneAlphaFlagBitsKHR_VK_DISPLAY_PLANE_ALPHA_PER_PIXEL_BIT_KHR as u32,
        VkDisplayPlaneAlphaFlagBitsKHR_VK_DISPLAY_PLANE_ALPHA_PER_PIXEL_PREMULTIPLIED_BIT_KHR
            as u32,
    ];

    for item in alpha_mode_types.iter() {
        if (display_plane_capabilities.supportedAlpha & *item) == 1 {
            return *item as VkCompositeAlphaFlagBitsKHR;
        }
    }
    VkDisplayPlaneAlphaFlagBitsKHR_VK_DISPLAY_PLANE_ALPHA_OPAQUE_BIT_KHR
}

pub fn find_available_memory_type(
    physical_device: VkPhysicalDevice,
    filter: u32,
    properties: VkMemoryPropertyFlags,
) -> u32 {
    let mem_properties = unsafe {
        let mut option = ::std::mem::MaybeUninit::uninit();
        vkGetPhysicalDeviceMemoryProperties.unwrap()(physical_device, option.as_mut_ptr());
        option.assume_init()
    };
    for i in 0..mem_properties.memoryTypeCount {
        let is_correct_type = (filter & (1 << i)) != 0;
        let mem_type = mem_properties.memoryTypes[i as usize];
        let has_needed_properties = (mem_type.propertyFlags & properties) == properties;
        if is_correct_type && has_needed_properties {
            return i;
        }
    }
    eprintln!(
        "Unable to find suitable memory type with filter {} and flags {}",
        filter, properties
    );
    VK_INVALID_ID as _
}

pub fn create_image_view(
    device: VkDevice,
    image: VkImage,
    format: VkFormat,
    aspect_flags: VkImageAspectFlags,
    layers_count: usize,
) -> VkImageView {
    let view_info = VkImageViewCreateInfo {
        sType: VkStructureType_VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
        pNext: ::std::ptr::null_mut(),
        flags: 0,
        image,
        viewType: if layers_count <= 1 {
            VkImageViewType_VK_IMAGE_VIEW_TYPE_2D
        } else {
            VkImageViewType_VK_IMAGE_VIEW_TYPE_2D_ARRAY
        },
        format,
        components: VkComponentMapping {
            r: VkComponentSwizzle_VK_COMPONENT_SWIZZLE_R,
            g: VkComponentSwizzle_VK_COMPONENT_SWIZZLE_G,
            b: VkComponentSwizzle_VK_COMPONENT_SWIZZLE_B,
            a: VkComponentSwizzle_VK_COMPONENT_SWIZZLE_A,
        },
        subresourceRange: VkImageSubresourceRange {
            aspectMask: aspect_flags,
            baseMipLevel: 0,
            levelCount: 1,
            baseArrayLayer: 0,
            layerCount: layers_count as _,
        },
    };

    unsafe {
        let mut option = ::std::mem::MaybeUninit::uninit();
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkCreateImageView.unwrap()(
                device,
                &view_info,
                ::std::ptr::null_mut(),
                option.as_mut_ptr()
            )
        );
        option.assume_init()
    }
}

pub fn find_supported_format(
    physical_device: VkPhysicalDevice,
    formats_candidates: &[VkFormat],
    tiling: VkImageTiling,
    features: VkFormatFeatureFlags,
) -> VkFormat {
    for format in formats_candidates.iter() {
        let props: VkFormatProperties = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            vkGetPhysicalDeviceFormatProperties.unwrap()(
                physical_device,
                *format,
                option.as_mut_ptr(),
            );
            option.assume_init()
        };
        if (tiling == VkImageTiling_VK_IMAGE_TILING_LINEAR
            && (props.linearTilingFeatures & features) == features)
            || (tiling == VkImageTiling_VK_IMAGE_TILING_OPTIMAL
                && (props.optimalTilingFeatures & features) == features)
        {
            return *format;
        }
    }
    panic!("Failed to find any supported format between available candidates");
}

pub fn find_depth_format(physical_device: VkPhysicalDevice) -> VkFormat {
    let format_candidates = [
        VkFormat_VK_FORMAT_D32_SFLOAT,
        VkFormat_VK_FORMAT_D32_SFLOAT_S8_UINT,
        VkFormat_VK_FORMAT_D24_UNORM_S8_UINT,
    ];
    find_supported_format(
        physical_device,
        &format_candidates,
        VkImageTiling_VK_IMAGE_TILING_OPTIMAL,
        VkFormatFeatureFlagBits_VK_FORMAT_FEATURE_DEPTH_STENCIL_ATTACHMENT_BIT as _,
    )
}

pub fn has_stencil_component(format: VkFormat) -> bool {
    format == VkFormat_VK_FORMAT_D32_SFLOAT_S8_UINT
        || format == VkFormat_VK_FORMAT_D24_UNORM_S8_UINT
}
