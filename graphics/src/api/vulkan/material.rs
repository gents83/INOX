use vulkan_bindings::*;
use super::pipeline::*;
use super::device::*;
use super::texture::*;

#[derive(PartialEq)]
pub struct MaterialInstance {
    textures: Vec<Texture>,
    descriptor_sets: Vec<VkDescriptorSet>,
}

impl MaterialInstance {
    pub fn create_from(device: &mut Device, pipeline: &super::pipeline::Pipeline) -> Self {
        let mut instance = MaterialInstance {
            textures: Vec::new(),
            descriptor_sets: Vec::new(),
        };
        instance.create_descriptor_sets(&device, &pipeline);
        instance       
    }

    pub fn destroy(&self, device: &Device) {
        for texture in self.textures.iter() {
            texture.destroy(device);
        }
    }  

    pub fn add_texture(&mut self, device: &Device, filepath: &str) -> &mut Self {
        self.textures.push( Texture::create_from(device, filepath) );
        self
    }

    pub fn create_descriptor_sets(&mut self, device: &Device, pipeline: &Pipeline) {
        let mut layouts = Vec::<VkDescriptorSetLayout>::with_capacity(device.get_images_count());
        unsafe {
            layouts.set_len(device.get_images_count());
        }
        for layout in layouts.iter_mut() {
            *layout = pipeline.get_descriptor_set_layout();
        }

        let alloc_info = VkDescriptorSetAllocateInfo{
            sType: VkStructureType_VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO,
            pNext: ::std::ptr::null_mut(),
            descriptorPool: pipeline.get_descriptor_pool(),
            descriptorSetCount: device.get_images_count() as _,
            pSetLayouts: layouts.as_mut_ptr(),
        };

        let mut descriptor_sets = Vec::<VkDescriptorSet>::with_capacity(device.get_images_count());
        unsafe {
            descriptor_sets.set_len(device.get_images_count());
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkAllocateDescriptorSets.unwrap()(device.get_device(), &alloc_info, descriptor_sets.as_mut_ptr())
            );
        }
        
        self.descriptor_sets = descriptor_sets;
    }

    pub fn update_descriptor_sets(&mut self, device: &Device, pipeline: &Pipeline, image_index: usize) {
        let buffer_info = VkDescriptorBufferInfo {
            buffer: pipeline.get_uniform_buffer(image_index),
            offset: 0,
            range: pipeline.get_uniform_buffers_size() as _,
        };

        let descriptor_write = [
            VkWriteDescriptorSet {
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
            },
            VkWriteDescriptorSet {
                sType: VkStructureType_VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
                pNext: ::std::ptr::null_mut(),
                dstSet: self.descriptor_sets[image_index],
                dstBinding: 1,
                dstArrayElement: 0,
                descriptorCount: 1,
                descriptorType: VkDescriptorType_VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
                pImageInfo: &self.textures[0].get_descriptor(),
                pBufferInfo: ::std::ptr::null_mut(),
                pTexelBufferView: ::std::ptr::null_mut(),
            },
        ];

        unsafe {
            vkUpdateDescriptorSets.unwrap()(device.get_device(), descriptor_write.len() as _, descriptor_write.as_ptr(), 0, ::std::ptr::null_mut());

            vkCmdBindDescriptorSets.unwrap()(device.get_current_command_buffer(), VkPipelineBindPoint_VK_PIPELINE_BIND_POINT_GRAPHICS, pipeline.get_pipeline_layout(), 0, 1, &self.descriptor_sets[image_index], 0, ::std::ptr::null_mut());
        }
    }
}

