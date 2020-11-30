 #![allow(dead_code)]

use vulkan_bindings::*;
use nrg_platform::*;
use super::utils::*;

#[test]
fn test_vulkan()
{
    use super::types::*;  
    use super::instance::*;
    use super::device::*;
    use super::shader::*;

    let window = 
    Window::create( String::from("NRG TEST"),
                   String::from("NRG - Vulkan Test"),
                   10, 10,
                   1024, 768 );

    let mut instance = Instance::new(&window.handle, false);
    let mut device = Device::new(&mut instance);
    device.create_swap_chain()
            .create_image_views()
            .create_render_pass()
            .create_graphics_pipeline()
            .create_framebuffers()
            .create_command_pool()
            .create_command_buffers()
            .create_sync_objects();
    
    device.temp_draw_frame();
                        
    device.delete();        
    instance.delete();
}

#[allow(non_snake_case)]
fn test_vulkan_create_win32_display_surface(instance:&mut VkInstance) -> VkSurfaceKHR
{
    let window =  Window::create( String::from("Test Window"),
                    String::from("Test Window"),
                    100, 100,
                    1024, 768 );

    let surface_create_info = VkWin32SurfaceCreateInfoKHR {
        sType: VkStructureType_VK_STRUCTURE_TYPE_WIN32_SURFACE_CREATE_INFO_KHR,
        pNext: ::std::ptr::null_mut(),
        flags: 0,
        hinstance: unsafe {::std::mem::transmute(window.handle.handle_impl.hinstance)},
        hwnd: unsafe {::std::mem::transmute(window.handle.handle_impl.hwnd)},
    };
    
    let surface:VkSurfaceKHR = unsafe {
        let mut output = ::std::mem::MaybeUninit::uninit();
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkCreateWin32SurfaceKHR.unwrap()(*instance, &surface_create_info, ::std::ptr::null_mut(), output.as_mut_ptr())
        );
        output.assume_init()
    };
    
    surface
}


fn test_vulkan_create_khr_display_surface(physical_device:&mut VkPhysicalDevice, instance:&mut VkInstance) -> VkSurfaceKHR
{
    let mut display_count:u32 = 0;
    unsafe {
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkGetPhysicalDeviceDisplayPropertiesKHR.unwrap()(*physical_device, &mut display_count, ::std::ptr::null_mut())
        );
    }
    assert_ne!(display_count, 0);

    let mut display_properties: Vec<VkDisplayPropertiesKHR> = Vec::with_capacity(display_count as usize);
    unsafe {
        display_properties.set_len(display_count as usize);
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkGetPhysicalDeviceDisplayPropertiesKHR.unwrap()(*physical_device, &mut display_count, display_properties.as_mut_ptr())
        );
    }  
    assert_ne!(display_properties.len(), 0);
    assert_eq!(display_properties.len(), display_count as usize);

    let display_selected = 0;
    let mut mode_count = 0;
    unsafe {
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkGetDisplayModePropertiesKHR.unwrap()(*physical_device, display_properties[display_selected].display, &mut mode_count, ::std::ptr::null_mut())
        );
    }
    assert_ne!(mode_count, 0);
    
    let mut display_modes: Vec<VkDisplayModePropertiesKHR> = Vec::with_capacity(mode_count as usize);
    unsafe {
        display_modes.set_len(mode_count as usize);
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkGetDisplayModePropertiesKHR.unwrap()(*physical_device, display_properties[display_selected].display, &mut mode_count, display_modes.as_mut_ptr())
        );
    }  
    assert_ne!(display_modes.len(), 0);
    assert_eq!(display_modes.len(), mode_count as usize);
    
    let mode_selected = 0;
    let mut plane_count = 0;
    unsafe {
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkGetPhysicalDeviceDisplayPlanePropertiesKHR.unwrap()(*physical_device, &mut plane_count, ::std::ptr::null_mut())
        );
    }
    assert_ne!(plane_count, 0);
            
    let mut display_planes: Vec<VkDisplayPlanePropertiesKHR> = Vec::with_capacity(plane_count as usize);
    unsafe {
        display_planes.set_len(plane_count as usize);
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkGetPhysicalDeviceDisplayPlanePropertiesKHR.unwrap()(*physical_device, &mut plane_count, display_planes.as_mut_ptr())
        );
    }  
    assert_ne!(display_planes.len(), 0);
    assert_eq!(display_planes.len(), plane_count as usize);

    let plane_selected = find_plane_for_display(physical_device, &display_properties[display_selected].display, &display_planes);
    assert_ne!(plane_selected, -1);

    let display_plane_capabilities = unsafe {
        let mut output = ::std::mem::MaybeUninit::uninit();
        assert_eq!(
            VkResult_VK_SUCCESS,
            vkGetDisplayPlaneCapabilitiesKHR.unwrap()(*physical_device, display_modes[mode_selected].displayMode, plane_selected as u32, output.as_mut_ptr())
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
            vkCreateDisplayPlaneSurfaceKHR.unwrap()(*instance, &surface_info, ::std::ptr::null(), &mut surface)
        );
    }  

    surface
}

