use nrg_math::{Vector3, Vector4, Zero};
use nrg_resources::{ResourceId, SharedDataRw};
use nrg_serialize::INVALID_UID;

use crate::{
    Device, InstanceCommand, InstanceData, Mesh, MeshInstance, Pipeline, PipelineData, RenderPass,
    TextureAtlas,
};

pub type PipelineId = ResourceId;

pub struct PipelineInstance {
    data: PipelineData,
    pipeline: Option<Pipeline>,
    mesh: Option<Mesh>,
    vertex_count: u32,
    index_count: u32,
    instance_count: usize,
    instance_data: Vec<InstanceData>,
    instance_commands: Vec<InstanceCommand>,
}

impl PipelineInstance {
    pub fn find_id(shared_data: &SharedDataRw, pipeline_name: &str) -> PipelineId {
        let data = shared_data.read().unwrap();
        data.match_resource(|p: &PipelineInstance| p.get_data().name == pipeline_name)
    }

    pub fn create(shared_data: &SharedDataRw, pipeline_data: &PipelineData) -> PipelineId {
        let mut data = shared_data.write().unwrap();
        let pipeline_id = data.match_resource(|p: &PipelineInstance| {
            p.data.fragment_shader == pipeline_data.fragment_shader
                && p.data.vertex_shader == pipeline_data.vertex_shader
        });
        if pipeline_id != INVALID_UID {
            return pipeline_id;
        }
        data.add_resource(PipelineInstance {
            data: pipeline_data.clone(),
            pipeline: None,
            mesh: None,
            vertex_count: 0,
            index_count: 0,
            instance_count: 0,
            instance_data: Vec::new(),
            instance_commands: Vec::new(),
        })
    }
    pub fn init(&mut self, device: &Device) -> &mut Self {
        if self.is_initialized() {
            return self;
        }

        if self.mesh.is_none() {
            let mut mesh = Mesh::create(device);
            mesh.fill_mesh_with_max_buffers();
            mesh.finalize();
            self.mesh = Some(mesh);
        }

        let render_pass = RenderPass::create_default(&device, &self.data.data);
        let pipeline = Pipeline::create(
            &device,
            self.data.vertex_shader.clone(),
            self.data.fragment_shader.clone(),
            render_pass,
        );
        self.pipeline.get_or_insert(pipeline);
        self
    }
    pub fn recreate(&mut self, device: &Device) -> &mut Self {
        if let Some(pipeline) = &mut self.pipeline {
            let render_pass = RenderPass::create_default(device, &self.data.data);
            pipeline.recreate(render_pass);
        }
        self
    }
    pub fn prepare(&mut self) -> &mut Self {
        self.vertex_count = 0;
        self.index_count = 0;
        self.instance_count = 0;
        self
    }
    pub fn is_initialized(&self) -> bool {
        self.pipeline.is_some()
    }
    pub fn is_empty(&self) -> bool {
        self.instance_commands.is_empty() || self.vertex_count.is_zero()
    }
    pub fn get_data(&self) -> &PipelineData {
        &self.data
    }
    pub fn get_instance_data(&self) -> &Vec<InstanceData> {
        &self.instance_data
    }
    pub fn get_instance_commands(&self) -> &Vec<InstanceCommand> {
        &self.instance_commands
    }
    pub fn get_instance_count(&self) -> usize {
        self.instance_count
    }

    pub fn add_mesh_instance(
        &mut self,
        mesh_instance: &MeshInstance,
        diffuse_color: Vector4,
        diffuse_texture_index: i32,
        diffuse_layer_index: i32,
    ) -> &mut Self {
        if let Some(mesh) = &mut self.mesh {
            let mesh_data_ref = mesh.bind_at_index(
                &mesh_instance.get_data().vertices,
                self.vertex_count,
                &mesh_instance.get_data().indices,
                self.index_count,
            );
            self.vertex_count += mesh_instance.get_data().vertices.len() as u32;
            self.index_count += mesh_instance.get_data().indices.len() as u32;

            let command = InstanceCommand {
                mesh_index: self.instance_count,
                mesh_data_ref,
            };
            let data = InstanceData {
                transform: mesh_instance.get_transform(),
                diffuse_color,
                diffuse_texture_index,
                diffuse_layer_index,
            };
            if self.instance_count >= self.instance_commands.len() {
                self.instance_commands.push(command);
                self.instance_data.push(data);
            } else {
                self.instance_commands[self.instance_count] = command;
                self.instance_data[self.instance_count] = data;
            }
            self.instance_count += 1;
        }
        self
    }

    pub fn begin(&mut self) -> &mut Self {
        if let Some(pipeline) = &mut self.pipeline {
            pipeline.begin(&self.instance_commands, &self.instance_data);
        }
        self
    }
    pub fn bind_vertices(&mut self) -> &mut Self {
        if let Some(mesh) = &mut self.mesh {
            mesh.bind_vertices();
        }
        self
    }
    pub fn bind_indices(&mut self) -> &mut Self {
        if let Some(mesh) = &mut self.mesh {
            mesh.bind_indices();
        }
        self
    }
    pub fn bind_indirect(&mut self) -> &mut Self {
        if let Some(pipeline) = &mut self.pipeline {
            pipeline.bind_indirect();
        }
        self
    }
    pub fn draw_indirect(&mut self) -> &mut Self {
        if let Some(pipeline) = &mut self.pipeline {
            pipeline.draw_indirect(self.instance_count);
        }
        self
    }
    pub fn update_uniform_buffer(&mut self, cam_pos: Vector3) -> &mut Self {
        if let Some(pipeline) = &mut self.pipeline {
            pipeline.update_uniform_buffer(cam_pos);
        }
        self
    }
    pub fn update_descriptor_sets(&mut self, textures: &[TextureAtlas]) -> &mut Self {
        if let Some(pipeline) = &mut self.pipeline {
            pipeline.update_descriptor_sets(textures);
        }
        self
    }
    pub fn end(&mut self) -> &mut Self {
        if let Some(pipeline) = &mut self.pipeline {
            pipeline.end();
        }
        self
    }
}
