use crate::Area;

use super::{device::*, find_depth_format};
use vulkan_bindings::*;

const TEXTURE_CHANNEL_COUNT: u32 = 4;

#[derive(Clone, PartialEq)]
pub struct BackendTexture {
    width: u32,
    height: u32,
    texture_image: VkImage,
    texture_image_memory: VkDeviceMemory,
    texture_image_view: VkImageView,
    texture_sampler: VkSampler,
}
unsafe impl Send for BackendTexture {}
unsafe impl Sync for BackendTexture {}

impl BackendTexture {
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn create(device: &BackendDevice, width: u32, height: u32, layers_count: usize) -> Self {
        let mut texture = Self {
            width,
            height,
            texture_image: ::std::ptr::null_mut(),
            texture_image_memory: ::std::ptr::null_mut(),
            texture_image_view: ::std::ptr::null_mut(),
            texture_sampler: ::std::ptr::null_mut(),
        };
        texture.create_texture_image(
            device,
            VkFormat_VK_FORMAT_R8G8B8A8_UNORM,
            layers_count,
            VkImageUsageFlagBits_VK_IMAGE_USAGE_TRANSFER_DST_BIT,
            VkImageAspectFlagBits_VK_IMAGE_ASPECT_COLOR_BIT,
        );
        texture.create_texture_sampler(device);
        texture
    }
    pub fn create_as_render_target(
        device: &BackendDevice,
        width: u32,
        height: u32,
        layers_count: usize,
        is_depth: bool,
    ) -> Self {
        let mut texture = Self {
            width,
            height,
            texture_image: ::std::ptr::null_mut(),
            texture_image_memory: ::std::ptr::null_mut(),
            texture_image_view: ::std::ptr::null_mut(),
            texture_sampler: ::std::ptr::null_mut(),
        };
        let format = if is_depth {
            find_depth_format(device.get_instance().get_physical_device())
        } else {
            VkFormat_VK_FORMAT_R8G8B8A8_UNORM
        };
        let specific_flags = if is_depth {
            VkImageUsageFlagBits_VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT as _
        } else {
            (VkImageUsageFlagBits_VK_IMAGE_USAGE_TRANSFER_DST_BIT
                | VkImageUsageFlagBits_VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT) as _
        };
        let aspect_flags = if is_depth {
            (VkImageAspectFlagBits_VK_IMAGE_ASPECT_DEPTH_BIT
                | VkImageAspectFlagBits_VK_IMAGE_ASPECT_STENCIL_BIT) as _
        } else {
            VkImageAspectFlagBits_VK_IMAGE_ASPECT_COLOR_BIT as _
        };
        texture.create_texture_image(device, format, layers_count, specific_flags, aspect_flags);
        texture.create_texture_sampler(device);
        texture
    }
    pub fn destroy(&self, device: &BackendDevice) {
        unsafe {
            vkDestroySampler.unwrap()(
                device.get_device(),
                self.texture_sampler,
                ::std::ptr::null_mut(),
            );
            vkDestroyImageView.unwrap()(
                device.get_device(),
                self.texture_image_view,
                ::std::ptr::null_mut(),
            );

            vkDestroyImage.unwrap()(
                device.get_device(),
                self.texture_image,
                ::std::ptr::null_mut(),
            );
            vkFreeMemory.unwrap()(
                device.get_device(),
                self.texture_image_memory,
                ::std::ptr::null_mut(),
            );
        }
    }

    pub fn get_descriptor(&self) -> VkDescriptorImageInfo {
        VkDescriptorImageInfo {
            imageLayout: VkImageLayout_VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL,
            imageView: self.texture_image_view,
            sampler: self.texture_sampler,
        }
    }

    pub fn get_image_view(&self) -> VkImageView {
        self.texture_image_view
    }

    pub fn add_in_layer(
        &mut self,
        device: &BackendDevice,
        index: usize,
        area: &Area,
        image_data: &[u8],
    ) {
        if self.width < area.width || self.height < area.height {
            panic!("Image resolution is different from texture one");
        }
        let image_size: VkDeviceSize = (area.width * area.height * TEXTURE_CHANNEL_COUNT) as _;
        let flags = VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT
            | VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_HOST_COHERENT_BIT;
        let mut staging_buffer: VkBuffer = ::std::ptr::null_mut();
        let mut staging_buffer_memory: VkDeviceMemory = ::std::ptr::null_mut();
        device.create_buffer(
            image_size as _,
            VkBufferUsageFlagBits_VK_BUFFER_USAGE_TRANSFER_SRC_BIT as _,
            flags as _,
            &mut staging_buffer,
            &mut staging_buffer_memory,
        );

        device.map_buffer_memory(&mut staging_buffer_memory, 0, image_data);

        device.transition_image_layout(
            self.texture_image,
            VkImageLayout_VK_IMAGE_LAYOUT_UNDEFINED,
            VkImageLayout_VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
            index,
            1,
        );

        device.copy_buffer_to_image(staging_buffer, self.texture_image, index, area);

        device.transition_image_layout(
            self.texture_image,
            VkImageLayout_VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
            VkImageLayout_VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL,
            index,
            1,
        );

        device.destroy_buffer(&staging_buffer, &staging_buffer_memory);
    }
}

impl BackendTexture {
    fn create_texture_image(
        &mut self,
        device: &BackendDevice,
        format: VkFormat,
        layers_count: usize,
        specific_flags: i32,
        aspect_flags: i32,
    ) {
        let flags = specific_flags | VkImageUsageFlagBits_VK_IMAGE_USAGE_SAMPLED_BIT;
        let device_image = device.create_image(
            (self.width, self.height, format),
            VkImageTiling_VK_IMAGE_TILING_OPTIMAL,
            flags as _,
            VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT as _,
            layers_count,
        );

        self.texture_image = device_image.0;
        self.texture_image_memory = device_image.1;
        self.texture_image_view =
            device.create_image_view(self.texture_image, format, aspect_flags as _, layers_count);
    }

    fn create_texture_sampler(&mut self, device: &BackendDevice) {
        let properties = device.get_instance().get_physical_device_properties();

        let sampler_info = VkSamplerCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_SAMPLER_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            magFilter: VkFilter_VK_FILTER_LINEAR,
            minFilter: VkFilter_VK_FILTER_LINEAR,
            mipmapMode: VkSamplerMipmapMode_VK_SAMPLER_MIPMAP_MODE_LINEAR,
            addressModeU: VkSamplerAddressMode_VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE,
            addressModeV: VkSamplerAddressMode_VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE,
            addressModeW: VkSamplerAddressMode_VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE,
            mipLodBias: 0.0,
            anisotropyEnable: VK_TRUE,
            maxAnisotropy: properties.limits.maxSamplerAnisotropy,
            compareEnable: VK_FALSE,
            compareOp: VkCompareOp_VK_COMPARE_OP_NEVER,
            minLod: 0.0,
            maxLod: 1.0,
            borderColor: VkBorderColor_VK_BORDER_COLOR_FLOAT_TRANSPARENT_BLACK,
            unnormalizedCoordinates: VK_FALSE,
        };

        self.texture_sampler = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkCreateSampler.unwrap()(
                    device.get_device(),
                    &sampler_info,
                    ::std::ptr::null_mut(),
                    option.as_mut_ptr()
                )
            );
            option.assume_init()
        };
    }
}
