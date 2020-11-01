 #[test]
fn test_vulkan()
{
    use vulkan_bindings::*;
    use super::types::*;

    let app_info = VkApplicationInfo {
        sType: VkStructureType_VK_STRUCTURE_TYPE_APPLICATION_INFO,
        pNext: ::std::ptr::null_mut(),
        pApplicationName: ::std::ffi::CString::new("NRG").unwrap().as_ptr(),
        applicationVersion: VK_API_VERSION_1_0,
        pEngineName: ::std::ffi::CString::new("NRGEngine").unwrap().as_ptr(),
        engineVersion: VK_API_VERSION_1_0,
        apiVersion: VK_API_VERSION_1_0,
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
    
    assert_eq!(true, true);
}