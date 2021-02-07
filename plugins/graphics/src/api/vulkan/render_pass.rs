use std::{cell::RefCell, rc::Rc};
use vulkan_bindings::*;
use super::device::*;


#[derive(Clone)]
pub struct RenderPassImmutable {
    render_pass: VkRenderPass,
    framebuffers: Vec::<VkFramebuffer>,
}


#[derive(Clone)]
pub struct RenderPass {
    inner: Rc<RefCell<RenderPassImmutable>>,
}



impl RenderPass {
    pub fn create_default(device:&Device) -> Self {
        let mut immutable = RenderPassImmutable {
            render_pass: RenderPassImmutable::base_pass(device),
            framebuffers: Vec::new(),
        };        
        let inner = Rc::new(RefCell::new(immutable));
        inner.borrow_mut().create_framebuffers(device);
        Self {
            inner,
        }
    }

    pub fn destroy(&mut self, device:&Device) {
        self.inner.borrow_mut().destroy_framebuffers(device);
        unsafe {
            vkDestroyRenderPass.unwrap()(device.get_device(), self.inner.borrow().render_pass, ::std::ptr::null_mut());
        }
    }
    
    pub fn begin(&self, device:&Device) {        
        let clear_value = VkClearValue{ color: VkClearColorValue { float32: [ 0.0, 0.0, 0.0, 1.0 ] } };
        let render_pass_begin_info = VkRenderPassBeginInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO,
            pNext: ::std::ptr::null_mut(),
            renderPass: self.inner.borrow().render_pass,
            framebuffer: self.inner.borrow().framebuffers[device.get_current_image_index()],
            renderArea: VkRect2D { 
                offset: VkOffset2D {x: 0, y: 0},
                extent: device.get_instance().get_swap_chain_info().capabilities.currentExtent,
            },
            clearValueCount: 1,
            pClearValues: &clear_value,
        };
        
        unsafe {
            vkCmdBeginRenderPass.unwrap()(device.get_current_command_buffer(), &render_pass_begin_info, VkSubpassContents_VK_SUBPASS_CONTENTS_INLINE);
        }
    }

    pub fn end(&self, device:&Device) {
        unsafe {
            vkCmdEndRenderPass.unwrap()(device.get_current_command_buffer());
        }
    }
}

impl From<&RenderPass> for VkRenderPass {
    fn from(render_pass: &RenderPass) -> VkRenderPass {
        render_pass.inner.borrow().render_pass
    }
}


impl RenderPassImmutable {
    fn base_pass(device:&Device) -> VkRenderPass {        
        let details = device.get_instance().get_swap_chain_info();     
        let color_attachment = VkAttachmentDescription {
            flags: 0,
            format: details.formats[0].format,
            samples: VkSampleCountFlagBits_VK_SAMPLE_COUNT_1_BIT,
            loadOp: VkAttachmentLoadOp_VK_ATTACHMENT_LOAD_OP_CLEAR,
            storeOp: VkAttachmentStoreOp_VK_ATTACHMENT_STORE_OP_STORE,
            stencilLoadOp: VkAttachmentLoadOp_VK_ATTACHMENT_LOAD_OP_DONT_CARE,
            stencilStoreOp: VkAttachmentStoreOp_VK_ATTACHMENT_STORE_OP_DONT_CARE,
            initialLayout: VkImageLayout_VK_IMAGE_LAYOUT_UNDEFINED,
            finalLayout: VkImageLayout_VK_IMAGE_LAYOUT_PRESENT_SRC_KHR,
        };

        let color_attachment_ref = VkAttachmentReference {
            attachment: 0,
            layout: VkImageLayout_VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL,
        };

        let subpass1 = VkSubpassDescription {
            flags: 0,
            pipelineBindPoint: VkPipelineBindPoint_VK_PIPELINE_BIND_POINT_GRAPHICS,
            inputAttachmentCount: 0,
            pInputAttachments: ::std::ptr::null_mut(),
            colorAttachmentCount: 1,
            pColorAttachments: &color_attachment_ref,
            pResolveAttachments: ::std::ptr::null_mut(),
            pDepthStencilAttachment: ::std::ptr::null_mut(),
            preserveAttachmentCount: 0,
            pPreserveAttachments: ::std::ptr::null_mut(),
        };

        let subpass_dependency = VkSubpassDependency { 
            srcSubpass: VK_SUBPASS_EXTERNAL as _,
            dstSubpass: 0,
            srcStageMask: VkPipelineStageFlagBits_VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT as _,
            dstStageMask: VkPipelineStageFlagBits_VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT as _,
            srcAccessMask: 0,
            dstAccessMask: VkAccessFlagBits_VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT as _,
            dependencyFlags: 0,
        };

        let render_pass_create_info = VkRenderPassCreateInfo{
            sType: VkStructureType_VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            attachmentCount: 1,
            pAttachments: &color_attachment,
            subpassCount: 1,
            pSubpasses: &subpass1,
            dependencyCount: 1,
            pDependencies: &subpass_dependency,
        };
        
        unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkCreateRenderPass.unwrap()(device.get_device(), &render_pass_create_info, ::std::ptr::null_mut(), option.as_mut_ptr())
            );
            option.assume_init()
        }
    }
    

    fn create_framebuffers(&mut self, device:&Device) -> &mut Self {
        let mut framebuffers = Vec::<VkFramebuffer>::with_capacity(device.get_images_count());
        unsafe {
            framebuffers.set_len(device.get_images_count());
        }
        
        let details = device.get_instance().get_swap_chain_info();  
        for (i, framebuffer) in framebuffers.iter_mut().enumerate().take(device.get_images_count()) {            
            let framebuffer_create_info = VkFramebufferCreateInfo {
                sType: VkStructureType_VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO,
                pNext: ::std::ptr::null_mut(),
                flags: 0,
                renderPass: self.render_pass,
                attachmentCount: 1,
                pAttachments: &device.get_image_view(i),
                width: details.capabilities.currentExtent.width,
                height: details.capabilities.currentExtent.height,
                layers: 1,
            };

            unsafe {
                assert_eq!(
                    VkResult_VK_SUCCESS,
                    vkCreateFramebuffer.unwrap()(device.get_device(), &framebuffer_create_info, ::std::ptr::null_mut(), framebuffer)
                );
            }
        }
        self.framebuffers = framebuffers;
        self
    }

    fn destroy_framebuffers(&mut self, device:&Device) {                   
        unsafe {    
            for framebuffer in self.framebuffers.iter() {
                vkDestroyFramebuffer.unwrap()(device.get_device(), *framebuffer, ::std::ptr::null_mut());
            }
        }
    }
}