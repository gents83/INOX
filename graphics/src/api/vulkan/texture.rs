use super::device::*;
use image::*;
use vulkan_bindings::*;

#[derive(PartialEq)]
pub struct Texture {
    texture_image: VkImage,
    texture_image_memory: VkDeviceMemory,
    texture_image_view: VkImageView,
    texture_sampler: VkSampler,
}
unsafe impl Send for Texture {}
unsafe impl Sync for Texture {}

impl Texture {
    pub fn create_from(device: &Device, filepath: &str) -> Self {
        let image_data = image::open(filepath).unwrap();
        Texture::create(&device, &image_data)
    }

    pub fn create(device: &Device, image_data: &DynamicImage) -> Self {
        let mut texture = Self {
            texture_image: ::std::ptr::null_mut(),
            texture_image_memory: ::std::ptr::null_mut(),
            texture_image_view: ::std::ptr::null_mut(),
            texture_sampler: ::std::ptr::null_mut(),
        };
        texture.create_texture_image(device, &image_data);
        texture.create_texture_sampler(device);
        texture
    }

    pub fn destroy(&self, device: &Device) {
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
}

impl Texture {
    fn create_texture_image(&mut self, device: &Device, image_data: &image::DynamicImage) {
        let image_size: VkDeviceSize =
            (image_data.width() * image_data.height() * image_data.color().channel_count() as u32)
                as _;
        let flags = VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT
            | VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_HOST_COHERENT_BIT;
        let format = match image_data.color() {
            image::ColorType::Rgb8 => VkFormat_VK_FORMAT_R8G8B8_UNORM,
            _ => VkFormat_VK_FORMAT_R8G8B8A8_UNORM,
        };
        let mut staging_buffer: VkBuffer = ::std::ptr::null_mut();
        let mut staging_buffer_memory: VkDeviceMemory = ::std::ptr::null_mut();
        device.create_buffer(
            image_size as _,
            VkBufferUsageFlagBits_VK_BUFFER_USAGE_TRANSFER_SRC_BIT as _,
            flags as _,
            &mut staging_buffer,
            &mut staging_buffer_memory,
        );

        device.map_buffer_memory(&mut staging_buffer_memory, image_data.as_bytes());

        let flags = VkImageUsageFlagBits_VK_IMAGE_USAGE_TRANSFER_DST_BIT
            | VkImageUsageFlagBits_VK_IMAGE_USAGE_SAMPLED_BIT;
        let device_image = device.create_image(
            (image_data.width(), image_data.height(), format),
            VkImageTiling_VK_IMAGE_TILING_OPTIMAL,
            flags as _,
            VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT as _,
        );

        self.texture_image = device_image.0;
        self.texture_image_memory = device_image.1;

        device.transition_image_layout(
            self.texture_image,
            VkImageLayout_VK_IMAGE_LAYOUT_UNDEFINED,
            VkImageLayout_VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
        );

        device.copy_buffer_to_image(
            staging_buffer,
            self.texture_image,
            image_data.width(),
            image_data.height(),
        );

        device.transition_image_layout(
            self.texture_image,
            VkImageLayout_VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
            VkImageLayout_VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL,
        );

        device.destroy_buffer(&staging_buffer, &staging_buffer_memory);

        self.texture_image_view = device.create_image_view(
            self.texture_image,
            format,
            VkImageAspectFlagBits_VK_IMAGE_ASPECT_COLOR_BIT as _,
        );
    }

    fn create_texture_sampler(&mut self, device: &Device) {
        let properties = device.get_instance().get_physical_device_properties();

        let sampler_info = VkSamplerCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_SAMPLER_CREATE_INFO,
            pNext: ::std::ptr::null_mut(),
            flags: 0,
            magFilter: VkFilter_VK_FILTER_LINEAR,
            minFilter: VkFilter_VK_FILTER_LINEAR,
            mipmapMode: VkSamplerMipmapMode_VK_SAMPLER_MIPMAP_MODE_LINEAR,
            addressModeU: VkSamplerAddressMode_VK_SAMPLER_ADDRESS_MODE_REPEAT,
            addressModeV: VkSamplerAddressMode_VK_SAMPLER_ADDRESS_MODE_REPEAT,
            addressModeW: VkSamplerAddressMode_VK_SAMPLER_ADDRESS_MODE_REPEAT,
            mipLodBias: 0.0,
            anisotropyEnable: VK_TRUE,
            maxAnisotropy: properties.limits.maxSamplerAnisotropy,
            compareEnable: VK_FALSE,
            compareOp: VkCompareOp_VK_COMPARE_OP_NEVER,
            minLod: 0.0,
            maxLod: 1.0,
            borderColor: VkBorderColor_VK_BORDER_COLOR_INT_OPAQUE_WHITE,
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
