use std::path::Path;

use inox_filesystem::{convert_from_local_path, File};
use inox_messenger::MessageHubRc;

use inox_log::debug_log;
use inox_uid::generate_uid_from_string;

use crate::{
    DataTypeResourceEvent, OnCreateData, Resource, ResourceId, ResourceTrait,
    SerializableResourceEvent, SharedData, SharedDataRc,
};

pub const DATA_RAW_FOLDER: &str = "data_raw";
pub const DATA_FOLDER: &str = "data";

pub const PC_FOLDER: &str = "pc";
pub const WEB_FOLDER: &str = "web";

pub struct Data {}
pub trait DataTypeResource: ResourceTrait {
    type DataType;

    fn new(id: ResourceId, shared_data: &SharedDataRc, message_hub: &MessageHubRc) -> Self;

    fn create_from_data(
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        id: ResourceId,
        data: &Self::DataType,
    ) -> Self
    where
        Self: Sized;

    fn new_resource(
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        id: ResourceId,
        data: &Self::DataType,
        on_create_data: Option<OnCreateData<Self>>,
    ) -> Resource<Self>
    where
        Self: Sized + 'static + Clone,
        <Self as DataTypeResource>::DataType: Send + Sync,
    {
        let mut resource = Self::create_from_data(shared_data, message_hub, id, data);
        if let Some(mut on_create_data) = on_create_data {
            on_create_data.call_func(&mut resource);
        }
        let resource = shared_data.add_resource(message_hub, id, resource);
        if crate::DEBUG_RESOURCES {
            inox_log::debug_log!(
                "Created resource {:?} with id {:?}",
                <Self as ResourceTrait>::typename(),
                id
            );
        }
        resource
    }
}

pub trait SerializableResource: DataTypeResource + Sized + Clone
where
    Self: 'static,
{
    fn set_path(&mut self, path: &Path) -> &mut Self;
    fn path(&self) -> &Path;
    fn extension() -> &'static str;
    fn deserialize_data(path: &Path, f: Box<dyn FnMut(Self::DataType) + 'static>);
    fn is_matching_extension(path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            if let Some(ext) = extension.to_str() {
                return ext == Self::extension();
            }
        } else {
            debug_log!("No extension found for {:?}", path);
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
    fn create_from_file<'a>(
        shared_data: &'a SharedDataRc,
        message_hub: &'a MessageHubRc,
        filepath: &'a Path,
        mut on_create_data: Option<OnCreateData<Self>>,
    ) where
        Self: Sized + DataTypeResource,
        <Self as DataTypeResource>::DataType: Send + Sync,
        Self: 'a,
    {
        let path = convert_from_local_path(Data::platform_data_folder().as_path(), filepath);
        if !File::new(path.as_path()).exists() {
            panic!(
                "Unable to create_from_file with an invalid path {:?}\nCombining {:?} with {:?}",
                path,
                Data::platform_data_folder().as_path(),
                filepath
            );
        }
        //debug_log!("Creating resource : {:?}", filepath);
        let cloned_shared_data = shared_data.clone();
        let cloned_message_hub = message_hub.clone();
        let cloned_path = path.clone();
        Self::deserialize_data(
            path.as_path(),
            Box::new(move |data| {
                let resource_id = generate_uid_from_string(cloned_path.as_path().to_str().unwrap());
                let resource = Self::new_resource(
                    &cloned_shared_data,
                    &cloned_message_hub,
                    resource_id,
                    &data,
                    on_create_data.take(),
                );
                resource.get_mut().set_path(cloned_path.as_path());
                cloned_message_hub
                    .send_event(DataTypeResourceEvent::<Self>::Loaded(resource_id, data));
                if crate::DEBUG_RESOURCES {
                    inox_log::debug_log!(
                        "Loaded resource {:?} with id {:?} form path {:?}",
                        <Self as ResourceTrait>::typename(),
                        resource_id,
                        cloned_path.as_path()
                    );
                }
            }),
        );
    }

    fn request_load(
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        filepath: &Path,
        on_create_data: Option<OnCreateData<Self>>,
    ) -> Resource<Self>
    where
        Self: Sized + DataTypeResource + 'static,
    {
        let path = convert_from_local_path(Data::platform_data_folder().as_path(), filepath);
        if !File::new(path.as_path()).exists() {
            panic!(
                "Unable to create_from_file with an invalid path {:?}\nCombining {:?} with {:?}",
                path,
                Data::platform_data_folder().as_path(),
                filepath
            );
        }
        let resource_id = generate_uid_from_string(path.as_path().to_str().unwrap());
        if SharedData::has::<Self>(shared_data, &resource_id) {
            return SharedData::get_resource::<Self>(shared_data, &resource_id).unwrap();
        }
        let resource = shared_data.add_resource(
            message_hub,
            resource_id,
            Self::new(resource_id, shared_data, message_hub),
        );
        message_hub.send_event(SerializableResourceEvent::<Self>::Load(
            path,
            on_create_data,
        ));
        resource
    }
}
