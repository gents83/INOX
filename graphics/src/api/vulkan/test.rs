 #![allow(dead_code)]

use vulkan_bindings::*;
use super::utils::*;

 #[test]
fn test_vulkan()
{
    use super::types::*;
    
    let lib = VK::new(&vulkan_bindings::Lib::new());

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
            lib.vkEnumerateInstanceLayerProperties.unwrap()(&mut layers_count, ::std::ptr::null_mut())
        );
    }
    assert_ne!(layers_count, 0);    

    let mut available_layers: Vec<VkLayerProperties> = Vec::with_capacity(layers_count as usize);
    unsafe {
        available_layers.set_len(layers_count as usize);
        assert_eq!(
            VkResult_VK_SUCCESS,
            lib.vkEnumerateInstanceLayerProperties.unwrap()(&mut layers_count, available_layers.as_mut_ptr())
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
            lib.vkEnumerateInstanceExtensionProperties.unwrap()(::std::ptr::null_mut(), &mut extension_count, ::std::ptr::null_mut())
        );
    }
    assert_ne!(extension_count, 0);

    let mut supported_extensions: Vec<VkExtensionProperties> = Vec::with_capacity(extension_count as usize);
    unsafe {
        supported_extensions.set_len(extension_count as usize);
        assert_eq!(
            VkResult_VK_SUCCESS,
            lib.vkEnumerateInstanceExtensionProperties.unwrap()(::std::ptr::null_mut(), &mut extension_count, supported_extensions.as_mut_ptr())
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
            lib.vkCreateInstance.unwrap()(&create_info, ::std::ptr::null_mut(), &mut instance)
        );
    }

    //Physical Device

    let mut physical_device_count = 0;
    unsafe {
        assert_eq!(
            VkResult_VK_SUCCESS,
            lib.vkEnumeratePhysicalDevices.unwrap()(instance, &mut physical_device_count, ::std::ptr::null_mut())
        );
    }
    assert_ne!(physical_device_count, 0);
    
    let mut physical_devices: Vec<VkPhysicalDevice> = Vec::with_capacity(physical_device_count as usize);
    unsafe {
        physical_devices.set_len(physical_device_count as usize);
        assert_eq!(
            VkResult_VK_SUCCESS,
            lib.vkEnumeratePhysicalDevices.unwrap()(instance, &mut physical_device_count, physical_devices.as_mut_ptr())
        );
    }    
    assert_ne!(physical_devices.len(), 0);
    assert_eq!(physical_devices.len(), physical_device_count as usize);

    for physical_device in physical_devices.into_iter() {
        assert_eq!(is_device_suitable(&lib, &physical_device), true);
        
        let queue_family_indices = find_queue_family_indices(&lib, &physical_device);

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
            lib.vkGetPhysicalDeviceFeatures.unwrap()(physical_device, output.as_mut_ptr());
            output.assume_init()
        };

        let mut device_extension_count = 0;
        unsafe {
            assert_eq!(
                VkResult_VK_SUCCESS,
                lib.vkEnumerateDeviceExtensionProperties.unwrap()(physical_device, ::std::ptr::null_mut(), &mut device_extension_count, ::std::ptr::null_mut())
            );
        }
        assert_ne!(device_extension_count, 0);
    
        let mut supported_device_extensions: Vec<VkExtensionProperties> = Vec::with_capacity(device_extension_count as usize);
        unsafe {
            supported_device_extensions.set_len(device_extension_count as usize);
            assert_eq!(
                VkResult_VK_SUCCESS,
                lib.vkEnumerateDeviceExtensionProperties.unwrap()(physical_device, ::std::ptr::null_mut(), &mut device_extension_count, supported_device_extensions.as_mut_ptr())
            );
        }    
        assert_ne!(supported_device_extensions.len(), 0);
        assert_eq!(supported_device_extensions.len(), device_extension_count as usize);

        let device_extension_names_str = supported_device_extensions.iter()
                                                    .map(|ext| unsafe {::std::ffi::CStr::from_ptr(ext.extensionName.as_ptr())}.to_owned())
                                                    .collect::<Vec<::std::ffi::CString>>();
        let device_extension_names_ptr = device_extension_names_str.iter()
                                                    .map(|e| e.as_ptr())
                                                    .collect::<Vec<*const i8>>();

        assert_eq!(device_extension_names_str.len(), supported_device_extensions.len());


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
                lib.vkCreateDevice.unwrap()(physical_device, &device_create_info, ::std::ptr::null_mut(), &mut device)
            );
        }
        assert_ne!(device, ::std::ptr::null_mut());

        let graphics_queue: VkQueue = unsafe {
            let mut output = ::std::mem::MaybeUninit::uninit();
            lib.vkGetDeviceQueue.unwrap()(device, queue_family_indices.graphics_family_index as u32, 0, output.as_mut_ptr());
            output.assume_init()
        };
        assert_ne!(graphics_queue, ::std::ptr::null_mut());


        unsafe {        
            lib.vkDestroyDevice.unwrap()(device, ::std::ptr::null_mut());
        }
    }    

    //Destroy Instance

    unsafe {        
        lib.vkDestroyInstance.unwrap()(instance, ::std::ptr::null_mut());
    }
}



fn test_vulkan_create_display_surface(lib:&VK, physical_device:&mut VkPhysicalDevice, instance:&mut VkInstance)
{
    let mut display_count:u32 = 0;
    unsafe {
        assert_eq!(
            VkResult_VK_SUCCESS,
            lib.vkGetPhysicalDeviceDisplayPropertiesKHR.unwrap()(*physical_device, &mut display_count, ::std::ptr::null_mut())
        );
    }
    assert_ne!(display_count, 0);

    let mut display_properties: Vec<VkDisplayPropertiesKHR> = Vec::with_capacity(display_count as usize);
    unsafe {
        display_properties.set_len(display_count as usize);
        assert_eq!(
            VkResult_VK_SUCCESS,
            lib.vkGetPhysicalDeviceDisplayPropertiesKHR.unwrap()(*physical_device, &mut display_count, display_properties.as_mut_ptr())
        );
    }  
    assert_ne!(display_properties.len(), 0);
    assert_eq!(display_properties.len(), display_count as usize);

    let display_selected = 0;
    let mut mode_count = 0;
    unsafe {
        assert_eq!(
            VkResult_VK_SUCCESS,
            lib.vkGetDisplayModePropertiesKHR.unwrap()(*physical_device, display_properties[display_selected].display, &mut mode_count, ::std::ptr::null_mut())
        );
    }
    assert_ne!(mode_count, 0);
    
    let mut display_modes: Vec<VkDisplayModePropertiesKHR> = Vec::with_capacity(mode_count as usize);
    unsafe {
        display_modes.set_len(mode_count as usize);
        assert_eq!(
            VkResult_VK_SUCCESS,
            lib.vkGetDisplayModePropertiesKHR.unwrap()(*physical_device, display_properties[display_selected].display, &mut mode_count, display_modes.as_mut_ptr())
        );
    }  
    assert_ne!(display_modes.len(), 0);
    assert_eq!(display_modes.len(), mode_count as usize);
    
    let mode_selected = 0;
    let mut plane_count = 0;
    unsafe {
        assert_eq!(
            VkResult_VK_SUCCESS,
            lib.vkGetPhysicalDeviceDisplayPlanePropertiesKHR.unwrap()(*physical_device, &mut plane_count, ::std::ptr::null_mut())
        );
    }
    assert_ne!(plane_count, 0);
            
    let mut display_planes: Vec<VkDisplayPlanePropertiesKHR> = Vec::with_capacity(plane_count as usize);
    unsafe {
        display_planes.set_len(plane_count as usize);
        assert_eq!(
            VkResult_VK_SUCCESS,
            lib.vkGetPhysicalDeviceDisplayPlanePropertiesKHR.unwrap()(*physical_device, &mut plane_count, display_planes.as_mut_ptr())
        );
    }  
    assert_ne!(display_planes.len(), 0);
    assert_eq!(display_planes.len(), plane_count as usize);

    let plane_selected = find_plane_for_display(&lib, physical_device, &display_properties[display_selected].display, &display_planes);
    assert_ne!(plane_selected, -1);

    let display_plane_capabilities = unsafe {
        let mut output = ::std::mem::MaybeUninit::uninit();
        assert_eq!(
            VkResult_VK_SUCCESS,
            lib.vkGetDisplayPlaneCapabilitiesKHR.unwrap()(*physical_device, display_modes[mode_selected].displayMode, plane_selected as u32, output.as_mut_ptr())
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
            lib.vkCreateDisplayPlaneSurfaceKHR.unwrap()(*instance, &surface_info, ::std::ptr::null(), &mut surface)
        );
    }  

    unsafe {        
        lib.vkDestroySurfaceKHR.unwrap()(*instance, surface, ::std::ptr::null_mut());
    }
}