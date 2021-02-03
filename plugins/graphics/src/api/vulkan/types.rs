use vulkan_bindings::*;

macro_rules! VK_MAKE_VERSION {
    ($major:expr, $minor:expr, $patch:expr) => {
        ($major as u32) << 22 | ($minor as u32) << 12 | ($patch as u32)
    };
}

pub const VK_API_VERSION_1_0: u32 = VK_MAKE_VERSION!(1, 0, 0);
pub const VK_API_VERSION_1_1: u32 = VK_MAKE_VERSION!(1, 1, 0);
pub const VK_API_VERSION_1_2: u32 = VK_MAKE_VERSION!(1, 2, 0);

pub const VK_INVALID_ID: i32 = -1;
pub const MAX_FRAMES_IN_FLIGHT:u32 = 2;


#[derive(Clone, Copy)]
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


#[derive(Clone)]
pub struct SwapChainSupportDetails {
    pub capabilities: VkSurfaceCapabilitiesKHR,
    pub formats: Vec<VkSurfaceFormatKHR>,
    pub present_modes: Vec<VkPresentModeKHR>,
}
