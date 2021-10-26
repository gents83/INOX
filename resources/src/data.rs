use std::path::{Path, PathBuf};

use nrg_filesystem::convert_from_local_path;
use nrg_messenger::{send_global_event, MessengerRw};
use nrg_profiler::debug_log;
use nrg_serialize::generate_uid_from_string;

use crate::{
    Function, LoadResourceEvent, Resource, ResourceId, ResourceTrait, SharedData, SharedDataRc,
};

pub const DATA_RAW_FOLDER: &str = "./data_raw/";
pub const DATA_FOLDER: &str = "./data/";

pub trait Data {
    #[inline]
    fn get_data_folder(&self) -> PathBuf {
        PathBuf::from(DATA_FOLDER)
    }
}

pub trait DataTypeResource: ResourceTrait + Default + Clone {
    type DataType;

    fn is_initialized(&self) -> bool;
    fn invalidate(&mut self);
    fn deserialize_data(path: &Path) -> Self::DataType;

    fn create_from_data(
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
        id: ResourceId,
        data: Self::DataType,
    ) -> Resource<Self>
    where
        Self: Sized;
}

impl<T> ResourceTrait for T where T: DataTypeResource {}

pub trait SerializableResource: DataTypeResource + Sized {
    fn set_path(&mut self, path: &Path);
    fn path(&self) -> &Path;
    fn is_matching_extension(path: &Path) -> bool
    where
        Self: Sized;

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
    ) -> Resource<Self>
    where
        Self: Sized + DataTypeResource,
    {
        let path = convert_from_local_path(PathBuf::from(DATA_FOLDER).as_path(), filepath);
        if !path.exists() || !path.is_file() {
            panic!(
                "Unable to create_from_file with an invalid path {}",
                path.to_str().unwrap()
            );
        }
        let data = Self::deserialize_data(path.as_path());
        let resource_id = generate_uid_from_string(path.as_path().to_str().unwrap());
        let resource = Self::create_from_data(shared_data, global_messenger, resource_id, data);
        debug_log(format!("Created resource {:?}", path.as_path()).as_str());
        resource.get_mut(|r| r.set_path(path.as_path()));
        resource
    }

    fn load_from_file(
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
        filepath: &Path,
        on_loaded_callback: Option<Box<dyn Function<Self>>>,
    ) -> Resource<Self>
    where
        Self: Sized + DataTypeResource,
    {
        let path = convert_from_local_path(PathBuf::from(DATA_FOLDER).as_path(), filepath);
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
        send_global_event(
            &global_messenger,
            LoadResourceEvent::<Self>::new(path.as_path(), on_loaded_callback),
        );
        resource
    }
}
