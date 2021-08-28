use nrg_math::{Mat4Ops, Matrix4};
use nrg_math::{Vector4, Zero};
use nrg_resources::ResourceData;

use crate::utils::compute_color_from_id;
use crate::{Mesh, MeshInstance, PipelineId};

use super::data_formats::*;
use super::device::*;
use super::render_pass::*;
use super::shader::*;
use super::texture::*;

#[derive(Clone)]
pub struct Pipeline {
    pub inner: crate::api::backend::pipeline::Pipeline,
    id: PipelineId,
    mesh: Mesh,
    vertex_count: u32,
    index_count: u32,
    instance_count: usize,
    instance_data: Vec<InstanceData>,
    instance_commands: Vec<InstanceCommand>,
}
unsafe impl Send for Pipeline {}
unsafe impl Sync for Pipeline {}

impl Pipeline {
    pub fn id(&self) -> PipelineId {
        self.id
    }

    pub fn create(
        device: &Device,
        id: PipelineId,
        data: &PipelineData,
        render_pass: &RenderPass,
    ) -> Pipeline {
        //TODO pipeline could be reused - while instance should be unique
        let mut pipeline = crate::api::backend::pipeline::Pipeline::create(&device.inner);

        pipeline
            .set_shader(ShaderType::Vertex, data.vertex_shader.as_path())
            .set_shader(ShaderType::Fragment, data.fragment_shader.as_path());
        if !data.tcs_shader.to_str().unwrap().is_empty() {
            pipeline.set_shader(ShaderType::TessellationControl, data.tcs_shader.as_path());
        }
        if !data.tes_shader.to_str().unwrap().is_empty() {
            pipeline.set_shader(
                ShaderType::TessellationEvaluation,
                data.tes_shader.as_path(),
            );
        }
        if !data.geometry_shader.to_str().unwrap().is_empty() {
            pipeline.set_shader(ShaderType::Geometry, data.geometry_shader.as_path());
        }
        pipeline.build(
            &device.inner,
            render_pass.get_pass(),
            &data.culling,
            &data.mode,
        );

        Pipeline {
            inner: pipeline,
            id,
            mesh: Mesh::create(device),
            vertex_count: 0,
            index_count: 0,
            instance_count: 0,
            instance_data: Vec::new(),
            instance_commands: Vec::new(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.instance_commands.is_empty() || self.vertex_count.is_zero()
    }
    pub fn get_instance_data(&self) -> &Vec<InstanceData> {
        &self.instance_data
    }
    pub fn get_instance_commands(&self) -> &Vec<InstanceCommand> {
        &self.instance_commands
    }

    pub fn destroy(&mut self) {
        self.inner.delete();
    }

    pub fn prepare(&mut self) -> &mut Self {
        self.vertex_count = 0;
        self.index_count = 0;
        self.instance_count = 0;
        self
    }

    pub fn begin(&mut self) -> &mut Self {
        self.inner
            .bind(&self.instance_commands, &self.instance_data)
            .bind_descriptors();
        self
    }

    pub fn update_runtime_data(
        &self,
        width: f32,
        height: f32,
        view: &Matrix4,
        proj: &Matrix4,
    ) -> &Self {
        self.inner.update_constant_data(width, height, view, proj);
        self.inner.update_uniform_buffer(view, proj);
        self
    }
    pub fn update_descriptor_sets(&self, textures: &[TextureAtlas]) -> &Self {
        self.inner.update_descriptor_sets(textures);
        self
    }

    pub fn bind_indirect(&mut self) -> &mut Self {
        self.inner.bind_indirect();
        self
    }
    pub fn bind_vertices(&mut self) -> &mut Self {
        self.mesh.bind_vertices();
        self
    }
    pub fn bind_indices(&mut self) -> &mut Self {
        self.mesh.bind_indices();
        self
    }

    pub fn draw_indirect(&mut self) -> &mut Self {
        self.inner.draw_indirect(self.instance_count);
        self
    }

    pub fn end(&mut self) -> &mut Self {
        self
    }

    pub fn add_mesh_instance(
        &mut self,
        mesh_instance: &MeshInstance,
        diffuse_color: Vector4,
        diffuse_texture_index: i32,
        diffuse_layer_index: i32,
    ) -> &mut Self {
        if mesh_instance.mesh_data().vertices.is_empty()
            || mesh_instance.mesh_data().indices.is_empty()
        {
            return self;
        }

        nrg_profiler::scoped_profile!(format!("add_mesh_instance[{}]", self.id()).as_str());

        let mesh_data_ref = self.mesh.bind_at_index(
            &mesh_instance.mesh_data().vertices,
            self.vertex_count,
            &mesh_instance.mesh_data().indices,
            self.index_count,
        );

        self.vertex_count += mesh_instance.mesh_data().vertices.len() as u32;
        self.index_count += mesh_instance.mesh_data().indices.len() as u32;
        let mesh_index = if mesh_instance.draw_index() >= 0 {
            mesh_instance.draw_index() as usize
        } else {
            self.instance_count
        };

        let command = InstanceCommand {
            mesh_index,
            mesh_data_ref,
        };
        let (position, rotation, scale) = mesh_instance.matrix().get_translation_rotation_scale();

        let data = InstanceData {
            id: compute_color_from_id(mesh_instance.id()),
            position,
            rotation,
            scale,
            draw_area: mesh_instance.draw_area(),
            diffuse_color,
            diffuse_texture_index,
            diffuse_layer_index,
        };
        if mesh_index >= self.instance_commands.len() {
            self.instance_commands
                .resize(mesh_index + 1, InstanceCommand::default());
        }
        if mesh_index >= self.instance_data.len() {
            self.instance_data
                .resize(mesh_index + 1, InstanceData::default());
        }
        self.instance_commands[mesh_index] = command;
        self.instance_data[mesh_index] = data;
        self.instance_count += 1;
        self
    }
}
