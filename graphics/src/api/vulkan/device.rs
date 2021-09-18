use crate::api::backend::{
    allocate_command_buffers, create_buffer, create_command_pool, create_image, create_image_view,
    destroy_buffer, find_depth_format, get_available_extensions_names, get_available_layers_names,
    get_minimum_required_vulkan_extensions, get_minimum_required_vulkan_layers, map_buffer_memory,
    BackendCommandBuffer, BackendInstance, BackendPhysicalDevice, BackendRenderPass,
};
use crate::{ConstantData, TextureAtlas, UniformData, MAX_TEXTURE_COUNT};
use nrg_math::matrix4_to_array;
use nrg_math::Matrix4;
use nrg_platform::{get_raw_thread_id, RawThreadId};
use std::collections::HashMap;
use std::os::raw::c_char;
use vulkan_bindings::*;

pub struct BackendSwapChain {
    ptr: VkSwapchainKHR,
    pub image_data: Vec<ImageViewData>,
    pub depth_image_data: Vec<ImageViewData>,
}

impl Default for BackendSwapChain {
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

struct ThreadData {
    command_pool: VkCommandPool,
    command_buffers: Vec<VkCommandBuffer>,
}

pub struct BackendDevice {
    device: VkDevice,
    transfers_queue: VkQueue,
    graphics_queue: VkQueue,
    present_queue: VkQueue,
    swap_chain: BackendSwapChain,
    pipeline_cache: VkPipelineCache,
    semaphore_id: usize,
    current_image_index: u32,
    semaphore_image_available: Vec<VkSemaphore>,
    semaphore_render_complete: Vec<VkSemaphore>,
    command_buffer_fences: Vec<VkFence>,
    thread_data: HashMap<RawThreadId, ThreadData>,
    graphics_family_index: u32,
    primary_command_pool: VkCommandPool,
    primary_command_buffers: Vec<VkCommandBuffer>,
    descriptor_set_layout: VkDescriptorSetLayout,
    descriptor_pool: VkDescriptorPool,
    descriptor_sets: Vec<VkDescriptorSet>,
    uniform_buffers_size: usize,
    uniform_buffers: Vec<VkBuffer>,
    uniform_buffers_memory: Vec<VkDeviceMemory>,
    pipeline_layout: VkPipelineLayout,
}

impl std::ops::Deref for BackendDevice {
    type Target = VkDevice;
    fn deref(&self) -> &Self::Target {
        &self.device
    }
}

impl std::ops::DerefMut for BackendDevice {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.device
    }
}

impl BackendDevice {
    pub fn new(instance: &BackendInstance, enable_validation: bool) -> Self {
        let physical_device = instance.get_physical_device();
        let queue_family_info = physical_device.get_queue_family_info();
        let queue_priority: f32 = 1.0;
        let mut hash_family_indices: ::std::collections::HashSet<u32> =
            ::std::collections::HashSet::new();
        hash_family_indices.insert(queue_family_info.graphics_family_index as _);
        hash_family_indices.insert(queue_family_info.present_family_index as _);

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
        let mut required_layers = get_minimum_required_vulkan_layers(enable_validation);
        for layer in layer_names_str.iter() {
            if let Some(index) = required_layers.iter().position(|l| l == layer) {
                required_layers.remove(index);
            }
        }
        let has_required_layers = required_layers.is_empty();
        debug_assert!(
            has_required_layers,
            "Device has not minimum requirement Vulkan layers"
        );
        required_layers = get_minimum_required_vulkan_layers(enable_validation);
        let layer_names_ptr = required_layers
            .iter()
            .map(|e| e.as_ptr())
            .collect::<Vec<*const c_char>>();

        let extension_names_str =
            get_available_extensions_names(physical_device.get_available_extensions());

        let mut required_exts = get_minimum_required_vulkan_extensions();
        for ext in extension_names_str.iter() {
            if let Some(index) = required_exts.iter().position(|r| r == ext) {
                required_exts.remove(index);
            }
        }
        let has_required_ext = required_exts.is_empty();
        debug_assert!(
            has_required_ext,
            "Device has not minimum requirement Vulkan extensions"
        );
        required_exts = get_minimum_required_vulkan_extensions();
        let extension_names_ptr = required_exts
            .iter()
            .map(|e| e.as_ptr() as *const c_char)
            .collect::<Vec<*const c_char>>();

        let mut device_features = physical_device.get_available_features();
        device_features.samplerAnisotropy = VK_TRUE;

        let device_create_info = VkDeviceCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            queueCreateInfoCount: queue_infos.len() as _,
            pQueueCreateInfos: queue_infos.as_ptr(),
            enabledLayerCount: layer_names_ptr.len() as _,
            ppEnabledLayerNames: layer_names_ptr.as_ptr(),
            enabledExtensionCount: extension_names_ptr.len() as u32,
            ppEnabledExtensionNames: extension_names_ptr.as_ptr(),
            pEnabledFeatures: &device_features,
        };

        let mut device: VkDevice = ::std::ptr::null_mut();
        unsafe {
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkCreateDevice.unwrap()(
                    **physical_device,
                    &device_create_info,
                    ::std::ptr::null_mut(),
                    &mut device
                )
            );
        }

        let transfers_queue: VkQueue = unsafe {
            let mut output = ::std::mem::MaybeUninit::uninit();
            vkGetDeviceQueue.unwrap()(
                device,
                queue_family_info.transfers_family_index as _,
                0,
                output.as_mut_ptr(),
            );
            output.assume_init()
        };
        let graphics_queue: VkQueue = unsafe {
            let mut output = ::std::mem::MaybeUninit::uninit();
            vkGetDeviceQueue.unwrap()(
                device,
                queue_family_info.graphics_family_index as _,
                0,
                output.as_mut_ptr(),
            );
            output.assume_init()
        };
        let present_queue: VkQueue = unsafe {
            let mut output = ::std::mem::MaybeUninit::uninit();
            vkGetDeviceQueue.unwrap()(
                device,
                queue_family_info.present_family_index as _,
                0,
                output.as_mut_ptr(),
            );
            output.assume_init()
        };

        let pipeline_cache_info = VkPipelineCacheCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_PIPELINE_CACHE_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            initialDataSize: 0,
            pInitialData: ::std::ptr::null_mut(),
        };
        let pipeline_cache: VkPipelineCache = unsafe {
            let mut output = ::std::mem::MaybeUninit::uninit();
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkCreatePipelineCache.unwrap()(
                    device,
                    &pipeline_cache_info,
                    ::std::ptr::null_mut(),
                    output.as_mut_ptr(),
                )
            );
            output.assume_init()
        };

        let mut inner_device = Self {
            device,
            transfers_queue,
            graphics_queue,
            present_queue,
            swap_chain: BackendSwapChain::default(),
            pipeline_cache,
            semaphore_id: 0,
            current_image_index: 0,
            semaphore_image_available: Vec::new(),
            semaphore_render_complete: Vec::new(),
            command_buffer_fences: Vec::new(),
            graphics_family_index: queue_family_info.graphics_family_index as _,
            thread_data: HashMap::new(),
            primary_command_pool: ::std::ptr::null_mut(),
            primary_command_buffers: Vec::new(),
            descriptor_set_layout: ::std::ptr::null_mut(),
            descriptor_sets: Vec::new(),
            descriptor_pool: ::std::ptr::null_mut(),
            uniform_buffers_size: 0,
            uniform_buffers: Vec::new(),
            uniform_buffers_memory: Vec::new(),
            pipeline_layout: ::std::ptr::null_mut(),
        };
        inner_device
            .create_swap_chain(physical_device, instance.get_surface())
            .create_image_views(physical_device)
            .create_sync_objects()
            .create_descriptor_set_layout()
            .create_uniform_buffers(physical_device)
            .create_descriptor_pool()
            .create_descriptor_sets()
            .create_pipeline_layout();

        inner_device.primary_command_pool =
            create_command_pool(device, queue_family_info.graphics_family_index as _);
        inner_device.primary_command_buffers = allocate_command_buffers(
            inner_device.device,
            inner_device.primary_command_pool,
            VkCommandBufferLevel_VK_COMMAND_BUFFER_LEVEL_PRIMARY,
            inner_device.swap_chain.image_data.len() as _,
        );
        inner_device.acquire_command_buffer();

        inner_device
    }

    pub fn get_pipeline_cache(&self) -> VkPipelineCache {
        self.pipeline_cache
    }

    pub fn get_images_count(&self) -> usize {
        self.swap_chain.image_data.len()
    }

    pub fn get_image(&self, index: usize) -> VkImage {
        self.swap_chain.image_data[index].image
    }

    pub fn get_image_view(&self, index: usize) -> VkImageView {
        self.swap_chain.image_data[index].image_view
    }

    pub fn get_depth_image_view(&self, index: usize) -> VkImageView {
        self.swap_chain.depth_image_data[index].image_view
    }

    pub fn get_current_image_index(&self) -> usize {
        self.current_image_index as _
    }

    pub fn get_descriptor_set_layout(&self) -> VkDescriptorSetLayout {
        self.descriptor_set_layout
    }

    pub fn get_pipeline_layout(&self) -> VkPipelineLayout {
        self.pipeline_layout
    }

    pub fn acquire_command_buffer(&mut self) -> VkCommandBuffer {
        self.create_thread_data().get_current_command_buffer()
    }

    pub fn delete(&mut self) {
        unsafe {
            let count = self.swap_chain.image_data.len();
            self.cleanup_swap_chain();

            self.thread_data.iter().for_each(|(_, t)| {
                vkFreeCommandBuffers.unwrap()(
                    self.device,
                    t.command_pool,
                    t.command_buffers.len() as _,
                    t.command_buffers.as_ptr(),
                );
                vkDestroyCommandPool.unwrap()(self.device, t.command_pool, ::std::ptr::null_mut());
            });

            for i in 0..self.uniform_buffers.len() {
                destroy_buffer(
                    self,
                    &self.uniform_buffers[i],
                    &self.uniform_buffers_memory[i],
                );
            }

            vkDestroyDescriptorSetLayout.unwrap()(
                self.device,
                self.descriptor_set_layout,
                ::std::ptr::null_mut(),
            );
            vkDestroyPipelineLayout.unwrap()(
                self.device,
                self.pipeline_layout,
                ::std::ptr::null_mut(),
            );

            for i in 0..count {
                vkDestroySemaphore.unwrap()(
                    self.device,
                    self.semaphore_image_available[i as usize],
                    ::std::ptr::null_mut(),
                );
                vkDestroySemaphore.unwrap()(
                    self.device,
                    self.semaphore_render_complete[i as usize],
                    ::std::ptr::null_mut(),
                );
                vkDestroyFence.unwrap()(
                    self.device,
                    self.command_buffer_fences[i as usize],
                    ::std::ptr::null_mut(),
                );
            }

            vkDestroyDevice.unwrap()(self.device, ::std::ptr::null_mut());
        }
    }

    pub fn acquire_image(&mut self) -> bool {
        unsafe {
            vkQueueWaitIdle.unwrap()(self.transfers_queue);
            vkQueueWaitIdle.unwrap()(self.graphics_queue);
            vkQueueWaitIdle.unwrap()(self.present_queue);

            self.semaphore_id = (self.semaphore_id + 1) % self.semaphore_image_available.len();

            let result = vkAcquireNextImageKHR.unwrap()(
                self.device,
                self.swap_chain.ptr,
                ::std::u64::MAX,
                self.semaphore_image_available[self.semaphore_id],
                ::std::ptr::null_mut(),
                &mut self.current_image_index,
            );
            if result == VkResult_VK_ERROR_OUT_OF_DATE_KHR {
                self.wait_device();
                self.semaphore_id = (self.semaphore_id - 1) % self.semaphore_image_available.len();
                return false;
            } else if result != VkResult_VK_SUCCESS && result != VkResult_VK_SUBOPTIMAL_KHR {
                eprintln!("Failed to acquire swap chain image");
                self.semaphore_id = (self.semaphore_id - 1) % self.semaphore_image_available.len();
                return false;
            }
        }

        true
    }
    pub fn begin_primary_command_buffer(&mut self) {
        let primary_command_buffer = self.get_primary_command_buffer();

        let flags = VkCommandBufferUsageFlagBits_VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT;
        let begin_info = VkCommandBufferBeginInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
            flags: flags as _,
            pNext: ::std::ptr::null_mut(),
            pInheritanceInfo: ::std::ptr::null_mut(),
        };
        unsafe {
            vkQueueWaitIdle.unwrap()(self.transfers_queue);
            vkQueueWaitIdle.unwrap()(self.graphics_queue);

            assert_eq!(
                VkResult_VK_SUCCESS,
                vkBeginCommandBuffer.unwrap()(primary_command_buffer, &begin_info)
            );
        }
    }

    pub fn begin_command_buffer(
        &self,
        command_buffer: &BackendCommandBuffer,
        render_pass: &BackendRenderPass,
    ) {
        let flags = VkCommandBufferUsageFlagBits_VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT
            | VkCommandBufferUsageFlagBits_VK_COMMAND_BUFFER_USAGE_RENDER_PASS_CONTINUE_BIT;
        let inheritance_info = VkCommandBufferInheritanceInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_COMMAND_BUFFER_INHERITANCE_INFO,
            pNext: ::std::ptr::null_mut(),
            renderPass: **render_pass,
            subpass: 0,
            framebuffer: render_pass.get_framebuffer(self.current_image_index as _),
            occlusionQueryEnable: VK_FALSE,
            queryFlags: 0,
            pipelineStatistics: 0,
        };
        let begin_info = VkCommandBufferBeginInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
            flags: flags as _,
            pNext: ::std::ptr::null_mut(),
            pInheritanceInfo: &inheritance_info,
        };
        unsafe {
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkBeginCommandBuffer.unwrap()(command_buffer.get(), &begin_info)
            );

            let viewport = VkViewport {
                x: 0.0,
                y: 0.0,
                width: render_pass.get_framebuffer_width() as _,
                height: render_pass.get_framebuffer_height() as _,
                minDepth: 0.0,
                maxDepth: 1.0,
            };

            let scissors = VkRect2D {
                offset: VkOffset2D { x: 0, y: 0 },
                extent: render_pass.get_extent(),
            };

            vkCmdSetViewport.unwrap()(command_buffer.get(), 0, 1, &viewport);
            vkCmdSetScissor.unwrap()(command_buffer.get(), 0, 1, &scissors);
        }
    }

    pub fn end_primary_command_buffer(&self) {
        let primary_command_buffer = self.get_primary_command_buffer();
        unsafe {
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkEndCommandBuffer.unwrap()(primary_command_buffer)
            );
        }
    }

    pub fn end_command_buffer(&self, command_buffer: &BackendCommandBuffer) {
        unsafe {
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkEndCommandBuffer.unwrap()(command_buffer.get())
            );
        }
    }

    pub fn graphics_queue_submit(&self, command_buffer: VkCommandBuffer) {
        unsafe {
            vkQueueWaitIdle.unwrap()(self.transfers_queue);

            let command_buffers = vec![command_buffer];
            let wait_stages = [
                VkPipelineStageFlagBits_VK_PIPELINE_STAGE_ALL_GRAPHICS_BIT as _,
                VkPipelineStageFlagBits_VK_PIPELINE_STAGE_ALL_COMMANDS_BIT as _,
            ];
            let submit_info = VkSubmitInfo {
                sType: VkStructureType_VK_STRUCTURE_TYPE_SUBMIT_INFO,
                pNext: ::std::ptr::null_mut(),
                waitSemaphoreCount: 1,
                pWaitSemaphores: &self.semaphore_image_available[self.semaphore_id],
                pWaitDstStageMask: wait_stages.as_ptr(),
                commandBufferCount: command_buffers.len() as _,
                pCommandBuffers: command_buffers.as_ptr(),
                signalSemaphoreCount: 1,
                pSignalSemaphores: &self.semaphore_render_complete[self.semaphore_id],
            };

            vkWaitForFences.unwrap()(
                self.device,
                1,
                &self.command_buffer_fences[self.semaphore_id],
                VK_TRUE,
                std::u64::MAX,
            );
            vkResetFences.unwrap()(
                self.device,
                1,
                &self.command_buffer_fences[self.semaphore_id],
            );

            let submit_result = vkQueueSubmit.unwrap()(
                self.graphics_queue,
                1,
                &submit_info,
                self.command_buffer_fences[self.semaphore_id],
            );
            if submit_result != VkResult_VK_SUCCESS {
                eprintln!("Unable to submit queue correctly on GPU");
            }
        }
    }

    pub fn present(&mut self) -> bool {
        unsafe {
            vkQueueWaitIdle.unwrap()(self.graphics_queue);

            let present_info = VkPresentInfoKHR {
                sType: VkStructureType_VK_STRUCTURE_TYPE_PRESENT_INFO_KHR,
                pNext: ::std::ptr::null_mut(),
                waitSemaphoreCount: 1,
                pWaitSemaphores: &self.semaphore_render_complete[self.semaphore_id],
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
                return false;
            }
        }

        true
    }

    pub fn get_transfers_queue(&self) -> VkQueue {
        self.transfers_queue
    }
    pub fn get_graphics_queue(&self) -> VkQueue {
        self.graphics_queue
    }
    pub fn get_primary_command_pool(&self) -> VkCommandPool {
        self.primary_command_pool
    }
    pub fn get_primary_command_buffer(&self) -> VkCommandBuffer {
        self.primary_command_buffers[self.current_image_index as usize]
    }
    pub fn get_current_command_pool(&self) -> VkCommandPool {
        let thread_data = self.get_thread_data();
        thread_data.command_pool
    }
    pub fn get_current_command_buffer(&self) -> VkCommandBuffer {
        let buffer_index = self.current_image_index as usize;
        let thread_data = self.get_thread_data();
        thread_data.command_buffers[buffer_index]
    }

    fn create_thread_data(&mut self) -> &Self {
        let thread_id = get_raw_thread_id();
        let device = self.device;
        let graphics_family_index = self.graphics_family_index;
        let num_frames = self.swap_chain.image_data.len() as _;
        self.thread_data.entry(thread_id).or_insert_with(|| {
            let command_pool = create_command_pool(device, graphics_family_index);
            let command_buffers = allocate_command_buffers(
                device,
                command_pool,
                VkCommandBufferLevel_VK_COMMAND_BUFFER_LEVEL_SECONDARY,
                num_frames,
            );
            ThreadData {
                command_pool,
                command_buffers,
            }
        });
        self
    }
    fn get_thread_data(&self) -> &ThreadData {
        let thread_id = get_raw_thread_id();
        &self.thread_data[&thread_id]
    }

    fn create_swap_chain(
        &mut self,
        physical_device: &BackendPhysicalDevice,
        surface: VkSurfaceKHR,
    ) -> &mut Self {
        let details = physical_device.get_swap_chain_info();
        let queue_family_info = physical_device.get_queue_family_info();
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
        )
        .max(1);
        swap_chain_extent.height = ::std::cmp::max(
            details.capabilities.minImageExtent.height,
            std::cmp::min(
                details.capabilities.maxImageExtent.height,
                swap_chain_extent.height,
            ),
        )
        .max(1);

        let format_index = if let Some(index) = details
            .formats
            .iter()
            .position(|f| f.format == VkFormat_VK_FORMAT_R8G8B8A8_UNORM)
        {
            index
        } else {
            0
        };
        let mut swap_chain_create_info = VkSwapchainCreateInfoKHR {
            sType: VkStructureType_VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            surface,
            minImageCount: ::std::cmp::min(
                details.capabilities.minImageCount + 1,
                details.capabilities.maxImageCount,
            ),
            imageFormat: details.formats[format_index].format,
            imageColorSpace: details.formats[format_index].colorSpace,
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

        self.swap_chain.depth_image_data.clear();

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

        let depth_format = find_depth_format(**physical_device);

        let (image, image_memory) = create_image(
            self,
            physical_device,
            (
                swap_chain_extent.width,
                swap_chain_extent.height,
                depth_format,
            ),
            VkImageTiling_VK_IMAGE_TILING_OPTIMAL,
            VkImageUsageFlagBits_VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT as _,
            VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT as _,
            1,
        );
        let image_view = create_image_view(
            self.device,
            image,
            depth_format,
            if depth_format >= VkFormat_VK_FORMAT_D16_UNORM_S8_UINT {
                (VkImageAspectFlagBits_VK_IMAGE_ASPECT_DEPTH_BIT
                    | VkImageAspectFlagBits_VK_IMAGE_ASPECT_STENCIL_BIT) as _
            } else {
                VkImageAspectFlagBits_VK_IMAGE_ASPECT_DEPTH_BIT as _
            },
            1,
        );
        self.swap_chain.depth_image_data.push(ImageViewData {
            image,
            image_view,
            image_memory,
        });

        self
    }

    fn create_image_views(&mut self, physical_device: &BackendPhysicalDevice) -> &mut Self {
        let selected_format = physical_device.get_swap_chain_info().get_preferred_format();
        let images = &mut self.swap_chain.image_data;
        for image_data in images.iter_mut() {
            image_data.image_view = create_image_view(
                self.device,
                image_data.image,
                selected_format,
                VkImageAspectFlagBits_VK_IMAGE_ASPECT_COLOR_BIT as _,
                1,
            );
        }
        self
    }
    fn create_sync_objects(&mut self) -> &mut Self {
        let count = self.swap_chain.image_data.len();

        self.semaphore_image_available = Vec::with_capacity(count);
        self.semaphore_render_complete = Vec::with_capacity(count);
        self.command_buffer_fences = Vec::with_capacity(count);
        unsafe {
            self.semaphore_image_available.set_len(count);
            self.semaphore_render_complete.set_len(count);
            self.command_buffer_fences.set_len(count);
        }

        let semaphore_create_info = VkSemaphoreCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: VkSemaphoreType_VK_SEMAPHORE_TYPE_BINARY as _,
        };
        let fence_create_info = VkFenceCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_FENCE_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: VkFenceCreateFlagBits_VK_FENCE_CREATE_SIGNALED_BIT as _,
        };

        unsafe {
            for i in 0..count {
                assert_eq!(
                    VkResult_VK_SUCCESS,
                    vkCreateSemaphore.unwrap()(
                        self.device,
                        &semaphore_create_info,
                        ::std::ptr::null_mut(),
                        &mut self.semaphore_image_available[i as usize]
                    )
                );
                assert_eq!(
                    VkResult_VK_SUCCESS,
                    vkCreateSemaphore.unwrap()(
                        self.device,
                        &semaphore_create_info,
                        ::std::ptr::null_mut(),
                        &mut self.semaphore_render_complete[i as usize]
                    )
                );
                assert_eq!(
                    VkResult_VK_SUCCESS,
                    vkCreateFence.unwrap()(
                        self.device,
                        &fence_create_info,
                        ::std::ptr::null_mut(),
                        &mut self.command_buffer_fences[i as usize]
                    )
                );
            }
        }

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
            }
        }
    }

    fn wait_device(&mut self) {
        unsafe {
            vkDeviceWaitIdle.unwrap()(self.device);
        }
    }
    fn cleanup_swap_chain(&mut self) {
        unsafe {
            self.wait_device();

            self.destroy_image_views();

            vkDestroySwapchainKHR.unwrap()(
                self.device,
                self.swap_chain.ptr,
                ::std::ptr::null_mut(),
            );
        }
    }

    pub fn recreate_swap_chain(
        &mut self,
        physical_device: &mut BackendPhysicalDevice,
        surface: VkSurfaceKHR,
    ) {
        self.cleanup_swap_chain();

        physical_device.compute_swap_chain_details(surface);

        self.create_swap_chain(physical_device, surface)
            .create_image_views(physical_device);
    }

    fn create_uniform_buffers(&mut self, physical_device: &BackendPhysicalDevice) -> &mut Self {
        let images_count = self.swap_chain.image_data.len();
        let mut uniform_buffers = Vec::<VkBuffer>::with_capacity(images_count);
        let mut uniform_buffers_memory = Vec::<VkDeviceMemory>::with_capacity(images_count);
        unsafe {
            uniform_buffers.set_len(images_count);
            uniform_buffers_memory.set_len(images_count);
        }

        let uniform_buffers_size = std::mem::size_of::<UniformData>();
        let flags = VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT
            | VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_HOST_COHERENT_BIT;
        for i in 0..uniform_buffers.len() {
            create_buffer(
                self,
                physical_device,
                uniform_buffers_size as _,
                VkBufferUsageFlagBits_VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT as _,
                flags as _,
                &mut uniform_buffers[i],
                &mut uniform_buffers_memory[i],
            );
        }

        self.uniform_buffers_size = uniform_buffers_size;
        self.uniform_buffers = uniform_buffers;
        self.uniform_buffers_memory = uniform_buffers_memory;
        self
    }
    fn create_descriptor_pool(&mut self) -> &mut Self {
        let images_count = self.swap_chain.image_data.len();
        let pool_sizes: Vec<VkDescriptorPoolSize> = vec![
            VkDescriptorPoolSize {
                type_: VkDescriptorType_VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
                descriptorCount: images_count as u32,
            },
            VkDescriptorPoolSize {
                type_: VkDescriptorType_VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
                descriptorCount: MAX_TEXTURE_COUNT as u32 * images_count as u32,
            },
        ];

        let pool_info = VkDescriptorPoolCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO,
            flags: 0,
            pNext: ::std::ptr::null_mut(),
            poolSizeCount: pool_sizes.len() as _,
            pPoolSizes: pool_sizes.as_ptr(),
            maxSets: images_count as _,
        };

        self.descriptor_pool = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkCreateDescriptorPool.unwrap()(
                    self.device,
                    &pool_info,
                    ::std::ptr::null_mut(),
                    option.as_mut_ptr()
                )
            );
            option.assume_init()
        };
        self
    }
    pub fn create_descriptor_sets(&mut self) -> &mut Self {
        let images_count = self.swap_chain.image_data.len();
        let mut layouts = Vec::<VkDescriptorSetLayout>::with_capacity(images_count);
        unsafe {
            layouts.set_len(images_count);
        }
        for layout in layouts.iter_mut() {
            *layout = self.descriptor_set_layout;
        }

        let alloc_info = VkDescriptorSetAllocateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO,
            pNext: ::std::ptr::null_mut(),
            descriptorPool: self.descriptor_pool,
            descriptorSetCount: images_count as _,
            pSetLayouts: layouts.as_mut_ptr(),
        };

        let mut descriptor_sets = Vec::<VkDescriptorSet>::with_capacity(images_count);
        unsafe {
            descriptor_sets.set_len(images_count);
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkAllocateDescriptorSets.unwrap()(
                    self.device,
                    &alloc_info,
                    descriptor_sets.as_mut_ptr()
                )
            );
        }

        self.descriptor_sets = descriptor_sets;
        self
    }

    fn create_pipeline_layout(&mut self) -> &mut Self {
        let push_constant_range = VkPushConstantRange {
            stageFlags: VkShaderStageFlagBits_VK_SHADER_STAGE_ALL_GRAPHICS as _,
            offset: 0,
            size: ::std::mem::size_of::<ConstantData>() as _,
        };

        let pipeline_layout_info = VkPipelineLayoutCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            setLayoutCount: 1,
            pSetLayouts: &self.descriptor_set_layout,
            pushConstantRangeCount: 1,
            pPushConstantRanges: &push_constant_range,
        };

        self.pipeline_layout = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkCreatePipelineLayout.unwrap()(
                    self.device,
                    &pipeline_layout_info,
                    ::std::ptr::null_mut(),
                    option.as_mut_ptr()
                )
            );
            option.assume_init()
        };
        self
    }

    fn create_descriptor_set_layout(&mut self) -> &mut Self {
        let bindings = vec![
            VkDescriptorSetLayoutBinding {
                binding: 0,
                descriptorCount: 1,
                descriptorType: VkDescriptorType_VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
                pImmutableSamplers: ::std::ptr::null_mut(),
                stageFlags: (VkShaderStageFlagBits_VK_SHADER_STAGE_VERTEX_BIT
                    | VkShaderStageFlagBits_VK_SHADER_STAGE_FRAGMENT_BIT)
                    as _,
            },
            VkDescriptorSetLayoutBinding {
                binding: 1,
                descriptorCount: MAX_TEXTURE_COUNT as _,
                descriptorType: VkDescriptorType_VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
                pImmutableSamplers: ::std::ptr::null_mut(),
                stageFlags: VkShaderStageFlagBits_VK_SHADER_STAGE_FRAGMENT_BIT as _,
            },
        ];

        let layout_create_info = VkDescriptorSetLayoutCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            flags: 0,
            pNext: ::std::ptr::null_mut(),
            bindingCount: bindings.len() as _,
            pBindings: bindings.as_ptr(),
        };

        self.descriptor_set_layout = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkCreateDescriptorSetLayout.unwrap()(
                    self.device,
                    &layout_create_info,
                    ::std::ptr::null_mut(),
                    option.as_mut_ptr()
                )
            );
            option.assume_init()
        };

        self
    }

    pub fn update_constant_data(
        &self,
        command_buffer: &BackendCommandBuffer,
        width: u32,
        height: u32,
        view: &Matrix4,
        proj: &Matrix4,
    ) -> &Self {
        let constant_data = ConstantData {
            view: matrix4_to_array(*view),
            proj: matrix4_to_array(*proj),
            screen_width: width as _,
            screen_height: height as _,
        };

        unsafe {
            vkCmdPushConstants.unwrap()(
                command_buffer.get(),
                self.pipeline_layout,
                VkShaderStageFlagBits_VK_SHADER_STAGE_ALL_GRAPHICS as _,
                0,
                ::std::mem::size_of::<ConstantData>() as _,
                &constant_data as *const ConstantData as _,
            );
        }
        self
    }

    pub fn update_uniform_buffer(&self, view: &Matrix4, proj: &Matrix4) -> &Self {
        let image_index = self.get_current_image_index();
        let uniform_data: [UniformData; 1] = [UniformData {
            view: *view,
            proj: *proj,
        }];

        let mut buffer_memory = self.uniform_buffers_memory[image_index];
        map_buffer_memory(self, &mut buffer_memory, 0, &uniform_data);

        let buffer_info = VkDescriptorBufferInfo {
            buffer: self.uniform_buffers[image_index],
            offset: 0,
            range: self.uniform_buffers_size as _,
        };

        let descriptor_write: Vec<VkWriteDescriptorSet> = vec![VkWriteDescriptorSet {
            sType: VkStructureType_VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
            pNext: ::std::ptr::null_mut(),
            dstSet: self.descriptor_sets[image_index],
            dstBinding: 0,
            dstArrayElement: 0,
            descriptorCount: 1,
            descriptorType: VkDescriptorType_VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
            pImageInfo: ::std::ptr::null_mut(),
            pBufferInfo: &buffer_info,
            pTexelBufferView: ::std::ptr::null_mut(),
        }];

        unsafe {
            vkUpdateDescriptorSets.unwrap()(
                self.device,
                descriptor_write.len() as _,
                descriptor_write.as_ptr(),
                0,
                ::std::ptr::null_mut(),
            );
        }
        self
    }

    pub fn update_descriptor_sets(&self, textures: &[TextureAtlas]) -> &Self {
        debug_assert!(
            !textures.is_empty(),
            "At least one texture should be received"
        );
        debug_assert!(
            textures.len() <= MAX_TEXTURE_COUNT,
            "Max num textures exceeded"
        );

        let image_index = self.get_current_image_index();

        let mut descriptor_write: Vec<VkWriteDescriptorSet> = Vec::new();
        let mut descriptors = Vec::new();
        for t in textures.iter() {
            descriptors.push(t.get_texture().get_descriptor());
        }
        if descriptors.len() < MAX_TEXTURE_COUNT {
            let descriptors_to_add = MAX_TEXTURE_COUNT - descriptors.len();
            for _i in 0..descriptors_to_add {
                descriptors.push(textures[0].get_texture().get_descriptor());
            }
        }
        descriptor_write.push(VkWriteDescriptorSet {
            sType: VkStructureType_VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
            pNext: ::std::ptr::null_mut(),
            dstSet: self.descriptor_sets[image_index],
            dstBinding: 1,
            dstArrayElement: 0,
            descriptorCount: MAX_TEXTURE_COUNT as _,
            descriptorType: VkDescriptorType_VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
            pImageInfo: descriptors.as_ptr(),
            pBufferInfo: ::std::ptr::null_mut(),
            pTexelBufferView: ::std::ptr::null_mut(),
        });

        unsafe {
            vkUpdateDescriptorSets.unwrap()(
                self.device,
                descriptor_write.len() as _,
                descriptor_write.as_ptr(),
                0,
                ::std::ptr::null_mut(),
            );
        }
        self
    }

    pub fn bind_descriptors(&self, command_buffer: &BackendCommandBuffer) -> &Self {
        let image_index = self.get_current_image_index();

        unsafe {
            vkCmdBindDescriptorSets.unwrap()(
                command_buffer.get(),
                VkPipelineBindPoint_VK_PIPELINE_BIND_POINT_GRAPHICS,
                self.pipeline_layout,
                0,
                1,
                &self.descriptor_sets[image_index],
                0,
                ::std::ptr::null_mut(),
            );
        }
        self
    }
}
