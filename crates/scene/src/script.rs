use std::path::{Path, PathBuf};

use inox_messenger::MessageHubRc;
use inox_nodes::LogicData;
use inox_resources::{
    DataTypeResource, Handle, Resource, ResourceId, ResourceTrait, SerializableResource,
    SharedData, SharedDataRc,
};
use inox_serialize::{inox_serializable::SerializableRegistryRc, read_from_file, SerializeFile};
use inox_time::Timer;

use crate::{Object, ObjectId};

pub type ScriptId = ResourceId;

#[derive(Clone)]
pub struct OnScriptCreateData {
    pub parent_id: ObjectId,
}

#[derive(Clone)]
pub struct Script {
    filepath: PathBuf,
    parent: Handle<Object>,
    logic: LogicData,
}

impl SerializableResource for Script {
    fn path(&self) -> &Path {
        self.filepath.as_path()
    }

    fn set_path(&mut self, path: &Path) -> &mut Self {
        self.filepath = path.to_path_buf();
        self
    }

    fn extension() -> &'static str {
        LogicData::extension()
    }

    fn deserialize_data(
        path: &std::path::Path,
        registry: &SerializableRegistryRc,
        f: Box<dyn FnMut(Self::DataType) + 'static>,
    ) {
        read_from_file::<Self::DataType>(path, registry, f);
    }
}

impl ResourceTrait for Script {
    type OnCreateData = OnScriptCreateData;

    fn on_create(
        &mut self,
        shared_data_rc: &SharedDataRc,
        _message_hub: &MessageHubRc,
        _id: &ObjectId,
        on_create_data: Option<&<Self as ResourceTrait>::OnCreateData>,
    ) {
        if let Some(on_create_data) = on_create_data {
            if let Some(parent) = shared_data_rc.get_resource::<Object>(&on_create_data.parent_id) {
                self.set_parent(&parent);
            }
        }
    }
    fn on_destroy(
        &mut self,
        _shared_data: &SharedData,
        _message_hub: &MessageHubRc,
        _id: &ObjectId,
    ) {
    }
    fn on_copy(&mut self, other: &Self)
    where
        Self: Sized,
    {
        *self = other.clone();
    }
}

impl DataTypeResource for Script {
    type DataType = LogicData;
    type OnCreateData = <Self as ResourceTrait>::OnCreateData;

    fn new(_id: ResourceId, _shared_data_rc: &SharedDataRc, _message_hub: &MessageHubRc) -> Self {
        Self {
            filepath: PathBuf::new(),
            parent: None,
            logic: LogicData::default(),
        }
    }

    fn is_initialized(&self) -> bool {
        self.logic.is_initialized()
    }
    fn invalidate(&mut self) -> &mut Self {
        eprintln!("Script cannot be invalidated!");
        self
    }
    fn create_from_data(
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        id: ScriptId,
        data: &Self::DataType,
    ) -> Self {
        let mut script = Self::new(id, shared_data, message_hub);
        let mut logic = data.clone();
        logic.init();
        script.logic = logic;
        script
    }
}

impl Script {
    pub const LOGIC_OBJECT: &'static str = "logic_object";

    #[inline]
    pub fn set_parent(&mut self, parent: &Resource<Object>) -> &mut Self {
        self.parent = Some(parent.clone());
        self.logic
            .context_mut()
            .set(Script::LOGIC_OBJECT, parent.clone());
        self
    }

    pub fn update(&mut self, timer: &Timer) {
        if self.logic.is_initialized() {
            self.logic.execute(timer.dt());
        }
    }
}
