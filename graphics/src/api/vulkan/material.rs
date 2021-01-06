
use image::*;
use nrg_math::*;
use vulkan_bindings::*;
use crate::data_formats::*;
use super::pipeline::*;
use super::device::*;
use super::texture::*;

#[derive(PartialEq)]
pub struct MaterialInstance {
    textures: Vec<Texture>,
    descriptor_pool: VkDescriptorPool,
    descriptor_sets: Vec<VkDescriptorSet>,
    uniform_buffers_size: usize,
    uniform_buffers: Vec<VkBuffer>,
    uniform_buffers_memory: Vec<VkDeviceMemory>,
}

impl MaterialInstance {
    pub fn create_from(device: &Device, pipeline: &super::pipeline::Pipeline) -> Self {
        let mut instance = MaterialInstance {
            textures: Vec::new(),
            descriptor_sets: Vec::new(),
            descriptor_pool: ::std::ptr::null_mut(),
            uniform_buffers_size: 0,
            uniform_buffers: Vec::new(),
            uniform_buffers_memory: Vec::new(),
        };
        instance.create_uniform_buffers(device);
        instance.create_descriptor_pool(device);
        instance.create_descriptor_sets(&device, &pipeline);
        instance       
    }

    pub fn destroy(&self, device: &Device) {
        for texture in self.textures.iter() {
            texture.destroy(device);
        }
    }  

    pub fn add_texture_from_image(&mut self, device: &Device, image: &DynamicImage) -> &mut Self {
        self.textures.push( Texture::create(device, image) );
        self
    }

    pub fn add_texture_from_path(&mut self, device: &Device, filepath: &str) -> &mut Self {
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
            descriptorPool: self.descriptor_pool,
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

    fn create_descriptor_pool(&mut self, device: &Device) {
        let pool_sizes = [
            VkDescriptorPoolSize {
                type_: VkDescriptorType_VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
                descriptorCount: device.get_images_count() as _,
            },
            VkDescriptorPoolSize {
                type_: VkDescriptorType_VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
                descriptorCount: device.get_images_count() as _,
            },
        ];

        let pool_info = VkDescriptorPoolCreateInfo {
            sType: VkStructureType_VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO,
            flags: 0,
            pNext: ::std::ptr::null_mut(),
            poolSizeCount: pool_sizes.len() as _,
            pPoolSizes: pool_sizes.as_ptr(),
            maxSets: device.get_images_count() as _,
        };

        self.descriptor_pool = unsafe {
            let mut option = ::std::mem::MaybeUninit::uninit();
            assert_eq!(
                VkResult_VK_SUCCESS,
                vkCreateDescriptorPool.unwrap()(device.get_device(), &pool_info, ::std::ptr::null_mut(), option.as_mut_ptr())
            );
            option.assume_init()
        };  
    }

    fn create_uniform_buffers(&mut self, device: &Device) {  
        let mut uniform_buffers = Vec::<VkBuffer>::with_capacity(device.get_images_count());
        let mut uniform_buffers_memory = Vec::<VkDeviceMemory>::with_capacity(device.get_images_count());
        unsafe {
            uniform_buffers.set_len(device.get_images_count());
            uniform_buffers_memory.set_len(device.get_images_count());
        }

        let uniform_buffers_size = std::mem::size_of::<UniformData>();
        let flags = VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VkMemoryPropertyFlagBits_VK_MEMORY_PROPERTY_HOST_COHERENT_BIT;
        for i in 0..uniform_buffers.len() {
            device.create_buffer(uniform_buffers_size as _, VkBufferUsageFlagBits_VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT as _, flags as _, &mut uniform_buffers[i], &mut uniform_buffers_memory[i]);
        }

        self.uniform_buffers_size = uniform_buffers_size;
        self.uniform_buffers = uniform_buffers;
        self.uniform_buffers_memory = uniform_buffers_memory;
    }
    
    pub fn update_uniform_buffer(&mut self, device:&Device, image_index: usize, model_transform: &Matrix4f, cam_pos: Vector3f) {
        let details = device.get_instance().get_swap_chain_info();
        let uniform_data: [UniformData; 1] = [
            UniformData {
                model: *model_transform,
                view: Matrix4::from_look_at(cam_pos.into(), 
                                            [0.0, 0.0, 0.0].into(),
                                            [0.0, 0.0, 1.0].into()),
                proj: Matrix4::create_perspective(Degree(45.0).into(), 
                                                    details.capabilities.currentExtent.width as f32 / details.capabilities.currentExtent.height as f32, 
                                                    0.1, 1000.0),
            }
        ];

        let mut buffer_memory = self.uniform_buffers_memory[image_index];
        device.map_buffer_memory(&mut buffer_memory, &uniform_data);
        self.uniform_buffers_memory[image_index] = buffer_memory;
    }
    
    pub fn update_descriptor_sets(&self, device: &Device, pipeline: &Pipeline, image_index: usize) {
        let buffer_info = VkDescriptorBufferInfo {
            buffer: self.uniform_buffers[image_index],
            offset: 0,
            range: self.uniform_buffers_size as _,
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

