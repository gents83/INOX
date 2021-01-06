use image::*;
use nrg_math::*;
use crate::device::*;
use crate::pipeline::*;

pub struct Material {
    pipeline: super::api::backend::pipeline::Pipeline,
    inner: super::api::backend::material::MaterialInstance,
}

impl Material {
    pub fn create(device:&Device, pipeline:&Pipeline) -> Material {
        let instance = super::api::backend::material::MaterialInstance::create_from(&device.inner, &pipeline.inner);
        Material {
            inner: instance,
            pipeline: pipeline.inner.clone(),
        } 
    }

    pub fn destroy(&mut self, device:&Device) {
        self.inner.destroy(device.get_internal_device());
    }
    
    pub fn add_texture_from_image(&mut self, device: &Device, image: &DynamicImage) -> &mut Self {
        self.inner.add_texture_from_image( device.get_internal_device(), image );
        self
    }

    pub fn add_texture_from_path(&mut self, device: &Device, filepath: &str) -> &mut Self {
        self.inner.add_texture_from_path( device.get_internal_device(), filepath );
        self
    }

    pub fn update_uniform_buffer(&mut self, device:&Device, model_transform: &Matrix4f, cam_pos: Vector3f) {
        self.inner.update_uniform_buffer(device.get_internal_device(), device.get_internal_device().get_current_image_index(), model_transform, cam_pos);
        self.update_simple(device);
    }

    pub fn update_simple(&self, device:&Device) {
        self.inner.update_descriptor_sets(device.get_internal_device(), &self.pipeline, device.get_internal_device().get_current_image_index()) ;
    }
}