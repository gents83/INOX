use std::path::{Path, PathBuf};

use inox_math::Vector3;
use inox_messenger::MessageHubRc;
use inox_resources::{
    DataTypeResource, ResourceEvent, ResourceId, ResourceTrait, SerializableResource, SharedData,
    SharedDataRc,
};
use inox_serialize::{inox_serializable::SerializableRegistryRc, read_from_file, SerializeFile};

use crate::{LightData, INVALID_INDEX};

pub type LightId = ResourceId;

#[derive(Clone)]
pub struct OnLightCreateData {
    pub position: Vector3,
}

#[derive(Clone)]
pub struct Light {
    filepath: PathBuf,
    id: LightId,
    message_hub: MessageHubRc,
    data: LightData,
    light_index: i32,
    is_active: bool,
}

impl ResourceTrait for Light {
    type OnCreateData = OnLightCreateData;

    fn on_create(
        &mut self,
        _shared_data_rc: &SharedDataRc,
        _message_hub: &MessageHubRc,
        _id: &LightId,
        on_create_data: Option<&<Self as ResourceTrait>::OnCreateData>,
    ) {
        if let Some(on_create_data) = on_create_data {
            self.set_position(on_create_data.position);
        }
    }
    fn on_destroy(
        &mut self,
        _shared_data: &SharedData,
        _message_hub: &MessageHubRc,
        _id: &LightId,
    ) {
    }
    fn on_copy(&mut self, other: &Self)
    where
        Self: Sized,
    {
        *self = other.clone();
    }
}

impl SerializableResource for Light {
    fn path(&self) -> &Path {
        self.filepath.as_path()
    }

    fn set_path(&mut self, path: &Path) -> &mut Self {
        self.filepath = path.to_path_buf();
        self
    }

    fn extension() -> &'static str {
        LightData::extension()
    }

    fn deserialize_data(
        path: &std::path::Path,
        registry: &SerializableRegistryRc,
        f: Box<dyn FnMut(Self::DataType) + 'static>,
    ) {
        read_from_file::<Self::DataType>(path, registry, f);
    }
}
impl DataTypeResource for Light {
    type DataType = LightData;
    type OnCreateData = <Self as ResourceTrait>::OnCreateData;

    fn new(id: ResourceId, _shared_data: &SharedDataRc, message_hub: &MessageHubRc) -> Self {
        Self {
            id,
            filepath: PathBuf::new(),
            data: LightData::default(),
            light_index: INVALID_INDEX,
            is_active: true,
            message_hub: message_hub.clone(),
        }
    }

    fn is_initialized(&self) -> bool {
        self.light_index != INVALID_INDEX
    }

    fn invalidate(&mut self) -> &mut Self {
        self.light_index = INVALID_INDEX;
        self
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
        let mut light = Self::new(id, shared_data, message_hub);
        light.data = *data;
        light
    }
}

impl Light {
    pub fn mark_as_dirty(&self) -> &Self {
        self.message_hub
            .send_event(ResourceEvent::<Self>::Changed(self.id));
        self
    }
    #[inline]
    pub fn set_position(&mut self, position: Vector3) -> &mut Self {
        let p = position.into();
        if self.data.position != p {
            self.data.position = p;
            self.mark_as_dirty();
        }
        self
    }

    #[inline]
    pub fn data(&self) -> &LightData {
        &self.data
    }

    #[inline]
    pub fn data_mut(&mut self) -> &mut LightData {
        &mut self.data
    }

    #[inline]
    pub fn set_active(&mut self, is_active: bool) -> &mut Self {
        if self.is_active != is_active {
            self.is_active = is_active;
            self.mark_as_dirty();
        }
        self
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn set_light_index(&mut self, light_index: u32) {
        self.light_index = light_index as _;
    }
    pub fn light_index(&self) -> i32 {
        self.light_index
    }
}
