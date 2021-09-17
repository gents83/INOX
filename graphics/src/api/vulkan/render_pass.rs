use super::device::*;
use super::utils::*;
use super::BackendTexture;
use crate::api::backend::physical_device::BackendPhysicalDevice;
use crate::common::data_formats::*;
use vulkan_bindings::*;

pub struct BackendRenderPass {
    render_pass: VkRenderPass,
    framebuffer: Vec<VkFramebuffer>,
    extent: VkExtent2D,
}

impl std::ops::Deref for BackendRenderPass {
    type Target = VkRenderPass;
    fn deref(&self) -> &Self::Target {
        &self.render_pass
    }
}

unsafe impl Send for BackendRenderPass {}
unsafe impl Sync for BackendRenderPass {}

impl BackendRenderPass {
    pub fn get_extent(&self) -> VkExtent2D {
        self.extent
    }
    pub fn get_framebuffer_width(&self) -> u32 {
        self.extent.width
    }
    pub fn get_framebuffer_height(&self) -> u32 {
        self.extent.height
    }
    pub fn get_framebuffer(&self, current_image_index: usize) -> VkFramebuffer {
        self.framebuffer[current_image_index]
    }

    pub fn create_default(
        device: &BackendDevice,
        physical_device: &BackendPhysicalDevice,
        data: &RenderPassData,
        color: Option<&BackendTexture>,
        depth: Option<&BackendTexture>,
    ) -> Self {
        let extent = if let Some(color) = color {
            VkExtent2D {
                width: color.width(),
                height: color.height(),
            }
        } else if let Some(depth) = depth {
            VkExtent2D {
                width: depth.width(),
                height: depth.height(),
            }
        } else {
            physical_device
                .get_swap_chain_info()
                .capabilities
                .currentExtent
        };
        let mut render_pass = Self {
            render_pass: Self::base_pass(device, physical_device, data),
            framebuffer: Vec::new(),
            extent,
        };
        render_pass.create_framebuffers(device, physical_device, color, depth);
        render_pass
    }

    pub fn destroy(&mut self, device: &BackendDevice) {
        unsafe {
            for framebuffer in self.framebuffer.iter() {
                vkDestroyFramebuffer.unwrap()(**device, *framebuffer, ::std::ptr::null_mut());
            }
            vkDestroyRenderPass.unwrap()(**device, self.render_pass, ::std::ptr::null_mut());
        }
    }

    pub fn begin(&self, device: &BackendDevice) {
        let clear_value = [
            VkClearValue {
                color: VkClearColorValue {
                    float32: [0.2, 0.2, 0.2, 1.0],
                },
            },
            VkClearValue {
                depthStencil: VkClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];
        let area = VkRect2D {
            offset: VkOffset2D { x: 0, y: 0 },
            extent: self.extent,
        };
        let render_pass_begin_info = VkRenderPassBeginInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO,
            pNext: ::std::ptr::null_mut(),
            renderPass: self.render_pass,
            framebuffer: self.framebuffer[device.get_current_image_index()],
            renderArea: area,
            clearValueCount: clear_value.len() as _,
            pClearValues: clear_value.as_ptr(),
        };
        unsafe {
            vkCmdBeginRenderPass.unwrap()(
                device.get_primary_command_buffer(),
                &render_pass_begin_info,
                VkSubpassContents_VK_SUBPASS_CONTENTS_SECONDARY_COMMAND_BUFFERS,
            );
        }
    }

    pub fn end(&self, device: &BackendDevice) {
        unsafe {
            vkCmdEndRenderPass.unwrap()(device.get_primary_command_buffer());
        }
    }

    fn base_pass(
        device: &BackendDevice,
        physical_device: &BackendPhysicalDevice,
        data: &RenderPassData,
    ) -> VkRenderPass {
        let details = physical_device.get_swap_chain_info();
        let color_attachment = VkAttachmentDescription {
            flags: 0,
            format: if data.render_to_texture {
                VkFormat_VK_FORMAT_R8G8B8A8_UNORM
            } else {
                details.get_preferred_format()
            },
            samples: VkSampleCountFlagBits_VK_SAMPLE_COUNT_1_BIT,
            loadOp: match data.load_color {
                LoadOperation::Clear => VkAttachmentLoadOp_VK_ATTACHMENT_LOAD_OP_CLEAR,
                LoadOperation::Load => VkAttachmentLoadOp_VK_ATTACHMENT_LOAD_OP_LOAD,
                _ => VkAttachmentLoadOp_VK_ATTACHMENT_LOAD_OP_DONT_CARE,
            },
            storeOp: match data.store_color {
                StoreOperation::Store => VkAttachmentStoreOp_VK_ATTACHMENT_STORE_OP_STORE,
                _ => VkAttachmentStoreOp_VK_ATTACHMENT_STORE_OP_DONT_CARE,
            },
            stencilLoadOp: VkAttachmentLoadOp_VK_ATTACHMENT_LOAD_OP_CLEAR,
            stencilStoreOp: VkAttachmentStoreOp_VK_ATTACHMENT_STORE_OP_DONT_CARE,
            initialLayout: VkImageLayout_VK_IMAGE_LAYOUT_UNDEFINED,
            finalLayout: if data.render_to_texture {
                VkImageLayout_VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL
            } else {
                VkImageLayout_VK_IMAGE_LAYOUT_PRESENT_SRC_KHR
            },
        };

        let color_attachment_ref = VkAttachmentReference {
            attachment: 0,
            layout: VkImageLayout_VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL,
        };

        let depth_attachment = VkAttachmentDescription {
            flags: 0,
            format: find_depth_format(**physical_device),
            samples: VkSampleCountFlagBits_VK_SAMPLE_COUNT_1_BIT,
            loadOp: match data.load_depth {
                LoadOperation::Clear => VkAttachmentLoadOp_VK_ATTACHMENT_LOAD_OP_CLEAR,
                LoadOperation::Load => VkAttachmentLoadOp_VK_ATTACHMENT_LOAD_OP_LOAD,
                _ => VkAttachmentLoadOp_VK_ATTACHMENT_LOAD_OP_DONT_CARE,
            },
            storeOp: match data.store_depth {
                StoreOperation::Store => VkAttachmentStoreOp_VK_ATTACHMENT_STORE_OP_STORE,
                _ => VkAttachmentStoreOp_VK_ATTACHMENT_STORE_OP_DONT_CARE,
            },
            stencilLoadOp: VkAttachmentLoadOp_VK_ATTACHMENT_LOAD_OP_CLEAR,
            stencilStoreOp: VkAttachmentStoreOp_VK_ATTACHMENT_STORE_OP_DONT_CARE,
            initialLayout: VkImageLayout_VK_IMAGE_LAYOUT_UNDEFINED,
            finalLayout: VkImageLayout_VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };

        let depth_attachment_ref = VkAttachmentReference {
            attachment: 1,
            layout: VkImageLayout_VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };

        let subpass1 = VkSubpassDescription {
            flags: 0,
            pipelineBindPoint: VkPipelineBindPoint_VK_PIPELINE_BIND_POINT_GRAPHICS,
            inputAttachmentCount: 0,
            pInputAttachments: ::std::ptr::null_mut(),
            colorAttachmentCount: 1,
            pColorAttachments: &color_attachment_ref,
            pResolveAttachments: ::std::ptr::null_mut(),
            pDepthStencilAttachment: &depth_attachment_ref,
            preserveAttachmentCount: 0,
            pPreserveAttachments: ::std::ptr::null_mut(),
        };

        let subpass_dependency = [
            VkSubpassDependency {
                srcSubpass: VK_SUBPASS_EXTERNAL as _,
                dstSubpass: 0,
                srcStageMask: VkPipelineStageFlagBits_VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT as _,
                dstStageMask: VkPipelineStageFlagBits_VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT
                    as _,
                srcAccessMask: VkAccessFlagBits_VK_ACCESS_SHADER_READ_BIT as _,
                dstAccessMask: VkAccessFlagBits_VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT as _,
                dependencyFlags: VkDependencyFlagBits_VK_DEPENDENCY_BY_REGION_BIT as _,
            },
            VkSubpassDependency {
                srcSubpass: 0,
                dstSubpass: VK_SUBPASS_EXTERNAL as _,
                srcStageMask: VkPipelineStageFlagBits_VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT
                    as _,
                dstStageMask: VkPipelineStageFlagBits_VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT as _,
                srcAccessMask: VkAccessFlagBits_VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT as _,
                dstAccessMask: VkAccessFlagBits_VK_ACCESS_SHADER_READ_BIT as _,
                dependencyFlags: VkDependencyFlagBits_VK_DEPENDENCY_BY_REGION_BIT as _,
            },
        ];

        let attachments = [color_attachment, depth_attachment];
        let render_pass_create_info = VkRenderPassCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            attachmentCount: attachments.len() as _,
            pAttachments: attachments.as_ptr(),
            subpassCount: 1,
            pSubpasses: &subpass1,
            dependencyCount: subpass_dependency.len() as _,
            pDependencies: subpass_dependency.as_ptr(),
        };
        unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkCreateRenderPass.unwrap()(
                    **device,
                    &render_pass_create_info,
                    ::std::ptr::null_mut(),
                    option.as_mut_ptr()
                )
            );
            option.assume_init()
        }
    }

    fn create_framebuffers(
        &mut self,
        device: &BackendDevice,
        physical_device: &BackendPhysicalDevice,
        color: Option<&BackendTexture>,
        depth: Option<&BackendTexture>,
    ) -> &mut Self {
        let details = physical_device.get_swap_chain_info();
        if let Some(texture) = color {
            self.extent = VkExtent2D {
                width: texture.width(),
                height: texture.height(),
            }
        } else {
            self.extent = details.capabilities.currentExtent;
        }

        let swapchain_images_count = device.get_images_count();
        self.framebuffer = Vec::with_capacity(swapchain_images_count);
        unsafe {
            self.framebuffer.set_len(swapchain_images_count);
        }
        for i in 0..device.get_images_count() {
            let attachments: Vec<VkImageView> = vec![
                if let Some(texture) = color {
                    texture.get_image_view()
                } else {
                    device.get_image_view(i)
                },
                if let Some(texture) = depth {
                    texture.get_image_view()
                } else {
                    device.get_depth_image_view(0)
                },
            ];

            let framebuffer_create_info = VkFramebufferCreateInfo {
                sType: VkStructureType_VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO,
                pNext: ::std::ptr::null_mut(),
                flags: 0,
                renderPass: self.render_pass,
                attachmentCount: attachments.len() as _,
                pAttachments: attachments.as_ptr(),
                width: self.extent.width,
                height: self.extent.height,
                layers: 1,
            };

            unsafe {
                assert_eq!(
                    VkResult_VK_SUCCESS,
                    vkCreateFramebuffer.unwrap()(
                        **device,
                        &framebuffer_create_info,
                        ::std::ptr::null_mut(),
                        &mut self.framebuffer[i]
                    )
                );
            }
        }

        self
    }
}
