use crate::Texture;

use super::device::*;
use super::pipeline::*;
use nrg_math::*;

pub struct Material {
    inner: crate::api::backend::material::MaterialInstance,
    pipeline: Pipeline,
    device: Device,
}

impl Material {
    pub fn create(device: &Device, pipeline: &Pipeline) -> Material {
        let instance = crate::api::backend::material::MaterialInstance::create_from(
            &device.inner,
            &pipeline.inner,
        );
        Material {
            inner: instance,
            pipeline: pipeline.clone(),
            device: device.clone(),
        }
    }

    pub fn update_uniform_buffer(
        &mut self,
        model_transform: &Matrix4f,
        cam_pos: Vector3f,
        textures: &[&Texture],
    ) {
        self.inner
            .update_uniform_buffer(&self.device.inner, model_transform, cam_pos);
        self.update_simple(textures);
    }

    pub fn update_simple(&self, textures: &[&Texture]) {
        let inner_textures: Vec<&crate::api::backend::Texture> =
            textures.iter().map(|t| &(*t).inner).collect();
        self.inner.update_descriptor_sets(
            &self.device.inner,
            &self.pipeline.inner,
            &inner_textures,
        );
    }
}
