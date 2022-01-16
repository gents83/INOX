use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
};

use sabi_commands::CommandParser;
use sabi_messenger::{implement_message, Message, MessageFromString, MessengerRw};

use crate::{ResourceId, ResourceTrait, SerializableResource, SharedDataRc};

pub trait Function<T>:
    Fn(&mut T, &ResourceId, Option<&<T as ResourceTrait>::OnCreateData>)
where
    T: ResourceTrait,
{
    fn as_boxed(&self) -> Box<dyn Function<T>>;
}
impl<F, T> Function<T> for F
where
    F: 'static + Fn(&mut T, &ResourceId, Option<&<T as ResourceTrait>::OnCreateData>) + Clone,
    T: ResourceTrait,
{
    fn as_boxed(&self) -> Box<dyn Function<T>> {
        Box::new(self.clone())
    }
}
impl<T> Clone for Box<dyn Function<T>>
where
    T: ResourceTrait,
{
    fn clone(&self) -> Self {
        (**self).as_boxed()
    }
}

pub struct LoadResourceEvent<T>
where
    T: SerializableResource,
{
    path: PathBuf,
    creation_data: Option<<T as ResourceTrait>::OnCreateData>,
    resource_type: PhantomData<T>,
}
implement_message!(LoadResourceEvent<SerializableResource>);

impl<T> Clone for LoadResourceEvent<T>
where
    T: SerializableResource,
{
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            creation_data: self.creation_data.clone(),
            resource_type: PhantomData::<T>::default(),
        }
    }
}

impl<T> MessageFromString for LoadResourceEvent<T>
where
    T: SerializableResource,
{
    fn from_command_parser(command_parser: CommandParser) -> Option<Box<dyn Message>>
    where
        Self: Sized,
    {
        if command_parser.has("load_file") {
            let values = command_parser.get_values_of::<String>("load_file");
            let path = PathBuf::from(values[0].as_str());
            let extension = path
                .extension()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default();
            if extension == T::extension() {
                return Some(LoadResourceEvent::<T>::new(path.as_path(), None).as_boxed());
            }
        }
        None
    }
}

unsafe impl<T> Send for LoadResourceEvent<T> where T: SerializableResource {}
unsafe impl<T> Sync for LoadResourceEvent<T> where T: SerializableResource {}

impl<T> LoadResourceEvent<T>
where
    T: SerializableResource,
{
    pub fn new(path: &Path, creation_data: Option<<T as ResourceTrait>::OnCreateData>) -> Self {
        Self {
            resource_type: PhantomData::<T>::default(),
            path: path.to_path_buf(),
            creation_data,
        }
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
    pub fn on_create_data(&self) -> Option<&<T as ResourceTrait>::OnCreateData> {
        self.creation_data.as_ref()
    }
}

#[derive(Clone)]
pub struct UpdateResourceEvent {
    pub path: PathBuf,
}
implement_message!(UpdateResourceEvent);

impl MessageFromString for UpdateResourceEvent {
    fn from_command_parser(command_parser: CommandParser) -> Option<Box<dyn Message>>
    where
        Self: Sized,
    {
        if command_parser.has("reload_file") {
            let values = command_parser.get_values_of::<String>("reload_file");
            return Some(
                UpdateResourceEvent {
                    path: PathBuf::from(values[0].as_str()),
                }
                .as_boxed(),
            );
        }
        None
    }
}

pub trait ResourceEventHandler {
    fn is_handled(&self, msg: &dyn Message) -> bool;
    fn handle_event(
        &self,
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
        msg: &dyn Message,
    ) -> bool;
}

#[derive(Default, Clone)]
pub struct TypedResourceEventHandler<T>
where
    T: SerializableResource,
{
    marker: PhantomData<T>,
}

impl<T> ResourceEventHandler for TypedResourceEventHandler<T>
where
    T: SerializableResource,
{
    fn is_handled(&self, msg: &dyn Message) -> bool {
        msg.as_any()
            .downcast_ref::<LoadResourceEvent<T>>()
            .is_some()
    }

    fn handle_event(
        &self,
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
        msg: &dyn Message,
    ) -> bool {
        if let Some(e) = msg.as_any().downcast_ref::<LoadResourceEvent<T>>() {
            if T::is_matching_extension(e.path.as_path()) {
                T::create_from_file(
                    shared_data,
                    global_messenger,
                    e.path.as_path(),
                    e.clone().on_create_data(),
                );
                return true;
            }
        }
        false
    }
}
