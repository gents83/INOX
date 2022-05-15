use std::path::PathBuf;

use inox_messenger::MessageHubRc;
use inox_resources::{
    DataTypeResource, Resource, ResourceId, ResourceTrait, SerializableResource, SharedData,
    SharedDataRc,
};
use inox_serialize::{inox_serializable::SerializableRegistryRc, read_from_file};

use crate::{ComputePassData, Pipeline, RenderContext};

pub type ComputePassId = ResourceId;

#[derive(Clone)]
pub struct ComputePass {
    shared_data: SharedDataRc,
    message_hub: MessageHubRc,
    data: ComputePassData,
    pipelines: Vec<Resource<Pipeline>>,
    is_initialized: bool,
}

impl ResourceTrait for ComputePass {
    type OnCreateData = ();

    fn on_create(
        &mut self,
        _shared_data_rc: &SharedDataRc,
        _message_hub: &MessageHubRc,
        _id: &ComputePassId,
        _on_create_data: Option<&<Self as ResourceTrait>::OnCreateData>,
    ) {
    }
    fn on_destroy(
        &mut self,
        _shared_data: &SharedData,
        _message_hub: &MessageHubRc,
        _id: &ComputePassId,
    ) {
    }
    fn on_copy(&mut self, other: &Self)
    where
        Self: Sized,
    {
        *self = other.clone();
    }
}

impl DataTypeResource for ComputePass {
    type DataType = ComputePassData;
    type OnCreateData = <Self as ResourceTrait>::OnCreateData;

    fn new(_id: ResourceId, shared_data: &SharedDataRc, message_hub: &MessageHubRc) -> Self {
        Self {
            shared_data: shared_data.clone(),
            message_hub: message_hub.clone(),
            data: ComputePassData::default(),
            pipelines: Vec::new(),
            is_initialized: false,
        }
    }
    fn invalidate(&mut self) -> &mut Self {
        self.is_initialized = false;
        self
    }
    fn is_initialized(&self) -> bool {
        self.is_initialized
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
        _id: ResourceId,
        data: Self::DataType,
    ) -> Self
    where
        Self: Sized,
    {
        let pipelines = data.pipelines.clone();
        let mut pass = Self {
            shared_data: shared_data.clone(),
            message_hub: message_hub.clone(),
            data,
            pipelines: Vec::new(),
            is_initialized: false,
        };
        pass.pipelines(pipelines);
        pass
    }
}

impl ComputePass {
    pub fn data(&self) -> &ComputePassData {
        &self.data
    }
    pub fn pipeline(&self, index: usize) -> Option<&Resource<Pipeline>> {
        self.pipelines.get(index)
    }
    pub fn pipelines(&mut self, pipelines: Vec<PathBuf>) -> &mut Self {
        self.pipelines.clear();
        pipelines.iter().for_each(|path| {
            if !path.as_os_str().is_empty() {
                let pipeline = Pipeline::request_load(
                    &self.shared_data,
                    &self.message_hub,
                    path.as_path(),
                    None,
                );
                self.pipelines.push(pipeline);
            };
        });
        self
    }

    pub fn init(&mut self, _context: &mut RenderContext) {
        if self.is_initialized {
            return;
        }
        self.is_initialized = true;
    }
}
