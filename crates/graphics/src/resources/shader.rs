use std::path::{Path, PathBuf};

use inox_messenger::MessageHubRc;

use inox_resources::{
    DataTypeResource, ResourceId, ResourceTrait, SerializableResource, SharedData, SharedDataRc,
};
use inox_serialize::{inox_serializable::SerializableRegistryRc, read_from_file};
use wgpu::ShaderModule;

use crate::{RenderContext, ShaderData, SHADER_EXTENSION};

pub type ShaderId = ResourceId;

pub struct Shader {
    path: PathBuf,
    data: ShaderData,
    module: Option<ShaderModule>,
}

impl Clone for Shader {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            data: self.data.clone(),
            module: None,
        }
    }
}

impl SerializableResource for Shader {
    fn set_path(&mut self, path: &Path) {
        self.path = path.to_path_buf();
    }
    fn path(&self) -> &Path {
        self.path.as_path()
    }

    fn extension() -> &'static str {
        SHADER_EXTENSION
    }
}

impl DataTypeResource for Shader {
    type DataType = ShaderData;
    type OnCreateData = ();

    fn new(_id: ResourceId, _shared_data: &SharedDataRc, _message_hub: &MessageHubRc) -> Self {
        Self {
            path: PathBuf::new(),
            data: ShaderData::default(),
            module: None,
        }
    }

    fn invalidate(&mut self) -> &mut Self {
        self.module = None;
        self
    }
    fn is_initialized(&self) -> bool {
        self.module.is_some()
    }
    fn deserialize_data(
        path: &std::path::Path,
        registry: &SerializableRegistryRc,
        f: Box<dyn FnMut(Self::DataType) + 'static>,
    ) {
        read_from_file::<Self::DataType>(path, registry, f);
    }
    fn on_create(
        &mut self,
        _shared_data_rc: &SharedDataRc,
        _message_hub: &MessageHubRc,
        _id: &ShaderId,
        _on_create_data: Option<&<Self as ResourceTrait>::OnCreateData>,
    ) {
    }
    fn on_destroy(
        &mut self,
        _shared_data: &SharedData,
        _message_hub: &MessageHubRc,
        _id: &ShaderId,
    ) {
        self.module = None;
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
        let mut shader = Self::new(id, shared_data, message_hub);
        shader.data = data;
        shader
    }
}

impl Shader {
    pub fn init(&mut self, context: &RenderContext) -> bool {
        if self.data.spirv_code.is_empty() {
            return false;
        }
        if self.module.is_none() {
            unsafe {
                let module =
                    context
                        .device
                        .create_shader_module_spirv(&wgpu::ShaderModuleDescriptorSpirV {
                            label: Some("Shader"),
                            source: std::borrow::Cow::Borrowed(self.data.spirv_code.as_slice()),
                        });
                self.module = Some(module);
            }
        }
        true
    }
    pub fn module(&self) -> &ShaderModule {
        self.module.as_ref().unwrap()
    }
}
