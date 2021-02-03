use image::*;
use nrg_math::*;
use super::device::*;
use super::pipeline::*;

pub struct Material {
    pipeline: super::backend::pipeline::Pipeline,
    inner: super::backend::material::MaterialInstance,
    device: Device,
}

impl Material {
    pub fn create(device:&Device, pipeline:&Pipeline) -> Material {
        let inner_device = device.inner.borrow();
        let instance = super::backend::material::MaterialInstance::create_from(&inner_device, &pipeline.inner);
        Material {
            inner: instance,
            pipeline: pipeline.inner.clone(),
            device: device.clone(),
        } 
    }

    pub fn destroy(&mut self) {
        self.inner.destroy(&self.device.inner.borrow());
    }
    
    pub fn add_texture_from_image(&mut self, image: &DynamicImage) -> &mut Self {
        self.inner.add_texture_from_image( &self.device.inner.borrow(), image );
        self
    }

    pub fn add_texture_from_path(&mut self, filepath: &str) -> &mut Self {
        self.inner.add_texture_from_path( &self.device.inner.borrow(), filepath );
        self
    }

    pub fn update_uniform_buffer(&mut self, model_transform: &Matrix4f, cam_pos: Vector3f) {
        let inner_device = self.device.inner.borrow();
        self.inner.update_uniform_buffer(&inner_device, model_transform, cam_pos);
        self.update_simple();
    }

    pub fn update_simple(&self) {
        let inner_device = self.device.inner.borrow();
        self.inner.update_descriptor_sets(&inner_device, &self.pipeline) ;
    }
}