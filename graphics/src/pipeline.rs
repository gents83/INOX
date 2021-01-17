use crate::device::*;
use crate::render_pass::*;
use crate::shader::*;

pub struct Pipeline {
    pub inner: super::api::backend::pipeline::Pipeline,
}

impl Pipeline {
    pub fn create(device:&mut Device, vert_filepath: &str, frag_filepath: &str) -> Pipeline {
        let inner_device = device.inner.borrow();
        //TODO pipeline could be reused - while instance should be unique
        let mut pipeline = super::api::backend::pipeline::Pipeline::create(&inner_device);
        pipeline.set_shader(ShaderType::Vertex, vert_filepath)
                .set_shader(ShaderType::Fragment, frag_filepath)
                .build();

        Pipeline {
            inner: pipeline
        }
    }

    pub fn destroy(&mut self) {
        self.inner.delete();
    }

    pub fn prepare(&mut self, render_pass: &RenderPass) {
        self.inner.prepare(render_pass.get_pass());
    }
}