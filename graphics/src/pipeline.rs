use crate::device::*;
use crate::render_pass::*;
use crate::shader::*;

pub struct Pipeline {
    pub inner: super::api::backend::pipeline::Pipeline,
}

impl Pipeline {
    pub fn create(device:&mut Device, vert_filepath: &str, frag_filepath: &str) -> Pipeline {
        
        //TODO pipeline could be reused - while instance should be unique
        let mut pipeline = super::api::backend::pipeline::Pipeline::default();
        pipeline.set_shader(device.get_internal_device(), ShaderType::Vertex, vert_filepath);
        pipeline.set_shader(device.get_internal_device(), ShaderType::Fragment, frag_filepath);
        pipeline.build(device.get_internal_device());

        Pipeline {
            inner: pipeline,
        }
    }

    pub fn destroy(&mut self, device:&Device) {
        self.inner.delete(device.get_internal_device());
    }

    pub fn prepare(&mut self, device:&Device, render_pass: &RenderPass) {
        self.inner.prepare(device.get_internal_device(), render_pass.get_pass());
    }
}