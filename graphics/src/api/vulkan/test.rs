 #[test]
fn test_vulkan()
{
    use vulkan_bindings::*;
    use super::types::*;

    let app_info = VkApplicationInfo {
        sType: VkStructureType_VK_STRUCTURE_TYPE_APPLICATION_INFO,
        pNext: ::std::ptr::null_mut(),
        pApplicationName: ::std::ffi::CString::new("NRG").unwrap().as_ptr(),
        applicationVersion: VK_API_VERSION_1_1,
        pEngineName: ::std::ffi::CString::new("NRGEngine").unwrap().as_ptr(),
        engineVersion: VK_API_VERSION_1_1,
        apiVersion: VK_API_VERSION_1_1,
    };

    let create_info = VkInstanceCreateInfo {
        sType: VkStructureType_VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
        pNext: ::std::ptr::null_mut(),
        flags: 0,
        pApplicationInfo: &app_info,
        enabledLayerCount: 0,
        ppEnabledLayerNames: ::std::ptr::null_mut(),
        enabledExtensionCount: 0,
        ppEnabledExtensionNames: ::std::ptr::null_mut(), 
    };

    let lib = LibLoader::new(&vulkan_bindings::Library::new(), "1.1").unwrap();
    
    let mut instance:VkInstance = ::std::ptr::null_mut();
    unsafe {        
        assert_eq!(
            VkResult_VK_SUCCESS,
            lib.vkCreateInstance.unwrap()(&create_info, ::std::ptr::null_mut(), &mut instance)
        );
    }

    let mut physical_device_count = 0;
    unsafe {
        assert_eq!(
            VkResult_VK_SUCCESS,
            lib.vkEnumeratePhysicalDevices.unwrap()(instance, &mut physical_device_count, ::std::ptr::null_mut())
        );
    }
    assert_ne!(physical_device_count, 0);

    let mut extension_count = 0;
    unsafe {
        assert_eq!(
            VkResult_VK_SUCCESS,
            lib.vkEnumerateInstanceExtensionProperties.unwrap()(::std::ptr::null_mut(), &mut extension_count, ::std::ptr::null_mut())
        );
    }
    assert_ne!(extension_count, 0);

}