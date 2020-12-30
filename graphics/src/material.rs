use nrg_math::*;
use crate::device::*;
use crate::render_pass::*;

pub enum ShaderType {
    Invalid,
    Vertex,
    Fragment,
}

pub struct Material {
    reference_material: super::api::backend::material::Material,
    inner: super::api::backend::material::MaterialInstance,
}

impl Material {
    pub fn create(device:&mut Device, vert_filepath: &str, frag_filepath: &str) -> Material {
        
        //TODO material could be reused - while instance should be unique
        let mut material = super::api::backend::material::Material::default();
        material.set_shader(device.get_internal_device(), ShaderType::Vertex, vert_filepath);
        material.set_shader(device.get_internal_device(), ShaderType::Fragment, frag_filepath);
        material.build_material(device.get_internal_device());
        let instance = super::api::backend::material::MaterialInstance::create_from(&mut device.inner, &material);
        Material {
            inner: instance,
            reference_material: material,
        } 
    }

    pub fn destroy(&mut self, device:&Device) {
        self.inner.destroy(device.get_internal_device());
        self.reference_material.delete(device.get_internal_device());
    }
    pub fn add_texture(&mut self, device: &Device, filepath: &str) -> &mut Self {
        self.inner.add_texture( device.get_internal_device(), filepath );
        self
    }

    pub fn prepare_pipeline(&mut self, device:&Device, render_pass: &RenderPass) {
        self.reference_material.create_graphics_pipeline(device.get_internal_device(), render_pass.get_pass());
        self.inner.update_descriptor_sets(device.get_internal_device(), &self.reference_material, device.get_internal_device().get_current_image_index()) ;
    }

    pub fn update_uniform_buffer(&mut self, device:&Device, model_transform: &Matrix4f, cam_pos: Vector3f) {
        self.reference_material.update_uniform_buffer(device.get_internal_device(), device.get_internal_device().get_current_image_index(), model_transform, cam_pos);
    }
}