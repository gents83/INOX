use std::{cell::RefCell, os::raw::c_char, rc::Rc};
use vulkan_bindings::*;
use nrg_platform::Handle;
use super::types::*;
use super::utils::*;
use super::physical_device::*;

struct InstanceImmutable {
    supported_layers: Vec<VkLayerProperties>,
    supported_extensions: Vec<VkExtensionProperties>,
    instance: VkInstance,
    surface: VkSurfaceKHR,
    physical_device: PhysicalDevice,
}

#[derive(Clone)]
pub struct Instance {
    inner: Rc<RefCell<InstanceImmutable>>,
}

impl Instance {    
    pub fn new(handle: &Handle, enable_validation: bool) -> Instance {
        let immutable = Rc::new(RefCell::new(InstanceImmutable::new(handle, enable_validation)));
        Instance{
            inner: immutable,
        }
    }
    
    pub fn delete(&self) {
        self.inner.borrow_mut().delete();
    }

    pub fn compute_swap_chain_details(&self) {
        let inner = self.inner.borrow_mut();
        inner.physical_device.compute_swap_chain_details(inner.surface);
    }

    pub fn get_surface(&self) -> VkSurfaceKHR {
        self.inner.borrow().surface
    }

    pub fn get_queue_family_info(&self) -> QueueFamilyIndices {
        self.inner.borrow().physical_device.get_queue_family_info()
    }

    pub fn get_physical_device(&self) -> VkPhysicalDevice {
        self.inner.borrow().physical_device.get_internal_device()
    }
    
    pub fn get_swap_chain_info(&self) -> SwapChainSupportDetails {
        self.inner.borrow().physical_device.get_swap_chain_info()
    }

    pub fn get_physical_device_properties(&self) -> VkPhysicalDeviceProperties {
        self.inner.borrow().physical_device.get_properties()
    }

    pub fn get_available_extensions(&self) -> Vec<VkExtensionProperties> {
        self.inner.borrow().physical_device.get_available_extensions()
    }

    pub fn get_available_features(&self) -> VkPhysicalDeviceFeatures {
        self.inner.borrow().physical_device.get_available_features()
    }

    pub fn get_supported_layers(&self) -> Vec<VkLayerProperties> {
        self.inner.borrow().supported_layers.clone()
    }

    pub fn get_supported_extensions(&self) -> Vec<VkExtensionProperties> {
        self.inner.borrow().supported_extensions.clone()
    }
}

impl InstanceImmutable {
    pub fn new(handle: &Handle, enable_validation: bool) -> InstanceImmutable {
        VK::initialize(&vulkan_bindings::Lib::default());
        let available_layers = if enable_validation { enumerate_available_layers() } else { Vec::new() };
        let available_extensions = enumerate_available_extensions();
        let inst = create_instance(&available_layers, &available_extensions);
        let surf = create_surface(inst, &handle);
        let physical_dev = pick_suitable_physical_device(inst, surf);
        if physical_dev.is_none() {
            eprintln!("Unable to find a physical device that support Vulkan needed API");
        }
        InstanceImmutable {
            supported_layers: available_layers,
            supported_extensions: available_extensions,
            instance: inst,
            surface: surf,
            physical_device: physical_dev.unwrap(),
        }
    }

    pub fn delete(&self) {
        unsafe {    
            vkDestroySurfaceKHR.unwrap()(self.instance, self.surface, ::std::ptr::null_mut());
            vkDestroyInstance.unwrap()(self.instance, ::std::ptr::null_mut());
        }
    }
}



fn create_instance( supported_layers: &[VkLayerProperties], 
                    supported_extensions: &[VkExtensionProperties] ) -> VkInstance {    

    let app_info = VkApplicationInfo {
        sType: VkStructureType_VK_STRUCTURE_TYPE_APPLICATION_INFO,
        pNext: ::std::ptr::null_mut(),
        pApplicationName: ::std::ptr::null_mut(),
        applicationVersion: VK_API_VERSION_1_1,
        pEngineName: ::std::ptr::null_mut(),
        engineVersion: VK_API_VERSION_1_1,
        apiVersion: VK_API_VERSION_1_1,
    };

    //Layers
    let layer_names_str = get_available_layers_names(supported_layers);
    let layer_names_ptr = layer_names_str
                                        .iter()
                                        .map(|e| e.as_ptr())
                                        .collect::<Vec<*const c_char>>();

    //Extensions
    let extension_names_str = get_available_extensions_names(supported_extensions);
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

    if instance == ::std::ptr::null_mut() {
        eprintln!("Unable to create instance that support Vulkan needed API");
    }
    instance
} 

#[allow(unused_assignments)]
pub fn create_surface(instance: VkInstance, handle: &Handle) -> VkSurfaceKHR {
    let mut surface: VkSurfaceKHR = ::std::ptr::null_mut();

    #[cfg(target_os = "android")]
    {
        surface = create_surface_android(instance, handle);
    }
    #[cfg(target_os = "ios")]
    {
        surface = create_surface_ios(instance, handle);
    }
    #[cfg(target_os = "macos")]
    {
        surface = create_surface_macos(instance, handle);
    }
    #[cfg(target_os = "unix")]
    {
        surface = create_surface_unix(instance, handle);
    }
    #[cfg(target_os = "wasm32")]
    {
        surface = create_surface_wasm32(instance, handle);
    }
    #[cfg(target_os = "windows")]
    {
        surface = create_surface_win32(instance, handle);
    }

    if surface == ::std::ptr::null_mut() {
        eprintln!("Unable to create a surface to support Vulkan needed API");
    }
    surface
}


pub fn pick_suitable_physical_device(instance: VkInstance, surface: VkSurfaceKHR) -> ::std::option::Option<PhysicalDevice> { 
 
    for vk_physical_device in enumerate_physical_devices(instance) {          
        let physical_device = PhysicalDevice::create(vk_physical_device, surface);

        if physical_device.is_device_suitable() {
            return Some(physical_device);
        }
    }
    None
}


fn create_surface_win32(instance:VkInstance, handle: &Handle) -> VkSurfaceKHR { 
    let surface_create_info = VkWin32SurfaceCreateInfoKHR {
        sType: VkStructureType_VK_STRUCTURE_TYPE_WIN32_SURFACE_CREATE_INFO_KHR,
        pNext: ::std::ptr::null_mut(),
        flags: 0,
        hinstance: handle.handle_impl.hinstance as *mut vulkan_bindings::HINSTANCE__,
        hwnd: handle.handle_impl.hwnd as *mut vulkan_bindings::HWND__,
    };
    
    let surface:VkSurfaceKHR = unsafe {
        let mut output = ::std::mem::MaybeUninit::uninit();
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkCreateWin32SurfaceKHR.unwrap()(instance, &surface_create_info, ::std::ptr::null_mut(), output.as_mut_ptr())
        );
        output.assume_init()
    };    
    surface
} 