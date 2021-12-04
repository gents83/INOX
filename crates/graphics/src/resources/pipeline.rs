use std::path::{Path, PathBuf};

use sabi_math::matrix4_to_array;
use sabi_messenger::MessengerRw;
use sabi_profiler::debug_log;
use sabi_resources::{DataTypeResource, ResourceId, SerializableResource, SharedDataRc};
use sabi_serialize::{read_from_file, SerializeFile};

use crate::{
    api::backend::{self, BackendPhysicalDevice, BackendPipeline},
    utils::compute_color_from_id,
    CommandBuffer, Device, DrawMode, GraphicsMesh, InstanceCommand, InstanceData, Mesh,
    MeshBindingData, MeshCategoryId, MeshId, PipelineBindingData, PipelineData, RenderPass,
    ShaderMaterialData, ShaderType, TextureAtlas, INVALID_INDEX,
};

pub type PipelineId = ResourceId;

#[derive(Default, Clone)]
pub struct Pipeline {
    path: PathBuf,
    backend_pipeline: Option<backend::BackendPipeline>,
    data: PipelineData,
    is_initialized: bool,
    mesh: GraphicsMesh,
    vertex_count: u32,
    index_count: u32,
    instance_count: usize,
    instance_data: Vec<InstanceData>,
    instance_commands: Vec<InstanceCommand>,
    last_binding_index: i32,
}

impl SerializableResource for Pipeline {
    fn set_path(&mut self, path: &Path) {
        self.path = path.to_path_buf();
    }
    fn path(&self) -> &Path {
        self.path.as_path()
    }

    fn extension() -> &'static str {
        PipelineData::extension()
    }
}

impl DataTypeResource for Pipeline {
    type DataType = PipelineData;

    fn invalidate(&mut self) {
        self.is_initialized = false;
        self.last_binding_index = -1;
    }
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
    fn deserialize_data(path: &Path) -> Self::DataType {
        read_from_file::<Self::DataType>(path)
    }

    fn create_from_data(
        _shared_data: &SharedDataRc,
        _global_messenger: &MessengerRw,
        _id: ResourceId,
        data: Self::DataType,
    ) -> Self
    where
        Self: Sized,
    {
        let canonicalized_pipeline_data = data.canonicalize_paths();
        Self {
            data: canonicalized_pipeline_data,
            ..Default::default()
        }
    }
}

impl Pipeline {
    pub fn data(&self) -> &PipelineData {
        &self.data
    }
    pub fn init(
        &mut self,
        device: &Device,
        physical_device: &BackendPhysicalDevice,
        render_pass: &RenderPass,
    ) -> &mut Self {
        if self.data.vertex_shader.to_str().unwrap().is_empty()
            || self.data.fragment_shader.to_str().unwrap().is_empty()
        {
            return self;
        }
        self.invalidate();
        if let Some(backend_pipeline) = &mut self.backend_pipeline {
            backend_pipeline.destroy(device);
        }
        let mut backend_pipeline = BackendPipeline::default();
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
        backend_pipeline.build(device, physical_device, &*render_pass, &self.data);
        self.backend_pipeline = Some(backend_pipeline);

        self.is_initialized = true;

        self
    }

    pub fn check_shaders_to_reload(&mut self, path_as_string: String) {
        if path_as_string.contains(self.data.vertex_shader.to_str().unwrap())
            && !self.data.vertex_shader.to_str().unwrap().is_empty()
        {
            self.invalidate();
            debug_log(format!("VertexShader {:?} will be reloaded", path_as_string).as_str());
        }
        if path_as_string.contains(self.data.fragment_shader.to_str().unwrap())
            && !self.data.fragment_shader.to_str().unwrap().is_empty()
        {
            self.invalidate();
            debug_log(format!("FragmentShader {:?} will be reloaded", path_as_string).as_str());
        }
        if path_as_string.contains(self.data.tcs_shader.to_str().unwrap())
            && !self.data.tcs_shader.to_str().unwrap().is_empty()
        {
            self.invalidate();
            debug_log(
                format!(
                    "TessellationControlShader {:?} will be reloaded",
                    path_as_string
                )
                .as_str(),
            );
        }
        if path_as_string.contains(self.data.tes_shader.to_str().unwrap())
            && !self.data.tes_shader.to_str().unwrap().is_empty()
        {
            self.invalidate();
            debug_log(
                format!(
                    "TessellationEvaluationShader {:?} will be reloaded",
                    path_as_string
                )
                .as_str(),
            );
        }
        if path_as_string.contains(self.data.geometry_shader.to_str().unwrap())
            && !self.data.geometry_shader.to_str().unwrap().is_empty()
        {
            self.invalidate();
            debug_log(format!("GeometryShader {:?} will be reloaded", path_as_string).as_str());
        }
    }

    pub fn prepare(&mut self) -> &mut Self {
        self.vertex_count = 0;
        self.index_count = 0;
        self.instance_count = 0;
        self.mesh.reset_mesh_categories();
        self
    }

    pub fn find_used_textures(
        &self,
        textures: &[TextureAtlas],
        material_data: &[ShaderMaterialData],
    ) -> Vec<bool> {
        let mut used_textures = Vec::new();
        used_textures.resize_with(textures.len(), || false);
        self.instance_data.iter().for_each(|data| {
            if data.material_index != INVALID_INDEX {
                material_data[data.material_index as usize]
                    .textures_indices
                    .iter()
                    .for_each(|index| {
                        if *index != INVALID_INDEX {
                            used_textures[*index as usize] = true;
                        }
                    });
            }
        });
        used_textures
    }

    pub fn update_bindings(
        &mut self,
        device: &Device,
        command_buffer: &CommandBuffer,
        binding_data: PipelineBindingData,
    ) -> &mut Self {
        sabi_profiler::scoped_profile!("device::update_bindings");
        if let Some(backend_pipeline) = &mut self.backend_pipeline {
            backend_pipeline
                .update_data_buffer(
                    device,
                    binding_data.light_data,
                    binding_data.texture_data,
                    binding_data.material_data,
                )
                .update_constant_data(
                    command_buffer,
                    binding_data.width,
                    binding_data.height,
                    binding_data.view,
                    binding_data.proj,
                )
                .update_descriptor_sets(device, binding_data.textures, binding_data.used_textures);
            self.last_binding_index =
                backend_pipeline.bind_descriptors(device, command_buffer) as _;
        }
        self
    }

    pub fn bind(&mut self, command_buffer: &CommandBuffer) -> &mut Self {
        sabi_profiler::scoped_profile!(format!("pipeline::bind[{:?}]", self.name()).as_str());
        if let Some(backend_pipeline) = &mut self.backend_pipeline {
            backend_pipeline.bind_pipeline(command_buffer);
        }
        self
    }

    fn bind_indirect(
        &mut self,
        device: &Device,
        physical_device: &BackendPhysicalDevice,
    ) -> &mut Self {
        sabi_profiler::scoped_profile!(
            format!("pipeline::bind_indirect[{:?}]", self.name()).as_str()
        );
        if let Some(backend_pipeline) = &mut self.backend_pipeline {
            backend_pipeline.bind_indirect(
                device,
                physical_device,
                &self.instance_commands,
                &self.instance_data,
            );
        }
        self
    }

    fn bind_instance_buffer(&mut self, command_buffer: &CommandBuffer) -> &mut Self {
        sabi_profiler::scoped_profile!(format!(
            "pipeline::bind_instance_buffer[{:?}]",
            self.name()
        )
        .as_str());
        if let Some(backend_pipeline) = &mut self.backend_pipeline {
            backend_pipeline.bind_instance_buffer(command_buffer);
        }
        self
    }
    fn bind_vertices(&mut self, command_buffer: &CommandBuffer) -> &mut Self {
        sabi_profiler::scoped_profile!(
            format!("pipeline::bind_vertices[{:?}]", self.name()).as_str()
        );
        self.mesh.bind_vertices(command_buffer);
        self
    }
    fn bind_indices(&mut self, command_buffer: &CommandBuffer) -> &mut Self {
        sabi_profiler::scoped_profile!(
            format!("pipeline::bind_indices[{:?}]", self.name()).as_str()
        );
        self.mesh.bind_indices(command_buffer);
        self
    }

    fn draw_single(&mut self, command_buffer: &CommandBuffer) -> &mut Self {
        sabi_profiler::scoped_profile!(
            format!("pipeline::draw_indexed[{:?}]", self.name()).as_str()
        );
        if let Some(backend_pipeline) = &mut self.backend_pipeline {
            backend_pipeline.draw_single(
                command_buffer,
                self.instance_commands.as_slice(),
                self.instance_data.as_slice(),
                self.instance_count,
            );
        }
        self
    }

    fn draw_indirect_batch(&mut self, command_buffer: &CommandBuffer) -> &mut Self {
        sabi_profiler::scoped_profile!(
            format!("pipeline::draw_indirect[{:?}]", self.name()).as_str()
        );
        if let Some(backend_pipeline) = &mut self.backend_pipeline {
            backend_pipeline.draw_indirect_batch(command_buffer, self.instance_count);
        }
        self
    }

    pub fn add_mesh_instance(
        &mut self,
        device: &Device,
        physical_device: &BackendPhysicalDevice,
        mesh_id: &MeshId,
        mesh: &Mesh,
        material_index: i32,
    ) -> &mut Self {
        if !self.is_initialized()
            || mesh.mesh_data().vertices.is_empty()
            || mesh.mesh_data().indices.is_empty()
        {
            return self;
        }

        sabi_profiler::scoped_profile!(
            format!("pipeline::add_mesh_instance[{}]", self.name()).as_str()
        );

        let mesh_data_ref = self.mesh.bind_at_index(
            device,
            physical_device,
            MeshBindingData {
                mesh_category_identifier: *mesh.category_identifier(),
                vertices: &mesh.mesh_data().vertices,
                first_vertex: self.vertex_count,
                indices: &mesh.mesh_data().indices,
                first_index: self.index_count,
            },
        );

        self.vertex_count += mesh.mesh_data().vertices.len() as u32;
        self.index_count += mesh.mesh_data().indices.len() as u32;
        let mesh_index = if mesh.draw_index() >= 0 {
            mesh.draw_index() as usize
        } else {
            self.instance_count
        };

        let command = InstanceCommand {
            mesh_index,
            mesh_data_ref,
        };

        let data = InstanceData {
            id: compute_color_from_id(mesh_id.as_u128() as _),
            matrix: matrix4_to_array(mesh.matrix()),
            draw_area: mesh.draw_area(),
            material_index,
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
        sabi_profiler::scoped_profile!(
            format!("renderer::draw_pipeline[{:?}]", self.name()).as_str()
        );

        self.bind_indirect(device, physical_device)
            .bind_vertices(command_buffer)
            .bind_instance_buffer(command_buffer)
            .bind_indices(command_buffer);

        if self.data.draw_mode == DrawMode::Single {
            self.draw_single(command_buffer);
        } else {
            self.draw_indirect_batch(command_buffer);
        }
    }
}
