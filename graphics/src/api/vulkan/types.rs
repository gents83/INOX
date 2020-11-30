use crate::VK_MAKE_VERSION;
use vulkan_bindings::*;

pub const VK_API_VERSION_1_0: u32 = VK_MAKE_VERSION!(1, 0, 0);
pub const VK_API_VERSION_1_1: u32 = VK_MAKE_VERSION!(1, 1, 0);
pub const VK_API_VERSION_1_2: u32 = VK_MAKE_VERSION!(1, 2, 0);

pub const VK_INVALID_ID: i32 = -1;
pub const MAX_FRAMES_IN_FLIGHT:u32 = 2;


pub struct QueueFamilyIndices {
    pub graphics_family_index: i32,
    pub present_family_index: i32,
}

impl QueueFamilyIndices {
    pub fn is_complete(&self) -> bool {
        self.graphics_family_index != VK_INVALID_ID && 
        self.present_family_index != VK_INVALID_ID
    }
}


pub struct SwapChainSupportDetails {
    pub capabilities: VkSurfaceCapabilitiesKHR,
    pub formats: Vec<VkSurfaceFormatKHR>,
    pub present_modes: Vec<VkPresentModeKHR>,
}
