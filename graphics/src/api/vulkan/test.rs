 #![allow(dead_code)]

use vulkan_bindings::*;
use nrg_platform::*;
use super::utils::*;

#[test]
fn test_vulkan()
{ 
    use crate::data_formats::*;
    use crate::block::*;
    use crate::chunk::*;
    use crate::world::*;
    use nrg_math::*;
    use super::instance::*;
    use super::device::*;

    let window = 
    Window::create( String::from("TEST"),
                   String::from("Vulkan Test"),
                   100, 100,
                   1024, 768 );

    let mut model_transform:Matrix4f = Matrix4f::identity();
    let cam_pos:Vector3f = [0.0, 16.0, -64.0].into();

    /*
    let vertices: [VertexData; 4] = [
        VertexData { pos: [-20., -20., 0.].into(), normal: [0., 0., 1.].into(), color: [1., 0., 0.].into(), tex_coord: [1., 0.].into()},
        VertexData { pos: [ 20., -20., 0.].into(), normal: [0., 0., 1.].into(), color: [0., 1., 0.].into(), tex_coord: [0., 0.].into()},
        VertexData { pos: [ 20.,  20., 0.].into(), normal: [0., 0., 1.].into(), color: [0., 0., 1.].into(), tex_coord: [0., 1.].into()},
        VertexData { pos: [-20.,  20., 0.].into(), normal: [0., 0., 1.].into(), color: [1., 1., 1.].into(), tex_coord: [1., 1.].into()},
    ]; 
    let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];
    */

    // Add sphere
    let radius = 6;
    let chunk_size = 8;
    let mut world = World::default();
    BlockConfig::new("Solid", true, false);
    world.create_sphere(Block::from_name("Solid"), 
                        &WorldBlockCoordinate::new(chunk_size - radius, chunk_size - radius, chunk_size - radius),
                        &WorldBlockCoordinate::new(chunk_size + radius, chunk_size + radius, chunk_size + radius));

    world.update(10, cam_pos);

    let mut vertices: Vec<VertexData> = Vec::new();
    for (index, chunk) in world.visible_chunks.iter_mut() {
        vertices.append(&mut chunk.vertices);
    }

    let mut instance = Instance::new(&window.handle, false);
    let mut device = Device::new(&mut instance);
    device.create_graphics_pipeline()
            .create_texture_image()
            .create_texture_sampler()
            .create_vertex_buffer(vertices.as_slice())
            //.create_vertex_buffer(&vertices)
            //.create_index_buffer(&indices)
            .create_uniform_buffers()
            .create_descriptor_sets()
            .create_command_buffers();
    
    let mut frame_index = 0;
    loop {

        let rotation = Matrix4::from_axis_angle([0.0, 0.0, 0.0].into(), Degree(0.1).into());
        model_transform.set_translation([0.0, 0.0, 0.0].into());
        model_transform = rotation * model_transform;

        frame_index = device.temp_draw_frame(frame_index, &model_transform, cam_pos);
        if frame_index > 1000 {
            break;
        }
    }
                        
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

