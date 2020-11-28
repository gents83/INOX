
use vulkan_bindings::*;
use super::types::*;
use std::os::raw::c_char;

pub struct Instance {
    instance: VkInstance,
}

impl Instance {
    pub fn new() -> Instance {
        VK::initialize(&vulkan_bindings::Lib::new());
        Self {
            instance: create_instance(),
        }
    }
}

fn get_available_layers_count() -> u32 {
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

fn get_available_layers() -> Vec<VkLayerProperties> {
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

fn get_available_extensions_count() -> u32 {
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

fn get_available_extensions() -> Vec<VkExtensionProperties> {
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


fn create_instance() -> VkInstance {
    
    let app_info = VkApplicationInfo {
        sType: VkStructureType_VK_STRUCTURE_TYPE_APPLICATION_INFO,
        pNext: ::std::ptr::null_mut(),
        pApplicationName: ::std::ffi::CString::new("NRG").unwrap().as_ptr(),
        applicationVersion: VK_API_VERSION_1_1,
        pEngineName: ::std::ffi::CString::new("NRGEngine").unwrap().as_ptr(),
        engineVersion: VK_API_VERSION_1_1,
        apiVersion: VK_API_VERSION_1_1,
    };

    //Layers
    let available_layers = get_available_layers();    
    let layer_names_str = available_layers.iter()
                                        .map(|layer| unsafe {::std::ffi::CStr::from_ptr(layer.layerName.as_ptr())}.to_owned())
                                        .collect::<Vec<::std::ffi::CString>>();
    let layer_names_ptr = layer_names_str.iter()
                                            .map(|e| e.as_ptr())
                                            .collect::<Vec<*const i8>>();

    //Extensions
    let supported_extensions = get_available_extensions();
    let extension_names_str = supported_extensions.iter()
                                            .map(|ext| unsafe {::std::ffi::CStr::from_ptr(ext.extensionName.as_ptr())}.to_owned())
                                            .collect::<Vec<::std::ffi::CString>>();
    let extension_names_ptr = extension_names_str.iter()
                                            .map(|e| e.as_ptr())
                                            .collect::<Vec<*const i8>>();

    //Create Instance
    let create_info = VkInstanceCreateInfo {
        sType: VkStructureType_VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
        pNext: ::std::ptr::null_mut(),
        flags: 0,
        pApplicationInfo: &app_info,
        enabledLayerCount: layer_names_ptr.len() as u32,
        ppEnabledLayerNames: layer_names_ptr.as_ptr(),
        enabledExtensionCount: extension_names_ptr.len() as u32,
        ppEnabledExtensionNames: extension_names_ptr.as_ptr(), 
    };
   
    let mut instance:VkInstance = ::std::ptr::null_mut();
    unsafe {        
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkCreateInstance.unwrap()(&create_info, ::std::ptr::null_mut(), &mut instance)
        );
    }
    instance
} 