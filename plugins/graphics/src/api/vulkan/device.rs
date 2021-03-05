use super::instance::*;
use super::types::*;
use super::utils::*;
use std::{cell::RefCell, os::raw::c_char, rc::Rc};
use vulkan_bindings::*;

pub struct SwapChain {
    ptr: VkSwapchainKHR,
    pub image_data: Vec<ImageViewData>,
    pub depth_image_data: Vec<ImageViewData>,
}

impl Default for SwapChain {
    fn default() -> Self {
        Self {
            ptr: ::std::ptr::null_mut(),
            image_data: Vec::new(),
            depth_image_data: Vec::new(),
        }
    }
}

pub struct ImageViewData {
    pub image: VkImage,
    pub image_view: VkImageView,
    pub image_memory: VkDeviceMemory,
}

impl Default for ImageViewData {
    fn default() -> Self {
        Self {
            image: ::std::ptr::null_mut(),
            image_view: ::std::ptr::null_mut(),
            image_memory: ::std::ptr::null_mut(),
        }
    }
}

pub struct DeviceImmutable {
    device: VkDevice,
    graphics_queue: VkQueue,
    present_queue: VkQueue,
    swap_chain: SwapChain,
    command_pool: VkCommandPool,
    command_buffers: Vec<VkCommandBuffer>,
    current_frame_index: u32,
    current_image_index: u32,
    image_available_semaphores: Vec<VkSemaphore>,
    render_finished_semaphores: Vec<VkSemaphore>,
    inflight_fences: Vec<VkFence>,
    inflight_images: Vec<VkFence>,
}

#[derive(Clone)]
pub struct Device {
    instance: Instance,
    inner: Rc<RefCell<DeviceImmutable>>,
}

impl Device {
    pub fn new(instance: &Instance) -> Self {
        let immutable = Rc::new(RefCell::new(DeviceImmutable::new(instance)));
        Device {
            instance: instance.clone(),
            inner: immutable,
        }
    }

    pub fn delete(&self) {
        self.inner.borrow_mut().delete()
    }

    pub fn get_device(&self) -> VkDevice {
        self.inner.borrow().device
    }

    pub fn get_instance(&self) -> &Instance {
        &self.instance
    }

    pub fn get_images_count(&self) -> usize {
        self.inner.borrow().swap_chain.image_data.len()
    }

    pub fn get_image_view(&self, index: usize) -> VkImageView {
        self.inner.borrow().swap_chain.image_data[index].image_view
    }

    pub fn get_depth_image_view(&self, index: usize) -> VkImageView {
        self.inner.borrow().swap_chain.depth_image_data[index].image_view
    }

    pub fn get_current_image_index(&self) -> usize {
        self.inner.borrow().current_image_index as _
    }

    pub fn get_current_command_buffer(&self) -> VkCommandBuffer {
        let inner = self.inner.borrow();
        inner.command_buffers[inner.current_image_index as usize]
    }

    pub fn create_buffer(
        &self,
        buffer_size: VkDeviceSize,
        usage: VkBufferUsageFlags,
        properties: VkMemoryPropertyFlags,
        buffer: &mut VkBuffer,
        buffer_memory: &mut VkDeviceMemory,
    ) {
        self.inner.borrow().create_buffer(
            self.instance.get_physical_device(),
            buffer_size,
            usage,
            properties,
            buffer,
            buffer_memory,
        );
    }

    pub fn destroy_buffer(&self, buffer: &VkBuffer, buffer_memory: &VkDeviceMemory) {
        self.inner.borrow().destroy_buffer(buffer, buffer_memory);
    }

    pub fn copy_buffer(
        &self,
        buffer_src: &VkBuffer,
        buffer_dst: &mut VkBuffer,
        buffer_size: VkDeviceSize,
    ) {
        self.inner
            .borrow()
            .copy_buffer(buffer_src, buffer_dst, buffer_size);
    }

    pub fn map_buffer_memory<T>(&self, buffer_memory: &mut VkDeviceMemory, data_src: &[T]) {
        self.inner
            .borrow()
            .map_buffer_memory(buffer_memory, data_src);
    }

    pub fn create_image_view(&self, image: VkImage, format: VkFormat) -> VkImageView {
        create_image_view(
            self.inner.borrow().device,
            image,
            format,
            VkImageAspectFlagBits_VK_IMAGE_ASPECT_COLOR_BIT as _,
        )
    }

    pub fn create_image(
        &self,
        image_properties: (u32, u32, VkFormat),
        tiling: VkImageTiling,
        usage: VkImageUsageFlags,
        properties: VkMemoryPropertyFlags,
    ) -> (VkImage, VkDeviceMemory) {
        self.inner.borrow().create_image(
            self.instance.get_physical_device(),
            image_properties,
            tiling,
            usage,
            properties,
        )
    }

    pub fn transition_image_layout(
        &self,
        image: VkImage,
        old_layout: VkImageLayout,
        new_layout: VkImageLayout,
    ) {
        self.inner
            .borrow()
            .transition_image_layout(image, old_layout, new_layout);
    }

    pub fn copy_buffer_to_image(
        &self,
        buffer: VkBuffer,
        image: VkImage,
        image_width: u32,
        image_height: u32,
    ) {
        self.inner
            .borrow()
            .copy_buffer_to_image(buffer, image, image_width, image_height);
    }

    pub fn begin_frame(&mut self) -> bool {
        if !self.inner.borrow_mut().prepare_for_new_frame() {
            return false;
        }
        let command_buffer = self.get_current_command_buffer();
        self.inner.borrow_mut().begin_frame(command_buffer);
        true
    }

    pub fn end_frame(&self) {
        let command_buffer = self.get_current_command_buffer();
        self.inner.borrow_mut().end_frame(command_buffer);
    }

    pub fn submit(&mut self) -> bool {
        let command_buffer = self.get_current_command_buffer();
        self.inner.borrow_mut().submit(command_buffer)
    }

    pub fn recreate_swap_chain(&mut self) {
        self.inner.borrow_mut().recreate_swap_chain(&self.instance);
    }
}

impl DeviceImmutable {
    pub fn new(instance: &Instance) -> Self {
        DeviceImmutable::create(instance)
    }

    pub fn delete(&mut self) {
        unsafe {
            self.cleanup_swap_chain();

            for i in 0..MAX_FRAMES_IN_FLIGHT {
                vkDestroySemaphore.unwrap()(
                    self.device,
                    self.render_finished_semaphores[i as usize],
                    ::std::ptr::null_mut(),
                );
                vkDestroySemaphore.unwrap()(
                    self.device,
                    self.image_available_semaphores[i as usize],
                    ::std::ptr::null_mut(),
                );
                vkDestroyFence.unwrap()(
                    self.device,
                    self.inflight_fences[i as usize],
                    ::std::ptr::null_mut(),
                );
            }

            vkDestroyCommandPool.unwrap()(self.device, self.command_pool, ::std::ptr::null_mut());
            vkDestroyDevice.unwrap()(self.device, ::std::ptr::null_mut());
        }
    }

    fn prepare_for_new_frame(&mut self) -> bool {
        unsafe {
            vkWaitForFences.unwrap()(
                self.device,
                1,
                &self.inflight_fences[self.current_frame_index as usize],
                VK_TRUE,
                std::u64::MAX,
            );

            let new_frame_index = (self.current_frame_index + 1) % MAX_FRAMES_IN_FLIGHT as u32;

            let result = vkAcquireNextImageKHR.unwrap()(
                self.device,
                self.swap_chain.ptr,
                ::std::u64::MAX,
                self.image_available_semaphores[new_frame_index as usize],
                ::std::ptr::null_mut(),
                &mut self.current_image_index,
            );

            if result == VkResult_VK_ERROR_OUT_OF_DATE_KHR {
                self.wait_device();
                return false;
            } else if result != VkResult_VK_SUCCESS && result != VkResult_VK_SUBOPTIMAL_KHR {
                eprintln!("Failed to acquire swap chain image");
            }

            if self.inflight_images[self.current_image_index as usize] != ::std::ptr::null_mut() {
                vkWaitForFences.unwrap()(
                    self.device,
                    1,
                    &self.inflight_images[self.current_image_index as usize],
                    VK_TRUE,
                    std::u64::MAX,
                );
            }
            self.inflight_images[self.current_image_index as usize] =
                self.inflight_fences[new_frame_index as usize];

            self.current_frame_index = new_frame_index;
        }

        true
    }

    fn begin_frame(&mut self, command_buffer: VkCommandBuffer) {
        let begin_info = VkCommandBufferBeginInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
            flags: 0,
            pNext: ::std::ptr::null_mut(),
            pInheritanceInfo: ::std::ptr::null_mut(),
        };

        unsafe {
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkBeginCommandBuffer.unwrap()(command_buffer, &begin_info)
            );
        }
    }

    fn end_frame(&self, command_buffer: VkCommandBuffer) {
        unsafe {
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkEndCommandBuffer.unwrap()(command_buffer)
            );
        }
    }

    fn submit(&mut self, command_buffer: VkCommandBuffer) -> bool {
        unsafe {
            let wait_stages =
                [VkPipelineStageFlagBits_VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT];
            let submit_info = VkSubmitInfo {
                sType: VkStructureType_VK_STRUCTURE_TYPE_SUBMIT_INFO,
                pNext: ::std::ptr::null_mut(),
                waitSemaphoreCount: 1,
                pWaitSemaphores: &self.image_available_semaphores
                    [self.current_frame_index as usize],
                pWaitDstStageMask: wait_stages.as_ptr() as *const _,
                commandBufferCount: 1,
                pCommandBuffers: &command_buffer,
                signalSemaphoreCount: 1,
                pSignalSemaphores: &self.render_finished_semaphores
                    [self.current_frame_index as usize],
            };

            vkResetFences.unwrap()(
                self.device,
                1,
                &self.inflight_fences[self.current_frame_index as usize],
            );

            assert_eq!(
                VkResult_VK_SUCCESS,
                vkQueueSubmit.unwrap()(
                    self.graphics_queue,
                    1,
                    &submit_info,
                    self.inflight_fences[self.current_frame_index as usize]
                )
            );

            let present_info = VkPresentInfoKHR {
                sType: VkStructureType_VK_STRUCTURE_TYPE_PRESENT_INFO_KHR,
                pNext: ::std::ptr::null_mut(),
                waitSemaphoreCount: 1,
                pWaitSemaphores: &self.render_finished_semaphores
                    [self.current_frame_index as usize],
                swapchainCount: 1,
                pSwapchains: &self.swap_chain.ptr,
                pImageIndices: &self.current_image_index,
                pResults: ::std::ptr::null_mut(),
            };

            let result = vkQueuePresentKHR.unwrap()(self.present_queue, &present_info);

            if result == VkResult_VK_ERROR_OUT_OF_DATE_KHR || result == VkResult_VK_SUBOPTIMAL_KHR {
                self.wait_device();
                return false;
            } else if result != VkResult_VK_SUCCESS {
                eprintln!("Failed to present swap chain image!");
            }

            vkQueueWaitIdle.unwrap()(self.present_queue);
        }
        true
    }

    pub fn create_image(
        &self,
        physical_device: VkPhysicalDevice,
        image_properties: (u32, u32, VkFormat),
        tiling: VkImageTiling,
        usage: VkImageUsageFlags,
        properties: VkMemoryPropertyFlags,
    ) -> (VkImage, VkDeviceMemory) {
        let mut image: VkImage = ::std::ptr::null_mut();
        let mut image_memory: VkDeviceMemory = ::std::ptr::null_mut();

        let image_info = VkImageCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            imageType: VkImageType_VK_IMAGE_TYPE_2D,
            format: image_properties.2,
            extent: VkExtent3D {
                width: image_properties.0,
                height: image_properties.1,
                depth: 1,
            },
            mipLevels: 1,
            arrayLayers: 1,
            samples: VkSampleCountFlagBits_VK_SAMPLE_COUNT_1_BIT,
            tiling: tiling as _,
            usage: usage as _,
            sharingMode: VkSharingMode_VK_SHARING_MODE_EXCLUSIVE,
            queueFamilyIndexCount: 0,
            pQueueFamilyIndices: ::std::ptr::null_mut(),
            initialLayout: VkImageLayout_VK_IMAGE_LAYOUT_UNDEFINED,
        };

        unsafe {
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkCreateImage.unwrap()(
                    self.device,
                    &image_info,
                    ::std::ptr::null_mut(),
                    &mut image
                )
            );
        }

        let mem_requirement = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            vkGetImageMemoryRequirements.unwrap()(self.device, image, option.as_mut_ptr());
            option.assume_init()
        };

        let mem_alloc_info = VkMemoryAllocateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
            pNext: ::std::ptr::null_mut(),
            allocationSize: mem_requirement.size,
            memoryTypeIndex: find_available_memory_type(
                physical_device,
                mem_requirement.memoryTypeBits,
                properties as _,
            ),
        };

        unsafe {
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkAllocateMemory.unwrap()(
                    self.device,
                    &mem_alloc_info,
                    ::std::ptr::null_mut(),
                    &mut image_memory
                )
            );

            vkBindImageMemory.unwrap()(self.device, image, image_memory, 0);
        }

        (image, image_memory)
    }

    fn transition_image_layout(
        &self,
        image: VkImage,
        old_layout: VkImageLayout,
        new_layout: VkImageLayout,
    ) {
        let command_buffer = self.begin_single_time_commands();

        let mut barrier = VkImageMemoryBarrier {
            sType: VkStructureType_VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER,
            pNext: ::std::ptr::null_mut(),
            srcAccessMask: 0,
            dstAccessMask: 0,
            oldLayout: old_layout,
            newLayout: new_layout,
            srcQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED as _,
            dstQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED as _,
            image,
            subresourceRange: VkImageSubresourceRange {
                aspectMask: VkImageAspectFlagBits_VK_IMAGE_ASPECT_COLOR_BIT as _,
                baseMipLevel: 0,
                levelCount: 1,
                baseArrayLayer: 0,
                layerCount: 1,
            },
        };

        let source_stage_flags: VkPipelineStageFlags;
        let destination_stage_flags: VkPipelineStageFlags;

        if old_layout == VkImageLayout_VK_IMAGE_LAYOUT_UNDEFINED
            && new_layout == VkImageLayout_VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL
        {
            barrier.srcAccessMask = 0;
            barrier.dstAccessMask = VkAccessFlagBits_VK_ACCESS_TRANSFER_WRITE_BIT as _;

            source_stage_flags = VkPipelineStageFlagBits_VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT as _;
            destination_stage_flags = VkPipelineStageFlagBits_VK_PIPELINE_STAGE_TRANSFER_BIT as _;
        } else if old_layout == VkImageLayout_VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL
            && new_layout == VkImageLayout_VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL
        {
            barrier.srcAccessMask = VkAccessFlagBits_VK_ACCESS_TRANSFER_WRITE_BIT as _;
            barrier.dstAccessMask = VkAccessFlagBits_VK_ACCESS_SHADER_READ_BIT as _;

            source_stage_flags = VkPipelineStageFlagBits_VK_PIPELINE_STAGE_TRANSFER_BIT as _;
            destination_stage_flags =
                VkPipelineStageFlagBits_VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT as _;
        } else {
            panic!("Unsupported couple of old_layout and new_layout");
        }

        unsafe {
            vkCmdPipelineBarrier.unwrap()(
                command_buffer,
                source_stage_flags,
                destination_stage_flags,
                0,
                0,
                ::std::ptr::null_mut(),
                0,
                ::std::ptr::null_mut(),
                1,
                &barrier,
            );
        }

        self.end_single_time_commands(command_buffer);
    }

    pub fn copy_buffer_to_image(
        &self,
        buffer: VkBuffer,
        image: VkImage,
        image_width: u32,
        image_height: u32,
    ) {
        let command_buffer = self.begin_single_time_commands();

        let region = VkBufferImageCopy {
            bufferOffset: 0,
            bufferRowLength: 0,
            bufferImageHeight: 0,
            imageSubresource: VkImageSubresourceLayers {
                aspectMask: VkImageAspectFlagBits_VK_IMAGE_ASPECT_COLOR_BIT as _,
                mipLevel: 0,
                baseArrayLayer: 0,
                layerCount: 1,
            },
            imageOffset: VkOffset3D { x: 0, y: 0, z: 0 },
            imageExtent: VkExtent3D {
                width: image_width,
                height: image_height,
                depth: 1,
            },
        };

        unsafe {
            vkCmdCopyBufferToImage.unwrap()(
                command_buffer,
                buffer,
                image,
                VkImageLayout_VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
                1,
                &region,
            );
        }

        self.end_single_time_commands(command_buffer);
    }

    pub fn begin_single_time_commands(&self) -> VkCommandBuffer {
        let command_alloc_info = VkCommandBufferAllocateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
            pNext: ::std::ptr::null_mut(),
            commandPool: self.command_pool,
            level: VkCommandBufferLevel_VK_COMMAND_BUFFER_LEVEL_PRIMARY,
            commandBufferCount: 1,
        };

        let command_buffer = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkAllocateCommandBuffers.unwrap()(
                    self.device,
                    &command_alloc_info,
                    option.as_mut_ptr()
                )
            );
            option.assume_init()
        };

        let begin_info = VkCommandBufferBeginInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
            flags: VkCommandBufferUsageFlagBits_VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT as _,
            pNext: ::std::ptr::null_mut(),
            pInheritanceInfo: ::std::ptr::null_mut(),
        };

        unsafe {
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkBeginCommandBuffer.unwrap()(command_buffer, &begin_info)
            );
        }

        command_buffer
    }

    pub fn end_single_time_commands(&self, command_buffer: VkCommandBuffer) {
        unsafe {
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkEndCommandBuffer.unwrap()(command_buffer)
            );
        }

        let submit_info = VkSubmitInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_SUBMIT_INFO,
            pNext: ::std::ptr::null_mut(),
            waitSemaphoreCount: 0,
            pWaitSemaphores: ::std::ptr::null_mut(),
            pWaitDstStageMask: ::std::ptr::null_mut(),
            commandBufferCount: 1,
            pCommandBuffers: &command_buffer,
            signalSemaphoreCount: 0,
            pSignalSemaphores: ::std::ptr::null_mut(),
        };

        unsafe {
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkQueueSubmit.unwrap()(
                    self.graphics_queue,
                    1,
                    &submit_info,
                    ::std::ptr::null_mut()
                )
            );

            vkQueueWaitIdle.unwrap()(self.graphics_queue);

            vkFreeCommandBuffers.unwrap()(self.device, self.command_pool, 1, &command_buffer);
        }
    }

    fn create_buffer(
        &self,
        physical_device: VkPhysicalDevice,
        buffer_size: VkDeviceSize,
        usage: VkBufferUsageFlags,
        properties: VkMemoryPropertyFlags,
        buffer: &mut VkBuffer,
        buffer_memory: &mut VkDeviceMemory,
    ) {
        let buffer_info = VkBufferCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            size: buffer_size as _,
            usage: usage as _,
            sharingMode: VkSharingMode_VK_SHARING_MODE_EXCLUSIVE,
            queueFamilyIndexCount: 0,
            pQueueFamilyIndices: ::std::ptr::null_mut(),
        };
        unsafe {
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkCreateBuffer.unwrap()(self.device, &buffer_info, ::std::ptr::null_mut(), buffer)
            );
        }

        let mem_requirement = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            vkGetBufferMemoryRequirements.unwrap()(self.device, *buffer, option.as_mut_ptr());
            option.assume_init()
        };

        let mem_alloc_info = VkMemoryAllocateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
            pNext: ::std::ptr::null_mut(),
            allocationSize: mem_requirement.size,
            memoryTypeIndex: find_available_memory_type(
                physical_device,
                mem_requirement.memoryTypeBits,
                properties as _,
            ),
        };

        unsafe {
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkAllocateMemory.unwrap()(
                    self.device,
                    &mem_alloc_info,
                    ::std::ptr::null_mut(),
                    buffer_memory
                )
            );

            vkBindBufferMemory.unwrap()(self.device, *buffer, *buffer_memory, 0);
        }
    }

    fn destroy_buffer(&self, buffer: &VkBuffer, buffer_memory: &VkDeviceMemory) {
        unsafe {
            vkDestroyBuffer.unwrap()(self.device, *buffer, ::std::ptr::null_mut());
            vkFreeMemory.unwrap()(self.device, *buffer_memory, ::std::ptr::null_mut());
        }
    }

    fn copy_buffer(
        &self,
        buffer_src: &VkBuffer,
        buffer_dst: &mut VkBuffer,
        buffer_size: VkDeviceSize,
    ) {
        let command_buffer = self.begin_single_time_commands();

        unsafe {
            let copy_region = VkBufferCopy {
                srcOffset: 0,
                dstOffset: 0,
                size: buffer_size,
            };

            vkCmdCopyBuffer.unwrap()(command_buffer, *buffer_src, *buffer_dst, 1, &copy_region);
        }
        self.end_single_time_commands(command_buffer);
    }

    fn map_buffer_memory<T>(&self, buffer_memory: &mut VkDeviceMemory, data_src: &[T]) {
        unsafe {
            let length = ::std::mem::size_of::<T>() * data_src.len();

            let data_ptr = {
                let mut option = ::std::mem::MaybeUninit::uninit();
                assert_eq!(
                    VkResult_VK_SUCCESS,
                    vkMapMemory.unwrap()(
                        self.device,
                        *buffer_memory,
                        0,
                        length as _,
                        0,
                        option.as_mut_ptr()
                    )
                );
                option.assume_init()
            };
            ::std::ptr::copy_nonoverlapping(data_src.as_ptr() as _, data_ptr, length as _);
            vkUnmapMemory.unwrap()(self.device, *buffer_memory);
        }
    }
}

impl DeviceImmutable {
    fn create(instance: &Instance) -> Self {
        let queue_priority: f32 = 1.0;
        let mut hash_family_indices: ::std::collections::HashSet<u32> =
            ::std::collections::HashSet::new();
        hash_family_indices.insert(instance.get_queue_family_info().graphics_family_index as _);
        hash_family_indices.insert(instance.get_queue_family_info().present_family_index as _);

        let mut queue_infos: Vec<VkDeviceQueueCreateInfo> = Vec::new();
        for family_index in hash_family_indices.into_iter() {
            let queue_create_info = VkDeviceQueueCreateInfo {
                sType: VkStructureType_VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
                pNext: ::std::ptr::null_mut(),
                flags: 0,
                queueFamilyIndex: family_index,
                queueCount: 1,
                pQueuePriorities: &queue_priority,
            };
            queue_infos.push(queue_create_info);
        }

        let layer_names_str = get_available_layers_names(&instance.get_supported_layers());
        let layer_names_ptr = layer_names_str
            .iter()
            .map(|e| e.as_ptr())
            .collect::<Vec<*const c_char>>();

        let device_extension_names_str =
            get_available_extensions_names(&instance.get_available_extensions());
        let device_extension_names_ptr = device_extension_names_str
            .iter()
            .map(|e| e.as_ptr())
            .collect::<Vec<*const c_char>>();

        let mut device_features = instance.get_available_features();
        device_features.samplerAnisotropy = VK_TRUE;

        let device_create_info = VkDeviceCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            queueCreateInfoCount: queue_infos.len() as _,
            pQueueCreateInfos: queue_infos.as_ptr(),
            enabledLayerCount: layer_names_str.len() as _,
            ppEnabledLayerNames: layer_names_ptr.as_ptr(),
            enabledExtensionCount: device_extension_names_ptr.len() as u32,
            ppEnabledExtensionNames: device_extension_names_ptr.as_ptr(),
            pEnabledFeatures: &device_features,
        };

        let mut device: VkDevice = ::std::ptr::null_mut();
        unsafe {
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkCreateDevice.unwrap()(
                    instance.get_physical_device(),
                    &device_create_info,
                    ::std::ptr::null_mut(),
                    &mut device
                )
            );
        }

        let graphics_queue: VkQueue = unsafe {
            let mut output = ::std::mem::MaybeUninit::uninit();
            vkGetDeviceQueue.unwrap()(
                device,
                instance.get_queue_family_info().graphics_family_index as _,
                0,
                output.as_mut_ptr(),
            );
            output.assume_init()
        };
        let present_queue: VkQueue = unsafe {
            let mut output = ::std::mem::MaybeUninit::uninit();
            vkGetDeviceQueue.unwrap()(
                device,
                instance.get_queue_family_info().present_family_index as _,
                0,
                output.as_mut_ptr(),
            );
            output.assume_init()
        };

        let mut inner_device = Self {
            device,
            graphics_queue,
            present_queue,
            swap_chain: SwapChain::default(),
            command_pool: ::std::ptr::null_mut(),
            command_buffers: Vec::new(),
            current_frame_index: 0,
            current_image_index: 0,
            image_available_semaphores: Vec::new(),
            render_finished_semaphores: Vec::new(),
            inflight_fences: Vec::new(),
            inflight_images: Vec::new(),
        };
        inner_device
            .create_swap_chain(&instance)
            .create_command_pool(&instance)
            .create_image_views(&instance)
            .allocate_command_buffers()
            .create_sync_objects();
        inner_device
    }

    fn create_swap_chain(&mut self, instance: &Instance) -> &mut Self {
        let details = instance.get_swap_chain_info();
        let queue_family_info = instance.get_queue_family_info();
        let mut family_indices: Vec<u32> = Vec::new();

        let mut swap_chain_extent = VkExtent2D {
            width: details.capabilities.currentExtent.width,
            height: details.capabilities.currentExtent.height,
        };
        swap_chain_extent.width = ::std::cmp::max(
            details.capabilities.minImageExtent.width,
            std::cmp::min(
                details.capabilities.maxImageExtent.width,
                swap_chain_extent.width,
            ),
        );
        swap_chain_extent.height = ::std::cmp::max(
            details.capabilities.minImageExtent.height,
            std::cmp::min(
                details.capabilities.maxImageExtent.height,
                swap_chain_extent.height,
            ),
        );

        let mut swap_chain_create_info = VkSwapchainCreateInfoKHR {
            sType: VkStructureType_VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            surface: instance.get_surface(),
            minImageCount: ::std::cmp::min(
                details.capabilities.minImageCount + 1,
                details.capabilities.maxImageCount,
            ),
            imageFormat: details.formats[0].format,
            imageColorSpace: details.formats[0].colorSpace,
            imageExtent: swap_chain_extent,
            imageArrayLayers: 1,
            imageUsage: VkImageUsageFlagBits_VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT as u32,
            imageSharingMode: VkSharingMode_VK_SHARING_MODE_EXCLUSIVE,
            queueFamilyIndexCount: 0,
            pQueueFamilyIndices: family_indices.as_mut_ptr(),
            preTransform: details.capabilities.currentTransform,
            compositeAlpha: VkCompositeAlphaFlagBitsKHR_VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR,
            presentMode: details.present_modes[0],
            clipped: VK_TRUE,
            oldSwapchain: self.swap_chain.ptr,
        };

        if queue_family_info.graphics_family_index != queue_family_info.present_family_index {
            family_indices.push(queue_family_info.graphics_family_index as _);
            family_indices.push(queue_family_info.present_family_index as _);
            swap_chain_create_info.imageSharingMode = VkSharingMode_VK_SHARING_MODE_CONCURRENT;
            swap_chain_create_info.queueFamilyIndexCount = 2;
            swap_chain_create_info.pQueueFamilyIndices = family_indices.as_mut_ptr();
        }

        self.swap_chain.ptr = unsafe {
            let mut output = ::std::mem::MaybeUninit::uninit();
            vkCreateSwapchainKHR.unwrap()(
                self.device,
                &swap_chain_create_info,
                ::std::ptr::null_mut(),
                output.as_mut_ptr(),
            );
            output.assume_init()
        };

        let mut swapchain_images_count = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            vkGetSwapchainImagesKHR.unwrap()(
                self.device,
                self.swap_chain.ptr,
                option.as_mut_ptr(),
                ::std::ptr::null_mut(),
            );
            option.assume_init()
        };

        let mut swap_chain_images = Vec::with_capacity(swapchain_images_count as usize);
        unsafe {
            swap_chain_images.set_len(swapchain_images_count as usize);
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkGetSwapchainImagesKHR.unwrap()(
                    self.device,
                    self.swap_chain.ptr,
                    &mut swapchain_images_count,
                    swap_chain_images.as_mut_ptr()
                )
            );
        }

        self.swap_chain.image_data = Vec::with_capacity(swapchain_images_count as usize);
        unsafe {
            self.swap_chain
                .image_data
                .set_len(swapchain_images_count as usize);
        }
        for (index, image) in swap_chain_images.into_iter().enumerate() {
            self.swap_chain.image_data[index].image = image;
        }

        let depth_format = find_depth_format(instance.get_physical_device());

        let (image, image_memory) = self.create_image(
            instance.get_physical_device(),
            (
                swap_chain_extent.width,
                swap_chain_extent.height,
                depth_format,
            ),
            VkImageTiling_VK_IMAGE_TILING_OPTIMAL,
            VkImageUsageFlagBits_VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT as _,
            VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT as _,
        );
        let image_view = create_image_view(
            self.device,
            image,
            depth_format,
            VkImageAspectFlagBits_VK_IMAGE_ASPECT_DEPTH_BIT as _,
        );
        self.swap_chain.depth_image_data.push(ImageViewData {
            image,
            image_view,
            image_memory,
        });

        self
    }

    fn create_image_views(&mut self, instance: &Instance) -> &mut Self {
        let selected_format = instance.get_swap_chain_info().formats[0].format;

        for image_data in self.swap_chain.image_data.iter_mut() {
            image_data.image_view = create_image_view(
                self.device,
                image_data.image,
                selected_format,
                VkImageAspectFlagBits_VK_IMAGE_ASPECT_COLOR_BIT as _,
            );
        }
        self
    }

    fn create_command_pool(&mut self, instance: &Instance) -> &mut Self {
        let command_pool_info = VkCommandPoolCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: VkCommandPoolCreateFlagBits_VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT as _,
            queueFamilyIndex: instance.get_queue_family_info().graphics_family_index as _,
        };

        let command_pool = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkCreateCommandPool.unwrap()(
                    self.device,
                    &command_pool_info,
                    ::std::ptr::null_mut(),
                    option.as_mut_ptr()
                )
            );
            option.assume_init()
        };
        self.command_pool = command_pool;
        self
    }

    fn create_sync_objects(&mut self) -> &mut Self {
        let mut image_available_semaphores: Vec<VkSemaphore> =
            Vec::with_capacity(MAX_FRAMES_IN_FLIGHT as usize);
        let mut render_finished_semaphores: Vec<VkSemaphore> =
            Vec::with_capacity(MAX_FRAMES_IN_FLIGHT as usize);
        let mut inflight_fences: Vec<VkFence> = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT as usize);
        let mut inflight_images: Vec<VkFence> =
            Vec::with_capacity(self.swap_chain.image_data.len());
        unsafe {
            image_available_semaphores.set_len(MAX_FRAMES_IN_FLIGHT as usize);
            render_finished_semaphores.set_len(MAX_FRAMES_IN_FLIGHT as usize);
            inflight_fences.set_len(MAX_FRAMES_IN_FLIGHT as usize);
            inflight_images.set_len(self.swap_chain.image_data.len());
        }
        let semaphore_create_info = VkSemaphoreCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
        };

        let fence_create_info = VkFenceCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_FENCE_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: VkFenceCreateFlagBits_VK_FENCE_CREATE_SIGNALED_BIT as _,
        };

        for i in 0..MAX_FRAMES_IN_FLIGHT {
            unsafe {
                assert_eq!(
                    VkResult_VK_SUCCESS,
                    vkCreateSemaphore.unwrap()(
                        self.device,
                        &semaphore_create_info,
                        ::std::ptr::null_mut(),
                        &mut image_available_semaphores[i as usize]
                    )
                );
                assert_eq!(
                    VkResult_VK_SUCCESS,
                    vkCreateSemaphore.unwrap()(
                        self.device,
                        &semaphore_create_info,
                        ::std::ptr::null_mut(),
                        &mut render_finished_semaphores[i as usize]
                    )
                );
                assert_eq!(
                    VkResult_VK_SUCCESS,
                    vkCreateFence.unwrap()(
                        self.device,
                        &fence_create_info,
                        ::std::ptr::null_mut(),
                        &mut inflight_fences[i as usize]
                    )
                );
            }
        }

        for i in 0..inflight_images.len() {
            inflight_images[i as usize] = ::std::ptr::null_mut();
        }

        self.image_available_semaphores = image_available_semaphores;
        self.render_finished_semaphores = render_finished_semaphores;
        self.inflight_fences = inflight_fences;
        self.inflight_images = inflight_images;
        self
    }

    fn destroy_image_views(&self) {
        for depth_data in self.swap_chain.depth_image_data.iter() {
            unsafe {
                vkDestroyImageView.unwrap()(
                    self.device,
                    depth_data.image_view,
                    ::std::ptr::null_mut(),
                );
                vkDestroyImage.unwrap()(self.device, depth_data.image, ::std::ptr::null_mut());
                vkFreeMemory.unwrap()(self.device, depth_data.image_memory, ::std::ptr::null_mut());
            }
        }

        for image_data in self.swap_chain.image_data.iter() {
            unsafe {
                vkDestroyImageView.unwrap()(
                    self.device,
                    image_data.image_view,
                    ::std::ptr::null_mut(),
                );
                vkDestroyImage.unwrap()(self.device, image_data.image, ::std::ptr::null_mut());
                vkFreeMemory.unwrap()(self.device, image_data.image_memory, ::std::ptr::null_mut());
            }
        }
    }

    fn wait_device(&mut self) {
        unsafe {
            for fence in self.inflight_fences.iter_mut() {
                vkWaitForFences.unwrap()(self.device, 1, fence, VK_TRUE, std::u64::MAX);
            }
            for fence in self.inflight_images.iter_mut() {
                vkWaitForFences.unwrap()(self.device, 1, fence, VK_TRUE, std::u64::MAX);
            }

            vkDeviceWaitIdle.unwrap()(self.device);
        }
    }
    fn cleanup_swap_chain(&mut self) {
        unsafe {
            self.wait_device();

            vkFreeCommandBuffers.unwrap()(
                self.device,
                self.command_pool,
                self.command_buffers.len() as _,
                self.command_buffers.as_ptr(),
            );

            self.destroy_image_views();

            vkDestroySwapchainKHR.unwrap()(
                self.device,
                self.swap_chain.ptr,
                ::std::ptr::null_mut(),
            );
        }
    }

    fn recreate_swap_chain(&mut self, instance: &Instance) {
        self.cleanup_swap_chain();

        instance.compute_swap_chain_details();

        self.create_swap_chain(instance)
            .create_image_views(instance)
            .allocate_command_buffers();
    }

    fn allocate_command_buffers(&mut self) -> &mut Self {
        let mut command_buffers =
            Vec::<VkCommandBuffer>::with_capacity(self.swap_chain.image_data.len());
        unsafe {
            command_buffers.set_len(self.swap_chain.image_data.len());
        }

        let command_alloc_info = VkCommandBufferAllocateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
            pNext: ::std::ptr::null_mut(),
            commandPool: self.command_pool,
            level: VkCommandBufferLevel_VK_COMMAND_BUFFER_LEVEL_PRIMARY,
            commandBufferCount: command_buffers.len() as _,
        };

        unsafe {
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkAllocateCommandBuffers.unwrap()(
                    self.device,
                    &command_alloc_info,
                    command_buffers.as_mut_ptr()
                )
            );
        }
        self.command_buffers = command_buffers;
        self
    }
}
