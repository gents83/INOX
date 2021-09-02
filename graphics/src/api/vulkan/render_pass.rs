use super::device::*;
use super::utils::*;
use super::BackendTexture;
use crate::common::data_formats::*;

use std::sync::{Arc, RwLock};
use vulkan_bindings::*;

#[derive(Clone)]
pub struct RenderPassImmutable {
    render_pass: VkRenderPass,
    framebuffers: Vec<VkFramebuffer>,
    extent: VkExtent2D,
}

#[derive(Clone)]
pub struct BackendRenderPass {
    inner: Arc<RwLock<RenderPassImmutable>>,
}

unsafe impl Send for BackendRenderPass {}
unsafe impl Sync for BackendRenderPass {}

impl BackendRenderPass {
    pub fn get_extent(&self) -> VkExtent2D {
        self.inner.read().unwrap().extent
    }
    pub fn get_framebuffer_width(&self) -> u32 {
        self.inner.read().unwrap().extent.width
    }
    pub fn get_framebuffer_height(&self) -> u32 {
        self.inner.read().unwrap().extent.height
    }

    pub fn get_render_pass(&self) -> VkRenderPass {
        self.inner.read().unwrap().render_pass
    }

    pub fn get_framebuffer(&self, device: &BackendDevice) -> VkFramebuffer {
        self.inner.read().unwrap().framebuffers[device.get_current_buffer_index()]
    }

    pub fn create_default(
        device: &mut BackendDevice,
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
            device
                .get_instance()
                .get_swap_chain_info()
                .capabilities
                .currentExtent
        };
        let immutable = RenderPassImmutable {
            render_pass: RenderPassImmutable::base_pass(device, data),
            framebuffers: Vec::new(),
            extent,
        };
        let inner = Arc::new(RwLock::new(immutable));
        inner
            .write()
            .unwrap()
            .create_framebuffers(device, color, depth);
        Self { inner }
    }

    pub fn destroy(&mut self, device: &BackendDevice) {
        self.inner.write().unwrap().destroy_framebuffers(device);
        unsafe {
            vkDestroyRenderPass.unwrap()(
                device.get_device(),
                self.get_render_pass(),
                ::std::ptr::null_mut(),
            );
        }
    }

    pub fn begin(&self, device: &mut BackendDevice) {
        let clear_value = [
            VkClearValue {
                color: VkClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
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
            extent: self.inner.read().unwrap().extent,
        };
        let render_pass_begin_info = VkRenderPassBeginInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO,
            pNext: ::std::ptr::null_mut(),
            renderPass: self.get_render_pass(),
            framebuffer: self.get_framebuffer(device),
            renderArea: area,
            clearValueCount: clear_value.len() as _,
            pClearValues: clear_value.as_ptr(),
        };
        unsafe {
            vkCmdBeginRenderPass.unwrap()(
                device.get_current_command_buffer(),
                &render_pass_begin_info,
                VkSubpassContents_VK_SUBPASS_CONTENTS_INLINE,
            );
        }
    }

    pub fn end(&self, device: &mut BackendDevice) {
        unsafe {
            vkCmdEndRenderPass.unwrap()(device.get_current_command_buffer());
        }
    }
}

impl From<&BackendRenderPass> for VkRenderPass {
    fn from(render_pass: &BackendRenderPass) -> VkRenderPass {
        render_pass.get_render_pass()
    }
}

impl RenderPassImmutable {
    fn base_pass(device: &BackendDevice, data: &RenderPassData) -> VkRenderPass {
        let details = device.get_instance().get_swap_chain_info();
        let color_attachment = VkAttachmentDescription {
            flags: 0,
            format: if data.render_to_texture {
                VkFormat_VK_FORMAT_R8G8B8A8_UNORM
            } else {
                details.formats[0].format
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
            finalLayout: VkImageLayout_VK_IMAGE_LAYOUT_PRESENT_SRC_KHR,
        };

        let color_attachment_ref = VkAttachmentReference {
            attachment: 0,
            layout: VkImageLayout_VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL,
        };

        let depth_attachment = VkAttachmentDescription {
            flags: 0,
            format: find_depth_format(device.get_instance().get_physical_device()),
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
                srcStageMask: (VkPipelineStageFlagBits_VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT) as _,
                dstStageMask:
                    (VkPipelineStageFlagBits_VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT) as _,
                srcAccessMask: (VkAccessFlagBits_VK_ACCESS_MEMORY_READ_BIT) as _,
                dstAccessMask: (VkAccessFlagBits_VK_ACCESS_COLOR_ATTACHMENT_READ_BIT
                    | VkAccessFlagBits_VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT)
                    as _,
                dependencyFlags: VkDependencyFlagBits_VK_DEPENDENCY_BY_REGION_BIT as _,
            },
            VkSubpassDependency {
                srcSubpass: 0,
                dstSubpass: VK_SUBPASS_EXTERNAL as _,
                srcStageMask:
                    (VkPipelineStageFlagBits_VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT) as _,
                dstStageMask: (VkPipelineStageFlagBits_VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT) as _,
                srcAccessMask: (VkAccessFlagBits_VK_ACCESS_COLOR_ATTACHMENT_READ_BIT
                    | VkAccessFlagBits_VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT)
                    as _,
                dstAccessMask: (VkAccessFlagBits_VK_ACCESS_MEMORY_READ_BIT) as _,
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
                    device.get_device(),
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
        color: Option<&BackendTexture>,
        depth: Option<&BackendTexture>,
    ) -> &mut Self {
        if !self.framebuffers.is_empty() {
            self.destroy_framebuffers(device);
        }

        let mut framebuffers = Vec::<VkFramebuffer>::with_capacity(device.get_images_count());
        unsafe {
            framebuffers.set_len(device.get_images_count());
        }

        let details = device.get_instance().get_swap_chain_info();
        if let Some(texture) = color {
            self.extent = VkExtent2D {
                width: texture.width(),
                height: texture.height(),
            }
        } else {
            self.extent = details.capabilities.currentExtent;
        }

        for (i, framebuffer) in framebuffers
            .iter_mut()
            .enumerate()
            .take(device.get_images_count())
        {
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
                        device.get_device(),
                        &framebuffer_create_info,
                        ::std::ptr::null_mut(),
                        framebuffer
                    )
                );
            }
        }
        self.framebuffers = framebuffers;
        self
    }

    fn destroy_framebuffers(&mut self, device: &BackendDevice) {
        unsafe {
            for framebuffer in self.framebuffers.iter() {
                vkDestroyFramebuffer.unwrap()(
                    device.get_device(),
                    *framebuffer,
                    ::std::ptr::null_mut(),
                );
            }
        }
    }
}
