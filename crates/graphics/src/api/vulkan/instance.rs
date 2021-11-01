use super::physical_device::BackendPhysicalDevice;
use super::{
    create_instance, create_surface, enumerate_available_extensions, enumerate_available_layers,
    pick_suitable_physical_device,
};
use nrg_platform::Handle;
use std::ffi::CString;
use vulkan_bindings::*;

pub struct BackendInstance {
    _lib: vulkan_bindings::Lib,
    supported_layers: Vec<VkLayerProperties>,
    supported_extensions: Vec<VkExtensionProperties>,
    instance: VkInstance,
    surface: VkSurfaceKHR,
    physical_device: BackendPhysicalDevice,
    debug_messenger: VkDebugUtilsMessengerEXT,
}

impl std::ops::Deref for BackendInstance {
    type Target = VkInstance;
    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}

impl BackendInstance {
    pub fn new(handle: &Handle, enable_validation: bool) -> BackendInstance {
        let lib = vulkan_bindings::Lib::default();
        VK::initialize(&lib);
        let available_layers = enumerate_available_layers();
        let available_extensions = enumerate_available_extensions();
        let instance = create_instance(&available_layers, &available_extensions, enable_validation);
        let surface = create_surface(instance, handle);
        let physical_dev = pick_suitable_physical_device(instance, surface);
        if physical_dev.is_none() {
            panic!("Unable to find a physical device that support Vulkan needed API");
        }

        let mut debug_messenger: VkDebugUtilsMessengerEXT = ::std::ptr::null_mut();
        unsafe {
            let func_name = CString::new("vkCreateDebugUtilsMessengerEXT").unwrap();
            let debug_create_opt_fn: PFN_vkCreateDebugUtilsMessengerEXT = ::std::mem::transmute(
                vkGetInstanceProcAddr.unwrap()(instance, func_name.as_ptr() as _),
            );
            if let Some(debug_create_fn) = debug_create_opt_fn {
                let severity_flags =
                    VkDebugUtilsMessageSeverityFlagBitsEXT_VK_DEBUG_UTILS_MESSAGE_SEVERITY_VERBOSE_BIT_EXT
                | VkDebugUtilsMessageSeverityFlagBitsEXT_VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT
                | VkDebugUtilsMessageSeverityFlagBitsEXT_VK_DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT
                | VkDebugUtilsMessageSeverityFlagBitsEXT_VK_DEBUG_UTILS_MESSAGE_SEVERITY_INFO_BIT_EXT;
                let message_type =
                    VkDebugUtilsMessageTypeFlagBitsEXT_VK_DEBUG_UTILS_MESSAGE_TYPE_GENERAL_BIT_EXT
                | VkDebugUtilsMessageTypeFlagBitsEXT_VK_DEBUG_UTILS_MESSAGE_TYPE_VALIDATION_BIT_EXT
                | VkDebugUtilsMessageTypeFlagBitsEXT_VK_DEBUG_UTILS_MESSAGE_TYPE_PERFORMANCE_BIT_EXT;
                let debug_create_info = VkDebugUtilsMessengerCreateInfoEXT {
                    sType: VkStructureType_VK_STRUCTURE_TYPE_DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
                    pNext: ::std::ptr::null_mut(),
                    flags: 0,
                    messageSeverity: severity_flags as _,
                    messageType: message_type as _,
                    pfnUserCallback: Some(debug_callback),
                    pUserData: ::std::ptr::null_mut(),
                };

                debug_create_fn(
                    instance,
                    &debug_create_info,
                    ::std::ptr::null_mut(),
                    &mut debug_messenger,
                );
            } else {
                eprintln!("No support for Vulkan debug callback");
            }
        }

        Self {
            _lib: lib,
            supported_layers: available_layers,
            supported_extensions: available_extensions,
            instance,
            surface,
            physical_device: physical_dev.unwrap(),
            debug_messenger,
        }
    }

    pub fn get_surface(&self) -> VkSurfaceKHR {
        self.surface
    }

    pub fn get_physical_device(&self) -> &BackendPhysicalDevice {
        &self.physical_device
    }

    pub fn get_physical_device_mut(&mut self) -> &mut BackendPhysicalDevice {
        &mut self.physical_device
    }

    pub fn get_supported_layers(&self) -> &Vec<VkLayerProperties> {
        &self.supported_layers
    }

    pub fn get_supported_extensions(&self) -> &Vec<VkExtensionProperties> {
        &self.supported_extensions
    }

    pub fn delete(&self) {
        unsafe {
            if !self.debug_messenger.is_null() {
                let func_name = CString::new("vkDestroyDebugUtilsMessengerEXT").unwrap();
                let debug_destroy_opt_fn: PFN_vkDestroyDebugUtilsMessengerEXT =
                    ::std::mem::transmute(vkGetInstanceProcAddr.unwrap()(
                        self.instance,
                        func_name.as_ptr() as _,
                    ));
                if let Some(debug_destroy_fn) = debug_destroy_opt_fn {
                    debug_destroy_fn(self.instance, self.debug_messenger, ::std::ptr::null_mut());
                }
            }
            vkDestroySurfaceKHR.unwrap()(self.instance, self.surface, ::std::ptr::null_mut());
            vkDestroyInstance.unwrap()(self.instance, ::std::ptr::null_mut());
        }
    }
}

extern "C" fn debug_callback(
    message_severity: VkDebugUtilsMessageSeverityFlagBitsEXT,
    _message_types: VkDebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const VkDebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut ::std::os::raw::c_void,
) -> VkBool32 {
    if message_severity
        < VkDebugUtilsMessageSeverityFlagBitsEXT_VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT
    {
    } else {
        if message_severity >= VkDebugUtilsMessageSeverityFlagBitsEXT_VK_DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT
        {
            eprintln!("VK Validation Layer ERROR:");
        } else if message_severity >= VkDebugUtilsMessageSeverityFlagBitsEXT_VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT
        {
            eprintln!("VK Validation Layer WARNING:");
        }
        unsafe {
            let str = std::ffi::CStr::from_ptr((*p_callback_data).pMessage);
            eprintln!("{}", std::str::from_utf8_unchecked(str.to_bytes()));
        }
    }
    VK_FALSE
}
