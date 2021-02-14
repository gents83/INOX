use std::path::PathBuf;

use super::device::*;
use super::render_pass::*;
use super::shader::*;

#[derive(Clone)]
pub struct Pipeline {
    pub inner: super::backend::pipeline::Pipeline,
    render_pass: RenderPass,
}

impl Pipeline {
    pub fn create(
        device: &Device,
        vert_filepath: PathBuf,
        frag_filepath: PathBuf,
        render_pass: RenderPass,
    ) -> Pipeline {
        //TODO pipeline could be reused - while instance should be unique
        let mut pipeline = super::backend::pipeline::Pipeline::create(&device.inner);
        pipeline
            .set_shader(ShaderType::Vertex, vert_filepath)
            .set_shader(ShaderType::Fragment, frag_filepath)
            .build();

        Pipeline {
            inner: pipeline,
            render_pass,
        }
    }

    pub fn destroy(&mut self) {
        self.inner.delete();
    }

    pub fn begin(&mut self) {
        self.render_pass.begin();
        self.inner.prepare(self.render_pass.get_pass());
    }

    pub fn end(&mut self) {
        self.render_pass.end();
    }
}
