use std::path::{Path, PathBuf};

use inox_log::debug_log;
use inox_messenger::MessageHubRc;
use inox_resources::{
    DataTypeResource, Handle, Resource, ResourceId, ResourceTrait, SerializableResource,
    SharedData, SharedDataRc,
};
use inox_serialize::{inox_serializable::SerializableRegistryRc, read_from_file, SerializeFile};

use crate::{BindingData, ComputePipelineData, RenderContext, Shader, SHADER_ENTRY_POINT};

pub type ComputePipelineId = ResourceId;

pub struct ComputePipeline {
    path: PathBuf,
    shared_data: SharedDataRc,
    message_hub: MessageHubRc,
    data: ComputePipelineData,
    shader: Handle<Shader>,
    compute_pipeline: Option<wgpu::ComputePipeline>,
}

impl Clone for ComputePipeline {
    fn clone(&self) -> Self {
        let shader = Self::load_shaders(&self.data, &self.shared_data, &self.message_hub);
        Self {
            path: self.path.clone(),
            data: self.data.clone(),
            shared_data: self.shared_data.clone(),
            message_hub: self.message_hub.clone(),
            shader: Some(shader),
            compute_pipeline: None,
        }
    }
}

impl ResourceTrait for ComputePipeline {
    type OnCreateData = ();

    fn on_create(
        &mut self,
        _shared_data_rc: &SharedDataRc,
        _message_hub: &MessageHubRc,
        _id: &ComputePipelineId,
        _on_create_data: Option<&<Self as ResourceTrait>::OnCreateData>,
    ) {
    }
    fn on_destroy(
        &mut self,
        _shared_data: &SharedData,
        _message_hub: &MessageHubRc,
        _id: &ComputePipelineId,
    ) {
        self.compute_pipeline = None;
        self.shader = None;
    }
    fn on_copy(&mut self, other: &Self)
    where
        Self: Sized,
    {
        *self = other.clone();
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
}

impl DataTypeResource for ComputePipeline {
    type DataType = ComputePipelineData;
    type OnCreateData = <Self as ResourceTrait>::OnCreateData;

    fn new(_id: ResourceId, shared_data: &SharedDataRc, message_hub: &MessageHubRc) -> Self {
        Self {
            path: PathBuf::new(),
            shared_data: shared_data.clone(),
            message_hub: message_hub.clone(),
            data: ComputePipelineData::default(),
            shader: None,
            compute_pipeline: None,
        }
    }

    fn invalidate(&mut self) -> &mut Self {
        self.compute_pipeline = None;
        self
    }
    fn is_initialized(&self) -> bool {
        self.data.is_valid() && self.shader.is_some() && self.compute_pipeline.is_some()
    }
    fn deserialize_data(
        path: &std::path::Path,
        registry: &SerializableRegistryRc,
        f: Box<dyn FnMut(Self::DataType) + 'static>,
    ) {
        read_from_file::<Self::DataType>(path, registry, f);
    }

    fn create_from_data(
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        id: ResourceId,
        data: Self::DataType,
    ) -> Self
    where
        Self: Sized,
    {
        let data = data.canonicalize_paths();
        let mut pipeline = Self::new(id, shared_data, message_hub);
        pipeline.data = data;
        let shader = Self::load_shaders(&pipeline.data, shared_data, message_hub);
        pipeline.shader = Some(shader);
        pipeline
    }
}

impl ComputePipeline {
    pub fn data(&self) -> &ComputePipelineData {
        &self.data
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
    pub fn init(&mut self, context: &RenderContext, binding_data: &BindingData) -> bool {
        inox_profiler::scoped_profile!("compute_pipeline::init");
        if self.shader.is_none() {
            return false;
        }
        if let Some(shader) = self.shader.as_ref() {
            if !shader.get_mut().init(context) {
                return false;
            }
        }
        if self.is_initialized() {
            return true;
        }
        let compute_pipeline_layout =
            context
                .core
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Compute Pipeline Layout"),
                    bind_group_layouts: binding_data
                        .bind_group_layouts()
                        .iter()
                        .collect::<Vec<_>>()
                        .as_slice(),
                    push_constant_ranges: &[],
                });

        let compute_pipeline = {
            inox_profiler::scoped_profile!("compute_pipeline::crate[{}]", self.name());
            context
                .core
                .device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
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

                    module: self.shader.as_ref().unwrap().get().module(),
                    entry_point: SHADER_ENTRY_POINT,
                })
        };
        self.compute_pipeline = Some(compute_pipeline);
        true
    }

    pub fn check_shaders_to_reload(&mut self, path_as_string: String) {
        if path_as_string.contains(self.data.shader.to_str().unwrap())
            && !self.data.shader.to_str().unwrap().is_empty()
        {
            self.invalidate();
            debug_log!("Compute Shader {:?} will be reloaded", path_as_string);
        }
    }
}
