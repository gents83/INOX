use std::path::{Path, PathBuf};

use inox_log::debug_log;
use inox_messenger::MessageHubRc;
use inox_resources::{
    DataTypeResource, Handle, Resource, ResourceId, ResourceTrait, SerializableResource,
    SharedDataRc,
};
use inox_serialize::{read_from_file, SerializationType, SerializeFile};

use crate::{
    BindingData, ComputePipelineData, ReloadError, ReloadState, RenderContext, Shader,
    SHADER_ENTRY_POINT,
};

pub type ComputePipelineId = ResourceId;

pub struct ComputePipeline {
    path: PathBuf,
    shared_data: SharedDataRc,
    message_hub: MessageHubRc,
    shader: Handle<Shader>,
    compute_pipeline: Option<wgpu::ComputePipeline>,
    reload_pending: bool,
    last_reload_error: Option<ReloadError>,
    reload_state: ReloadState,
}

impl Clone for ComputePipeline {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            shared_data: self.shared_data.clone(),
            message_hub: self.message_hub.clone(),
            shader: self.shader.clone(),
            compute_pipeline: None,
            reload_pending: self.reload_pending,
            last_reload_error: self.last_reload_error.clone(),
            reload_state: self.reload_state.clone(),
        }
    }
}

impl ResourceTrait for ComputePipeline {
    fn invalidate(&mut self) -> &mut Self {
        self.compute_pipeline = None;
        self.reload_pending = false;
        self
    }

    fn replace_from(&mut self, other: &Self) {
        self.path = other.path.clone();
        self.shader = other.shader.clone();
        self.reload_pending = true;
        self.last_reload_error = None;
        self.reload_state.stage();
    }

    fn is_initialized(&self) -> bool {
        self.shader.is_some() && self.compute_pipeline.is_some()
    }
}

impl SerializableResource for ComputePipeline {
    fn set_path(&mut self, path: &Path) -> &mut Self {
        self.path = path.to_path_buf();
        self
    }
    fn path(&self) -> &Path {
        self.path.as_path()
    }

    fn extension() -> &'static str {
        ComputePipelineData::extension()
    }
    fn deserialize_data(path: &std::path::Path, f: Box<dyn FnMut(Self::DataType) + 'static>) {
        read_from_file::<Self::DataType>(path, SerializationType::Json, f);
    }
}

impl DataTypeResource for ComputePipeline {
    type DataType = ComputePipelineData;

    fn new(_id: ResourceId, shared_data: &SharedDataRc, message_hub: &MessageHubRc) -> Self {
        Self {
            path: PathBuf::new(),
            shared_data: shared_data.clone(),
            message_hub: message_hub.clone(),
            shader: None,
            compute_pipeline: None,
            reload_pending: false,
            last_reload_error: None,
            reload_state: ReloadState::default(),
        }
    }

    fn create_from_data(
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        id: ResourceId,
        data: &Self::DataType,
    ) -> Self
    where
        Self: Sized,
    {
        let data = data.canonicalize_paths();
        let mut pipeline = Self::new(id, shared_data, message_hub);
        let shader = Self::load_shaders(&data, shared_data, message_hub);
        pipeline.shader = Some(shader);
        pipeline
    }
}

impl ComputePipeline {
    #[doc(hidden)]
    pub fn from_shader(
        path: PathBuf,
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        shader: Resource<Shader>,
    ) -> Self {
        let mut pipeline = Self::new(ResourceId::default(), shared_data, message_hub);
        pipeline.path = path;
        pipeline.shader = Some(shader);
        pipeline
    }

    pub fn compute_pipeline(&self) -> &wgpu::ComputePipeline {
        self.compute_pipeline.as_ref().unwrap()
    }
    fn load_shaders(
        data: &ComputePipelineData,
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
    ) -> Resource<Shader> {
        Shader::request_load(shared_data, message_hub, data.shader.as_path(), None)
    }
    pub fn init(
        &mut self,
        context: &RenderContext,
        binding_data: &BindingData,
        entry_point_name: Option<&str>,
    ) -> bool {
        inox_profiler::scoped_profile!("compute_pipeline::init");
        if self.shader.is_none() {
            return false;
        }
        if let Some(shader) = self.shader.as_ref() {
            if shader.get().path().as_os_str().is_empty() {
                return false;
            }
            if !shader.get().is_initialized() || shader.get().has_pending_reload() {
                if let Err(error) = shader.get_mut().init(context) {
                    self.last_reload_error = Some(error.clone());
                    self.reload_state.fail(error);
                    return false;
                }
            }
        }
        if self.is_initialized() && !self.reload_pending {
            return true;
        }
        let compute_pipeline_layout =
            context
                .webgpu
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Compute Pipeline Layout"),
                    bind_group_layouts: binding_data
                        .bind_group_layouts()
                        .iter()
                        .map(Some)
                        .collect::<Vec<_>>()
                        .as_slice(),
                    ..Default::default()
                });

        let compute_pipeline = {
            inox_profiler::scoped_profile!("compute_pipeline::create[{}]", self.name());
            context.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(
                    format!(
                        "Compute Pipeline [{:?}]",
                        self.path
                            .file_stem()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default()
                    )
                    .as_str(),
                ),
                layout: Some(&compute_pipeline_layout),
                cache: None,
                module: self.shader.as_ref().unwrap().get().module(),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                entry_point: Some(entry_point_name.unwrap_or(SHADER_ENTRY_POINT)),
            })
        };
        match compute_pipeline {
            Ok(compute_pipeline) => {
                self.compute_pipeline = Some(compute_pipeline);
                self.reload_pending = false;
                self.last_reload_error = None;
                self.reload_state.commit();
                true
            }
            Err(diagnostic) => {
                let error = ReloadError::new(
                    &self.path,
                    "compute pipeline",
                    diagnostic,
                    self.compute_pipeline.is_some(),
                );
                debug_log!(
                    "Compute pipeline reload failed for {:?}: {}",
                    error.path,
                    error.diagnostic
                );
                self.last_reload_error = Some(error.clone());
                self.reload_state.fail(error);
                false
            }
        }
    }

    pub fn check_shaders_to_reload(&mut self, path_as_string: String) {
        if let Some(shader) = &self.shader {
            let shader_path = shader.get().path().to_string_lossy().into_owned();
            if !shader_path.is_empty() && path_as_string.contains(shader_path.as_str()) {
                self.reload_pending = true;
                debug_log!("Compute Shader {:?} will be reloaded", path_as_string);
            }
        }
    }

    pub fn reload_state(&self) -> &ReloadState {
        &self.reload_state
    }

    pub fn reload_state_mut(&mut self) -> &mut ReloadState {
        &mut self.reload_state
    }

    pub fn record_reload_failure(&mut self, error: ReloadError) {
        self.last_reload_error = Some(error.clone());
        self.reload_state.fail(error);
    }

    pub fn last_reload_error(&self) -> Option<&ReloadError> {
        self.last_reload_error.as_ref()
    }
}
