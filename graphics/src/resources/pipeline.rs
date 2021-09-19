use nrg_math::{Mat4Ops, Vector4};
use nrg_resources::{
    DataTypeResource, Handle, Resource, ResourceData, ResourceId, SharedData, SharedDataRw,
};
use nrg_serialize::generate_random_uid;

use crate::{
    api::backend::{self, BackendPhysicalDevice, BackendPipeline},
    utils::compute_color_from_id,
    CommandBuffer, Device, GraphicsMesh, InstanceCommand, InstanceData, Mesh, MeshCategoryId,
    PipelineData, RenderPass, ShaderType,
};

pub type PipelineId = ResourceId;

#[derive(Default)]
pub struct Pipeline {
    id: ResourceId,
    backend_pipeline: Option<backend::BackendPipeline>,
    data: PipelineData,
    is_initialized: bool,
    mesh: GraphicsMesh,
    vertex_count: u32,
    index_count: u32,
    instance_count: usize,
    instance_data: Vec<InstanceData>,
    instance_commands: Vec<InstanceCommand>,
}

impl ResourceData for Pipeline {
    fn id(&self) -> ResourceId {
        self.id
    }
}

impl DataTypeResource for Pipeline {
    type DataType = PipelineData;
    fn create_from_data(
        shared_data: &SharedDataRw,
        pipeline_data: Self::DataType,
    ) -> Resource<Self> {
        let canonicalized_pipeline_data = pipeline_data.canonicalize_paths();
        if let Some(pipeline) = Pipeline::find_from_data(shared_data, &canonicalized_pipeline_data)
        {
            return pipeline;
        }
        SharedData::add_resource(
            shared_data,
            Pipeline {
                id: generate_random_uid(),
                data: canonicalized_pipeline_data,
                ..Default::default()
            },
        )
    }
}

impl Pipeline {
    fn find_from_data(shared_data: &SharedDataRw, pipeline_data: &PipelineData) -> Handle<Self> {
        SharedData::match_resource(shared_data, |p: &Pipeline| {
            pipeline_data.has_same_shaders(&p.data)
        })
    }
    pub fn data(&self) -> &PipelineData {
        &self.data
    }
    pub fn init(&mut self, device: &Device, render_pass: &RenderPass) -> &mut Self {
        if let Some(backend_pipeline) = &mut self.backend_pipeline {
            backend_pipeline.destroy(device);
        }

        let mut backend_pipeline = BackendPipeline::default();

        if self.data.vertex_shader.to_str().unwrap().is_empty() {
            eprintln!(
                "Trying to init a pipeline {:?} with NO vertex shader",
                self.id()
            );
        }
        if self.data.fragment_shader.to_str().unwrap().is_empty() {
            eprintln!(
                "Trying to init a pipeline {:?} with NO fragment shader",
                self.id()
            );
        }
        backend_pipeline
            .set_shader(
                device,
                ShaderType::Vertex,
                self.data.vertex_shader.as_path(),
            )
            .set_shader(
                device,
                ShaderType::Fragment,
                self.data.fragment_shader.as_path(),
            );
        if !self.data.tcs_shader.to_str().unwrap().is_empty() {
            backend_pipeline.set_shader(
                device,
                ShaderType::TessellationControl,
                self.data.tcs_shader.as_path(),
            );
        }
        if !self.data.tes_shader.to_str().unwrap().is_empty() {
            backend_pipeline.set_shader(
                device,
                ShaderType::TessellationEvaluation,
                self.data.tes_shader.as_path(),
            );
        }
        if !self.data.geometry_shader.to_str().unwrap().is_empty() {
            backend_pipeline.set_shader(
                device,
                ShaderType::Geometry,
                self.data.geometry_shader.as_path(),
            );
        }
        backend_pipeline.build(
            device,
            &*render_pass,
            &self.data.culling,
            &self.data.mode,
            &self.data.src_color_blend_factor,
            &self.data.dst_color_blend_factor,
            &self.data.src_alpha_blend_factor,
            &self.data.dst_alpha_blend_factor,
        );
        self.backend_pipeline = Some(backend_pipeline);

        self.is_initialized = true;

        self
    }

    pub fn invalidate(&mut self) {
        self.is_initialized = false;
    }

    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    pub fn check_shaders_to_reload(&mut self, path_as_string: String) {
        if path_as_string.contains(self.data.vertex_shader.to_str().unwrap())
            && !self.data.vertex_shader.to_str().unwrap().is_empty()
        {
            self.invalidate();
            println!("VertexShader {:?} will be reloaded", path_as_string);
        }
        if path_as_string.contains(self.data.fragment_shader.to_str().unwrap())
            && !self.data.fragment_shader.to_str().unwrap().is_empty()
        {
            self.invalidate();
            println!("FragmentShader {:?} will be reloaded", path_as_string);
        }
        if path_as_string.contains(self.data.tcs_shader.to_str().unwrap())
            && !self.data.tcs_shader.to_str().unwrap().is_empty()
        {
            self.invalidate();
            println!(
                "TessellationControlShader {:?} will be reloaded",
                path_as_string
            );
        }
        if path_as_string.contains(self.data.tes_shader.to_str().unwrap())
            && !self.data.tes_shader.to_str().unwrap().is_empty()
        {
            self.invalidate();
            println!(
                "TessellationEvaluationShader {:?} will be reloaded",
                path_as_string
            );
        }
        if path_as_string.contains(self.data.geometry_shader.to_str().unwrap())
            && !self.data.geometry_shader.to_str().unwrap().is_empty()
        {
            self.invalidate();
            println!("GeometryShader {:?} will be reloaded", path_as_string);
        }
    }

    pub fn prepare(&mut self) -> &mut Self {
        self.vertex_count = 0;
        self.index_count = 0;
        self.instance_count = 0;
        self.mesh.reset_mesh_categories();
        self
    }

    fn begin(
        &mut self,
        device: &Device,
        physical_device: &BackendPhysicalDevice,
        command_buffer: &CommandBuffer,
    ) -> &mut Self {
        nrg_profiler::scoped_profile!(
            format!("renderer::draw_pipeline_begin[{:?}]", self.id()).as_str()
        );
        if let Some(backend_pipeline) = &mut self.backend_pipeline {
            backend_pipeline.bind_indirect(
                device,
                physical_device,
                command_buffer,
                &self.instance_commands,
                &self.instance_data,
            );
        }
        device.bind_descriptors(command_buffer);
        self
    }

    fn bind_instance_buffer(&mut self, command_buffer: &CommandBuffer) -> &mut Self {
        nrg_profiler::scoped_profile!(format!("pipeline::bind_indirect[{:?}]", self.id()).as_str());
        if let Some(backend_pipeline) = &mut self.backend_pipeline {
            backend_pipeline.bind_instance_buffer(command_buffer);
        }
        self
    }
    fn bind_vertices(&mut self, command_buffer: &CommandBuffer) -> &mut Self {
        nrg_profiler::scoped_profile!(format!("pipeline::bind_vertices[{:?}]", self.id()).as_str());
        self.mesh.bind_vertices(command_buffer);
        self
    }
    fn bind_indices(&mut self, command_buffer: &CommandBuffer) -> &mut Self {
        nrg_profiler::scoped_profile!(format!("pipeline::bind_indices[{:?}]", self.id()).as_str());
        self.mesh.bind_indices(command_buffer);
        self
    }

    #[allow(dead_code)]
    fn draw_indexed(&mut self, command_buffer: &CommandBuffer) -> &mut Self {
        nrg_profiler::scoped_profile!(format!("pipeline::draw_indexed[{:?}]", self.id()).as_str());
        if let Some(backend_pipeline) = &mut self.backend_pipeline {
            backend_pipeline.draw_indexed(
                command_buffer,
                self.instance_commands.as_slice(),
                self.instance_count,
            );
        }
        self
    }

    fn draw_indirect(&mut self, command_buffer: &CommandBuffer) -> &mut Self {
        nrg_profiler::scoped_profile!(format!("pipeline::draw_indirect[{:?}]", self.id()).as_str());
        if let Some(backend_pipeline) = &mut self.backend_pipeline {
            backend_pipeline.draw_indirect(command_buffer, self.instance_count);
        }
        self
    }

    fn end(&mut self) -> &mut Self {
        nrg_profiler::scoped_profile!(format!("pipeline::end[{:?}]", self.id()).as_str());
        self
    }

    pub fn add_mesh_instance(
        &mut self,
        device: &Device,
        physical_device: &BackendPhysicalDevice,
        mesh_instance: &Mesh,
        diffuse_color: Vector4,
        diffuse_texture_index: i32,
        diffuse_layer_index: i32,
    ) -> &mut Self {
        if mesh_instance.mesh_data().vertices.is_empty()
            || mesh_instance.mesh_data().indices.is_empty()
        {
            return self;
        }

        nrg_profiler::scoped_profile!(
            format!("pipeline::add_mesh_instance[{}]", self.id()).as_str()
        );

        let mesh_data_ref = self.mesh.bind_at_index(
            device,
            physical_device,
            mesh_instance.category_identifier(),
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

    pub fn should_draw(&self, mesh_category_to_draw: &[MeshCategoryId]) -> bool {
        let mut should_draw = false;
        mesh_category_to_draw.iter().for_each(|category_id| {
            should_draw |= self.mesh.mesh_categories().iter().any(|c| c == category_id);
        });
        should_draw
    }

    pub fn fill_command_buffer(
        &mut self,
        device: &Device,
        physical_device: &BackendPhysicalDevice,
        command_buffer: &CommandBuffer,
    ) {
        nrg_profiler::scoped_profile!(format!("renderer::draw_pipeline[{:?}]", self.id()).as_str());

        self.begin(device, physical_device, command_buffer)
            .bind_vertices(command_buffer)
            .bind_instance_buffer(command_buffer)
            .bind_indices(command_buffer)
            .draw_indirect(command_buffer);

        self.end();
    }
}
