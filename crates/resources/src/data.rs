use std::{
    env,
    path::{Path, PathBuf},
};

use sabi_filesystem::convert_from_local_path;
use sabi_messenger::{GlobalMessenger, MessengerRw};

use sabi_serialize::generate_uid_from_string;

use crate::{LoadResourceEvent, Resource, ResourceId, ResourceTrait, SharedData, SharedDataRc};

pub const DATA_RAW_FOLDER: &str = "data_raw";
pub const DATA_FOLDER: &str = "data";

pub struct Data {}
impl Data {
    #[inline]
    pub fn data_raw_folder() -> PathBuf {
        env::current_dir().unwrap().join(DATA_RAW_FOLDER)
    }
    #[inline]
    pub fn data_folder() -> PathBuf {
        env::current_dir().unwrap().join(DATA_FOLDER)
    }
}
pub trait DataTypeResource: ResourceTrait + Default + Clone
where
    <Self as DataTypeResource>::OnCreateData: Clone,
{
    type DataType;
    type OnCreateData;

    fn on_create(
        &mut self,
        shared_data: &SharedDataRc,
        id: &ResourceId,
        on_create_data: Option<&<Self as ResourceTrait>::OnCreateData>,
    );
    fn on_destroy(&mut self, shared_data: &SharedData, id: &ResourceId);

    fn is_initialized(&self) -> bool;
    fn invalidate(&mut self);
    fn deserialize_data(path: &Path) -> Self::DataType;

    fn create_from_data(
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
        id: ResourceId,
        data: Self::DataType,
    ) -> Self
    where
        Self: Sized;

    fn new_resource(
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
        id: ResourceId,
        data: Self::DataType,
    ) -> Resource<Self>
    where
        Self: Sized,
    {
        let resource = Self::create_from_data(shared_data, global_messenger, id, data);
        SharedData::add_resource(shared_data, id, resource)
    }
}

impl<T> ResourceTrait for T
where
    T: DataTypeResource,
    <T as DataTypeResource>::OnCreateData: Clone,
{
    type OnCreateData = <T as DataTypeResource>::OnCreateData;

    fn on_create_resource(
        &mut self,
        shared_data: &SharedDataRc,
        id: &ResourceId,
        on_create_data: Option<&<Self as ResourceTrait>::OnCreateData>,
    ) where
        Self: Sized,
    {
        self.on_create(shared_data, id, on_create_data);
    }
    fn on_destroy_resource(&mut self, shared_data: &SharedData, id: &ResourceId) {
        self.on_destroy(shared_data, id);
    }

    fn on_copy_resource(&mut self, other: &Self)
    where
        Self: Sized + Clone,
    {
        *self = other.clone();
    }
}

pub trait SerializableResource: DataTypeResource + Sized {
    fn set_path(&mut self, path: &Path);
    fn path(&self) -> &Path;
    fn extension() -> &'static str;
    fn is_matching_extension(path: &Path) -> bool {
        if let Some(ext) = path.extension().unwrap().to_str() {
            return ext == Self::extension();
        }
        false
    }

    #[inline]
    fn name(&self) -> String {
        self.path()
            .file_stem()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_string()
    }
    #[inline]
    fn create_from_file(
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
        filepath: &Path,
        on_create_data: Option<&<Self as ResourceTrait>::OnCreateData>,
    ) -> Resource<Self>
    where
        Self: Sized + DataTypeResource,
    {
        let path = convert_from_local_path(Data::data_folder().as_path(), filepath);
        if !path.exists() || !path.is_file() {
            panic!(
                "Unable to create_from_file with an invalid path {}",
                path.to_str().unwrap()
            );
        }
        let data = Self::deserialize_data(path.as_path());
        let resource_id = generate_uid_from_string(path.as_path().to_str().unwrap());
        let mut resource = Self::create_from_data(shared_data, global_messenger, resource_id, data);
        //debug_log(format!("Created resource [{:?}] {:?}", resource_id, path.as_path()).as_str());
        resource.set_path(path.as_path());

        resource.on_create(shared_data, &resource_id, on_create_data);

        shared_data.add_resource(resource_id, resource)
    }

    fn request_load(
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
        filepath: &Path,
        on_create_data: Option<<Self as ResourceTrait>::OnCreateData>,
    ) -> Resource<Self>
    where
        Self: Sized + DataTypeResource,
    {
        let path = convert_from_local_path(Data::data_folder().as_path(), filepath);
        if !path.exists() || !path.is_file() {
            panic!(
                "Unable to load_from_file with an invalid path {}",
                path.to_str().unwrap()
            );
        }
        let resource_id = generate_uid_from_string(path.as_path().to_str().unwrap());
        if SharedData::has::<Self>(shared_data, &resource_id) {
            return SharedData::get_resource::<Self>(shared_data, &resource_id).unwrap();
        }
        let resource = SharedData::add_resource(shared_data, resource_id, Self::default());
        global_messenger.send_event(LoadResourceEvent::<Self>::new(
            path.as_path(),
            on_create_data,
        ));
        resource
    }
}
