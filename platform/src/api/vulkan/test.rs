 #[test]
fn test_vulkan()
{
    use vulkan_bindings::*;
    use super::types::*;
    use super::lib::*;
    use crate::loader::*;

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

    let library_path = get_vulkan_lib_path();
    let lib = LibLoader::new(library_path);

    let vkCreateInstance = lib.get::<PFN_vkCreateInstance>("vkCreateInstance").unwrap();
    let mut instance:VkInstance = ::std::ptr::null_mut();
    unsafe {        
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkCreateInstance(&create_info, ::std::ptr::null_mut(), &mut instance)
        );
    }

    lib.close();

    assert_eq!(true, true);
}