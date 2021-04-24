use nrg_math::Vector3;

use crate::api::backend::Texture;

use super::data_formats::*;
use super::device::*;
use super::render_pass::*;
use super::shader::*;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Pipeline {
    pub inner: crate::api::backend::pipeline::Pipeline,
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
        let mut pipeline = crate::api::backend::pipeline::Pipeline::create(&device.inner);
        pipeline
            .set_shader(ShaderType::Vertex, vert_filepath)
            .set_shader(ShaderType::Fragment, frag_filepath)
            .build(&device.inner, &render_pass.get_pass());

        Pipeline {
            inner: pipeline,
            render_pass,
        }
    }

    pub fn destroy(&mut self) {
        self.inner.delete();
    }

    pub fn recreate(&mut self, render_pass: RenderPass) {
        self.render_pass.destroy();
        self.render_pass = render_pass;
    }

    pub fn begin(&mut self, commands: &[InstanceCommand], instances: &[InstanceData]) {
        self.render_pass.begin();
        self.inner.bind(commands, instances).bind_descriptors();
    }

    pub fn update_uniform_buffer(&self, cam_pos: Vector3) {
        self.inner.update_uniform_buffer(cam_pos);
    }

    pub fn update_descriptor_sets(&self, textures: &[&Texture]) {
        self.inner.update_descriptor_sets(textures);
    }

    pub fn bind_indirect(&mut self) {
        self.inner.bind_indirect();
    }

    pub fn draw_indirect(&mut self, count: usize) {
        self.inner.draw_indirect(count);
    }

    pub fn end(&mut self) {
        self.render_pass.end();
    }
}
