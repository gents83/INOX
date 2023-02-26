use std::path::{Path, PathBuf};

use inox_log::debug_log;
use inox_messenger::MessageHubRc;
use inox_resources::{
    DataTypeResource, Handle, Resource, ResourceId, ResourceTrait, SerializableResource,
    SharedDataRc,
};
use inox_serialize::{inox_serializable::SerializableRegistryRc, read_from_file, SerializeFile};

use crate::{BindingData, ComputePipelineData, RenderContext, Shader, SHADER_ENTRY_POINT};

pub type ComputePipelineId = ResourceId;

pub struct ComputePipeline {
    path: PathBuf,
    shared_data: SharedDataRc,
    message_hub: MessageHubRc,
    shader: Handle<Shader>,
    compute_pipeline: Option<wgpu::ComputePipeline>,
}

impl Clone for ComputePipeline {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            shared_data: self.shared_data.clone(),
            message_hub: self.message_hub.clone(),
            shader: self.shader.clone(),
            compute_pipeline: None,
        }
    }
}

impl ResourceTrait for ComputePipeline {
    fn invalidate(&mut self) -> &mut Self {
        self.compute_pipeline = None;
        self
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
    fn deserialize_data(
        path: &std::path::Path,
        registry: &SerializableRegistryRc,
        f: Box<dyn FnMut(Self::DataType) + 'static>,
    ) {
        read_from_file::<Self::DataType>(path, registry, f);
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
            if !shader.get().is_initialized() {
                if !shader.get_mut().init(context) {
                    return false;
                }
                self.compute_pipeline = None;
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
            inox_profiler::scoped_profile!("compute_pipeline::create[{}]", self.name());
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
        if let Some(shader) = &self.shader {
            if path_as_string.contains(shader.get().path().to_str().unwrap())
                && !shader.get().path().to_str().unwrap().is_empty()
            {
                self.invalidate();
                debug_log!("Compute Shader {:?} will be reloaded", path_as_string);
            }
        }
    }
}
