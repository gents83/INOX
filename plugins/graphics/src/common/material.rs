use image::*;
use nrg_math::*;
use super::device::*;
use super::pipeline::*;

pub struct Material {
    inner: crate::api::backend::material::MaterialInstance,
    pipeline: Pipeline,
    device: Device,
}

impl Material {
    pub fn create(device:&Device, pipeline:&Pipeline) -> Material {
        let instance = crate::api::backend::material::MaterialInstance::create_from(&device.inner, &pipeline.inner);
        Material {
            inner: instance,
            pipeline: pipeline.clone(),
            device: device.clone(),
        } 
    }

    pub fn destroy(&mut self) {
        self.inner.destroy(&self.device.inner);
    }
    
    pub fn add_texture_from_image(&mut self, image: &DynamicImage) -> &mut Self {
        self.inner.add_texture_from_image( &self.device.inner, image );
        self
    }

    pub fn add_texture_from_path(&mut self, filepath: &str) -> &mut Self {
        self.inner.add_texture_from_path( &self.device.inner, filepath );
        self
    }

    pub fn update_uniform_buffer(&mut self, model_transform: &Matrix4f, cam_pos: Vector3f) {
        self.inner.update_uniform_buffer(&self.device.inner, model_transform, cam_pos);
        self.update_simple();
    }

    pub fn update_simple(&self) {
        self.inner.update_descriptor_sets(&self.device.inner, &self.pipeline.inner);
    }
}