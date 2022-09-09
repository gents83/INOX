use std::path::Path;

use inox_filesystem::{convert_from_local_path, File};
use inox_messenger::MessageHubRc;

use inox_log::debug_log;
use inox_serialize::inox_serializable::SerializableRegistryRc;
use inox_uid::generate_uid_from_string;

use crate::{
    DataTypeResourceEvent, Resource, ResourceEvent, ResourceId, ResourceTrait,
    SerializableResourceEvent, SharedData, SharedDataRc,
};

pub const DATA_RAW_FOLDER: &str = "data_raw";
pub const DATA_FOLDER: &str = "data";

pub const PC_FOLDER: &str = "pc";
pub const WEB_FOLDER: &str = "web";

pub struct Data {}
pub trait DataTypeResource: ResourceTrait
where
    <Self as DataTypeResource>::OnCreateData: Clone,
{
    type DataType;
    type OnCreateData;

    fn new(id: ResourceId, shared_data: &SharedDataRc, message_hub: &MessageHubRc) -> Self;

    fn is_initialized(&self) -> bool;
    fn invalidate(&mut self) -> &mut Self;

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
        data: Self::DataType,
        on_create_data: Option<<Self as ResourceTrait>::OnCreateData>,
    ) -> Resource<Self>
    where
        Self: Sized + 'static,
        <Self as DataTypeResource>::DataType: Send + Sync,
    {
        let mut resource = Self::create_from_data(shared_data, message_hub, id, &data);
        resource.on_create(shared_data, message_hub, &id, on_create_data.as_ref());
        let resource = shared_data.add_resource(message_hub, id, resource);

        message_hub.send_event(ResourceEvent::<Self>::Created(
            shared_data.get_resource(&id).unwrap(),
        ));
        message_hub.send_event(DataTypeResourceEvent::<Self>::Loaded(id, data));
        resource
    }
}

pub trait SerializableResource: DataTypeResource + Sized {
    fn set_path(&mut self, path: &Path) -> &mut Self;
    fn path(&self) -> &Path;
    fn extension() -> &'static str;
    fn deserialize_data(
        path: &Path,
        registry: &SerializableRegistryRc,
        f: Box<dyn FnMut(Self::DataType) + 'static>,
    );
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
    fn create_from_file(
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        filepath: &Path,
        mut on_create_data: Option<<Self as ResourceTrait>::OnCreateData>,
    ) where
        Self: Sized + DataTypeResource + 'static,
        <Self as DataTypeResource>::DataType: Send + Sync,
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
            shared_data.serializable_registry(),
            Box::new(move |data| {
                let resource_id = generate_uid_from_string(cloned_path.as_path().to_str().unwrap());
                let resource = Self::new_resource(
                    &cloned_shared_data,
                    &cloned_message_hub,
                    resource_id,
                    data,
                    on_create_data.take(),
                );
                resource.get_mut().set_path(cloned_path.as_path());
            }),
        );
    }

    fn request_load(
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        filepath: &Path,
        on_create_data: Option<<Self as ResourceTrait>::OnCreateData>,
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
