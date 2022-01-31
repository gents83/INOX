use std::{marker::PhantomData, path::PathBuf};

use sabi_commands::CommandParser;
use sabi_messenger::{implement_message, Message, MessageFromString, MessengerRw};

use crate::{Resource, ResourceId, ResourceTrait, SerializableResource, SharedDataRc};

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

pub enum ResourceEvent<T>
where
    T: ResourceTrait,
{
    Load(PathBuf, Option<<T as ResourceTrait>::OnCreateData>),
    Created(Resource<T>),
    Changed(ResourceId),
    Destroyed(ResourceId),
}
implement_message!(ResourceEvent<ResourceTrait>);

impl<T> Clone for ResourceEvent<T>
where
    T: ResourceTrait,
{
    fn clone(&self) -> Self {
        match self {
            ResourceEvent::Load(path, on_create_data) => {
                ResourceEvent::Load(path.clone(), on_create_data.clone())
            }
            ResourceEvent::Created(resource) => ResourceEvent::Created(resource.clone()),
            ResourceEvent::Changed(id) => ResourceEvent::Changed(*id),
            ResourceEvent::Destroyed(id) => ResourceEvent::Destroyed(*id),
        }
    }
}
unsafe impl<T> Send for ResourceEvent<T> where T: ResourceTrait {}
unsafe impl<T> Sync for ResourceEvent<T> where T: ResourceTrait {}

impl<T> MessageFromString for ResourceEvent<T>
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
                return Some(
                    ResourceEvent::<T>::Load(path.as_path().to_path_buf(), None).as_boxed(),
                );
            }
        }
        None
    }
}

#[derive(Clone)]
pub struct ReloadEvent {
    pub path: PathBuf,
}
implement_message!(ReloadEvent);

impl MessageFromString for ReloadEvent {
    fn from_command_parser(command_parser: CommandParser) -> Option<Box<dyn Message>>
    where
        Self: Sized,
    {
        if command_parser.has("reload_file") {
            let values = command_parser.get_values_of::<String>("reload_file");
            return Some(
                ReloadEvent {
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
        msg.as_any().downcast_ref::<ResourceEvent<T>>().is_some()
    }

    fn handle_event(
        &self,
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
        msg: &dyn Message,
    ) -> bool {
        if let Some(ResourceEvent::Load(path, on_create_data)) =
            msg.as_any().downcast_ref::<ResourceEvent<T>>()
        {
            if T::is_matching_extension(path.as_path()) {
                T::create_from_file(
                    shared_data,
                    global_messenger,
                    path.as_path(),
                    on_create_data.as_ref(),
                );
                return true;
            }
        }
        false
    }
}
