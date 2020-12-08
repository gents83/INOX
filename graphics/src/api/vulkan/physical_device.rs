
use vulkan_bindings::*;
use super::types::*;

pub struct PhysicalDevice {    
    physical_device: VkPhysicalDevice,
    physical_device_properties: VkPhysicalDeviceProperties,
    physical_device_features: VkPhysicalDeviceFeatures,
    physical_device_extensions: Vec<VkExtensionProperties>,
    queue_family_indices: QueueFamilyIndices,
    swap_chain_details: SwapChainSupportDetails,
}

impl PhysicalDevice {
    pub fn create(physical_device: VkPhysicalDevice, surface: VkSurfaceKHR) -> PhysicalDevice {
                          
        let physical_device_properties: VkPhysicalDeviceProperties = unsafe {
            let mut output = ::std::mem::MaybeUninit::uninit();
            vkGetPhysicalDeviceProperties.unwrap()(physical_device, output.as_mut_ptr());
            output.assume_init()
        };
        let physical_device_features: VkPhysicalDeviceFeatures = unsafe {
            let mut output = ::std::mem::MaybeUninit::uninit();
            vkGetPhysicalDeviceFeatures.unwrap()(physical_device, output.as_mut_ptr());
            output.assume_init()
        };
        let mut device_extension_count = unsafe {
            let mut output = ::std::mem::MaybeUninit::uninit();
            vkEnumerateDeviceExtensionProperties.unwrap()(physical_device, ::std::ptr::null_mut(), output.as_mut_ptr(), ::std::ptr::null_mut());
            output.assume_init()
        };            
        let mut physical_device_extensions: Vec<VkExtensionProperties> = Vec::with_capacity(device_extension_count as usize);
        unsafe {
            physical_device_extensions.set_len(device_extension_count as usize);
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkEnumerateDeviceExtensionProperties.unwrap()(physical_device, ::std::ptr::null_mut(), &mut device_extension_count, physical_device_extensions.as_mut_ptr())
            );
        }   

        let mut result = PhysicalDevice {
            physical_device: physical_device,
            physical_device_properties: physical_device_properties, 
            physical_device_features: physical_device_features,
            physical_device_extensions: physical_device_extensions,
            queue_family_indices: QueueFamilyIndices {
                graphics_family_index: VK_INVALID_ID, 
                present_family_index: VK_INVALID_ID, 
            },
            swap_chain_details: SwapChainSupportDetails {
                capabilities: unsafe { ::std::mem::zeroed() }, 
                formats: Vec::new(),
                present_modes: Vec::new(),
            },
        };
        
        result.find_queue_family_indices(surface);
        result.find_swap_chain_support(surface);
        
        result
    }

    pub fn is_initialized(&self) -> bool {
        self.physical_device != ::std::ptr::null_mut()
    }

    fn find_queue_family_indices(&mut self, surface: VkSurfaceKHR) {    
        let mut queue_family_count: u32 = unsafe {
            let mut output = ::std::mem::MaybeUninit::uninit();
            vkGetPhysicalDeviceQueueFamilyProperties.unwrap()(self.physical_device, output.as_mut_ptr(), ::std::ptr::null_mut());
            output.assume_init()
        };
        
        let mut queue_family_properties: Vec<VkQueueFamilyProperties> = Vec::with_capacity(queue_family_count as usize);
        unsafe {
            queue_family_properties.set_len(queue_family_count as usize);
            vkGetPhysicalDeviceQueueFamilyProperties.unwrap()(self.physical_device, &mut queue_family_count, queue_family_properties.as_mut_ptr());
        }    

        let mut graphic_index = VK_INVALID_ID;
        let mut present_index = VK_INVALID_ID;
        
        for (index, q) in queue_family_properties.iter().enumerate() {            
            if (q.queueFlags & VkQueueFlagBits_VK_QUEUE_GRAPHICS_BIT as u32) != 0 {
                graphic_index = index as _;
            }
            let mut present_support:VkBool32 = VK_FALSE;
            unsafe {
                vkGetPhysicalDeviceSurfaceSupportKHR.unwrap()(self.physical_device, index as u32, surface, &mut present_support);
            }
            if present_support != VK_FALSE {
                present_index = index as _;
            }
            if graphic_index != VK_INVALID_ID && present_index != VK_INVALID_ID {
                break;
            }
        }

        self.queue_family_indices = QueueFamilyIndices {
            graphics_family_index: graphic_index,
            present_family_index: present_index,
        };
    }
    
    fn find_swap_chain_support(&mut self, surface: VkSurfaceKHR) {
        let surface_capabilities = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            vkGetPhysicalDeviceSurfaceCapabilitiesKHR.unwrap()(self.physical_device, surface, option.as_mut_ptr());
            option.assume_init()
        };

        let mut format_count = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            vkGetPhysicalDeviceSurfaceFormatsKHR.unwrap()(self.physical_device, surface, option.as_mut_ptr(), ::std::ptr::null_mut());
            option.assume_init()
        };
            
        let mut supported_formats: Vec<VkSurfaceFormatKHR> = Vec::with_capacity(format_count as usize);
        unsafe {
            supported_formats.set_len(format_count as usize);
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkGetPhysicalDeviceSurfaceFormatsKHR.unwrap()(self.physical_device, surface, &mut format_count, supported_formats.as_mut_ptr())
            );
        }       

        let mut present_mode_count = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            vkGetPhysicalDeviceSurfacePresentModesKHR.unwrap()(self.physical_device, surface, option.as_mut_ptr(), ::std::ptr::null_mut());
            option.assume_init()
        };
            
        let mut supported_present_modes: Vec<VkPresentModeKHR> = Vec::with_capacity(present_mode_count as usize);
        unsafe {
            supported_present_modes.set_len(present_mode_count as usize);
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkGetPhysicalDeviceSurfacePresentModesKHR.unwrap()(self.physical_device, surface, &mut present_mode_count, supported_present_modes.as_mut_ptr())
            );
        }    

        self.swap_chain_details = SwapChainSupportDetails {
            capabilities: surface_capabilities,
            formats: supported_formats,
            present_modes: supported_present_modes,
        };
    }

        
    pub fn is_device_suitable(&self) -> bool {
        let device_extension_names_str = self.physical_device_extensions
                                                    .iter()
                                                    .map(|ext| unsafe {::std::ffi::CStr::from_ptr(ext.extensionName.as_ptr())}.to_owned())
                                                    .collect::<Vec<::std::ffi::CString>>();
        let has_required_ext = device_extension_names_str
                                    .iter()
                                    .find(|elem| {
                                        elem.to_owned().to_str() == unsafe{ 
                                            ::std::ffi::CStr::from_ptr(VK_KHR_SWAPCHAIN_EXTENSION_NAME.as_ptr() as *const i8) 
                                        }.to_str()
                                    })
                                    .map_or(false, |_| true);

        let has_swap_chain_support =   has_required_ext && 
                                            self.swap_chain_details.formats.len() > 0 && 
                                            self.swap_chain_details.present_modes.len() > 0;

        let has_required_device_type = self.physical_device_properties.deviceType == VkPhysicalDeviceType_VK_PHYSICAL_DEVICE_TYPE_DISCRETE_GPU ||
                                            self.physical_device_properties.deviceType == VkPhysicalDeviceType_VK_PHYSICAL_DEVICE_TYPE_INTEGRATED_GPU;

        let has_minimum_features = self.physical_device_features.geometryShader != 0 &&
                                        self.physical_device_features.logicOp != 0;

        let has_surface_support =  self.queue_family_indices.is_complete();

        if has_required_device_type && has_minimum_features && has_surface_support && has_swap_chain_support {
            return true
        }
        false
    }

    pub fn get_queue_family_info(&self) -> &QueueFamilyIndices {
        &self.queue_family_indices
    }

    pub fn get_swap_chain_info(&self) -> &SwapChainSupportDetails {
        &self.swap_chain_details
    }

    pub fn get_available_extensions(&self) -> &Vec<VkExtensionProperties> {
        &self.physical_device_extensions
    }

    pub fn get_available_features(&self) -> &VkPhysicalDeviceFeatures {
        &self.physical_device_features
    }

    pub fn get_properties(&self) -> &VkPhysicalDeviceProperties {
        &self.physical_device_properties
    }
}

impl From<&PhysicalDevice> for VkPhysicalDevice {
    fn from(physical_device: &PhysicalDevice) -> VkPhysicalDevice {
        physical_device.physical_device
    }
}