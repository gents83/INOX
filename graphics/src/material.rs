use crate::device::*;
use crate::pipeline::*;

pub struct Material {
    pipeline: super::api::backend::pipeline::Pipeline,
    inner: super::api::backend::material::MaterialInstance,
}

impl Material {
    pub fn create(device:&mut Device, pipeline:&Pipeline) -> Material {
        let instance = super::api::backend::material::MaterialInstance::create_from(&mut device.inner, &pipeline.inner);
        Material {
            inner: instance,
            pipeline: pipeline.inner.clone(),
        } 
    }

    pub fn destroy(&mut self, device:&Device) {
        self.inner.destroy(device.get_internal_device());
    }
    pub fn add_texture(&mut self, device: &Device, filepath: &str) -> &mut Self {
        self.inner.add_texture( device.get_internal_device(), filepath );
        self
    }

    pub fn update(&mut self, device:&Device) {
        self.inner.update_descriptor_sets(device.get_internal_device(), &self.pipeline, device.get_internal_device().get_current_image_index()) ;
    }
}