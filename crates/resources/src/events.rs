use std::{marker::PhantomData, path::PathBuf};

use inox_commands::CommandParser;
use inox_messenger::{implement_message, Listener, MessageHubRc};

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

pub trait DeserializeFunction: FnOnce(&SharedDataRc, &MessageHubRc) + Send + Sync {}
impl<F> DeserializeFunction for F where F: FnOnce(&SharedDataRc, &MessageHubRc) + Send + Sync {}

pub trait LoadFunction: Fn(Box<dyn DeserializeFunction>) + Send + Sync {}
impl<F> LoadFunction for F where F: Fn(Box<dyn DeserializeFunction>) + Clone + Send + Sync {}

pub enum ResourceEvent<T>
where
    T: ResourceTrait,
{
    Load(PathBuf, Option<<T as ResourceTrait>::OnCreateData>),
    Created(Resource<T>),
    Changed(ResourceId),
    Destroyed(ResourceId),
}
implement_message!(
    ResourceEvent<ResourceTrait>,
    ResourceEvent<SerializableResource>,
    message_from_command_parser,
    compare_and_discard
);

unsafe impl<T> Send for ResourceEvent<T> where T: ResourceTrait {}
unsafe impl<T> Sync for ResourceEvent<T> where T: ResourceTrait {}

impl<T> ResourceEvent<T>
where
    T: ResourceTrait,
{
    fn compare_and_discard(&self, other: &Self) -> bool {
        match self {
            Self::Load(path, _on_create_data) => match other {
                Self::Load(other_path, _other_on_create_data) => path == other_path,
                _ => false,
            },
            Self::Created(resource) => match other {
                Self::Created(other_resource) => resource.id() == other_resource.id(),
                _ => false,
            },
            Self::Changed(id) => match other {
                Self::Changed(other_id) => id == other_id,
                _ => false,
            },
            Self::Destroyed(id) => match other {
                Self::Destroyed(other_id) => id == other_id,
                _ => false,
            },
        }
    }
}

impl<T> ResourceEvent<T>
where
    T: SerializableResource,
{
    fn message_from_command_parser(command_parser: CommandParser) -> Option<Self> {
        if command_parser.has("load_file") {
            let values = command_parser.get_values_of::<String>("load_file");
            let path = PathBuf::from(values[0].as_str());
            let extension = path
                .extension()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default();
            if extension == T::extension() {
                return Some(ResourceEvent::<T>::Load(path.as_path().to_path_buf(), None));
            }
        }
        None
    }
}

#[derive(Clone)]
pub struct ReloadEvent {
    pub path: PathBuf,
}
implement_message!(
    ReloadEvent,
    message_from_command_parser,
    compare_and_discard
);

impl ReloadEvent {
    fn compare_and_discard(&self, other: &Self) -> bool {
        self.path == other.path
    }
    fn message_from_command_parser(command_parser: CommandParser) -> Option<Self> {
        if command_parser.has("reload_file") {
            let values = command_parser.get_values_of::<String>("reload_file");
            return Some(ReloadEvent {
                path: PathBuf::from(values[0].as_str()),
            });
        }
        None
    }
}

pub trait ResourceEventHandler {
    fn handle_events(&self, f: &dyn LoadFunction);
}

pub struct TypedResourceEventHandler<T>
where
    T: ResourceTrait,
{
    marker: PhantomData<T>,
    listener: Listener,
}

impl<T> TypedResourceEventHandler<T>
where
    T: ResourceTrait,
{
    pub fn new(message_hub: &MessageHubRc) -> Self {
        let listener = Listener::new(message_hub);
        listener.register::<ResourceEvent<T>>();
        TypedResourceEventHandler {
            marker: PhantomData::<T>::default(),
            listener,
        }
    }
}

impl<T> ResourceEventHandler for TypedResourceEventHandler<T>
where
    T: SerializableResource,
{
    fn handle_events(&self, f: &dyn LoadFunction) {
        self.listener.process_messages(|msg: &ResourceEvent<T>| {
            if let ResourceEvent::<T>::Load(path, on_create_data) = msg {
                if T::is_matching_extension(path.as_path()) {
                    let p = path.clone();
                    let on_create_data = on_create_data.clone();
                    f(Box::new(move |shared_data, message_hub| {
                        T::create_from_file(
                            shared_data,
                            message_hub,
                            p.as_path(),
                            on_create_data.as_ref(),
                        );
                    }));
                }
            }
        });
    }
}
