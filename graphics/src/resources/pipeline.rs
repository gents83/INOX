use nrg_math::{Mat4Ops, Matrix4, Vector4};
use nrg_resources::{
    DataTypeResource, ResourceData, ResourceId, ResourceRef, SharedData, SharedDataRw,
};
use nrg_serialize::{generate_random_uid, INVALID_UID};

use crate::{
    api::backend, utils::compute_color_from_id, Device, GraphicsMesh, InstanceCommand,
    InstanceData, Mesh, MeshCategoryId, PipelineData, RenderPass, ShaderType, TextureAtlas,
};

pub type PipelineId = ResourceId;
pub type PipelineRc = ResourceRef<Pipeline>;

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

impl Default for Pipeline {
    fn default() -> Self {
        Self {
            id: INVALID_UID,
            data: PipelineData::default(),
            is_initialized: false,
            backend_pipeline: None,
            mesh: GraphicsMesh::default(),
            vertex_count: 0,
            index_count: 0,
            instance_count: 0,
            instance_data: Vec::new(),
            instance_commands: Vec::new(),
        }
    }
}

impl ResourceData for Pipeline {
    fn id(&self) -> ResourceId {
        self.id
    }
}

impl DataTypeResource for Pipeline {
    type DataType = PipelineData;
    fn create_from_data(shared_data: &SharedDataRw, pipeline_data: Self::DataType) -> PipelineRc {
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
    fn find_from_data(
        shared_data: &SharedDataRw,
        pipeline_data: &PipelineData,
    ) -> Option<PipelineRc> {
        SharedData::match_resource(shared_data, |p: &Pipeline| {
            pipeline_data.has_same_shaders(&p.data)
        })
    }
    pub fn data(&self) -> &PipelineData {
        &self.data
    }
    pub fn init(&mut self, device: &Device, render_pass: &RenderPass) -> &mut Self {
        if let Some(backend_pipeline) = &mut self.backend_pipeline {
            backend_pipeline.destroy();
        }

        let mut backend_pipeline = backend::BackendPipeline::create(&*device);

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
            .set_shader(ShaderType::Vertex, self.data.vertex_shader.as_path())
            .set_shader(ShaderType::Fragment, self.data.fragment_shader.as_path());
        if !self.data.tcs_shader.to_str().unwrap().is_empty() {
            backend_pipeline.set_shader(
                ShaderType::TessellationControl,
                self.data.tcs_shader.as_path(),
            );
        }
        if !self.data.tes_shader.to_str().unwrap().is_empty() {
            backend_pipeline.set_shader(
                ShaderType::TessellationEvaluation,
                self.data.tes_shader.as_path(),
            );
        }
        if !self.data.geometry_shader.to_str().unwrap().is_empty() {
            backend_pipeline.set_shader(ShaderType::Geometry, self.data.geometry_shader.as_path());
        }
        backend_pipeline.build(&*device, &*render_pass, &self.data.culling, &self.data.mode);
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

    fn begin(&mut self) -> &mut Self {
        nrg_profiler::scoped_profile!(
            format!("renderer::draw_pipeline_begin[{:?}]", self.id()).as_str()
        );
        if let Some(backend_pipeline) = &mut self.backend_pipeline {
            backend_pipeline
                .bind(&self.instance_commands, &self.instance_data)
                .bind_descriptors();
        }
        self
    }

    fn update_runtime_data(
        &self,
        width: u32,
        height: u32,
        view: &Matrix4,
        proj: &Matrix4,
    ) -> &Self {
        nrg_profiler::scoped_profile!(
            format!("pipeline::update_runtime_data[{:?}]", self.id()).as_str()
        );
        if let Some(backend_pipeline) = &self.backend_pipeline {
            backend_pipeline
                .update_constant_data(width, height, view, proj)
                .update_uniform_buffer(view, proj);
        }
        self
    }
    fn update_descriptor_sets(&self, textures: &[TextureAtlas]) -> &Self {
        nrg_profiler::scoped_profile!(format!(
            "pipeline::update_descriptors_sets[{:?}]",
            self.id()
        )
        .as_str());
        if let Some(backend_pipeline) = &self.backend_pipeline {
            backend_pipeline.update_descriptor_sets(textures);
        }
        self
    }

    fn bind_indirect(&mut self) -> &mut Self {
        nrg_profiler::scoped_profile!(format!("pipeline::bind_indirect[{:?}]", self.id()).as_str());
        if let Some(backend_pipeline) = &self.backend_pipeline {
            backend_pipeline.bind_indirect();
        }
        self
    }
    fn bind_vertices(&mut self, device: &Device) -> &mut Self {
        nrg_profiler::scoped_profile!(format!("pipeline::bind_vertices[{:?}]", self.id()).as_str());
        self.mesh.bind_vertices(device);
        self
    }
    fn bind_indices(&mut self, device: &Device) -> &mut Self {
        nrg_profiler::scoped_profile!(format!("pipeline::bind_indices[{:?}]", self.id()).as_str());
        self.mesh.bind_indices(device);
        self
    }

    fn draw_indirect(&mut self) -> &mut Self {
        nrg_profiler::scoped_profile!(format!("pipeline::draw_indirect[{:?}]", self.id()).as_str());
        if let Some(backend_pipeline) = &mut self.backend_pipeline {
            backend_pipeline.draw_indirect(self.instance_count);
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

    pub fn draw(
        &mut self,
        device: &Device,
        width: u32,
        height: u32,
        view: &Matrix4,
        proj: &Matrix4,
        textures: &[TextureAtlas],
    ) {
        nrg_profiler::scoped_profile!(format!("renderer::draw_pipeline[{:?}]", self.id()).as_str());

        self.update_runtime_data(width, height, view, proj)
            .update_descriptor_sets(textures);

        self.begin()
            .bind_vertices(device)
            .bind_indirect()
            .bind_indices(device)
            .draw_indirect()
            .end();
    }
}
