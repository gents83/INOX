use std::path::PathBuf;

use inox_commands::CommandParser;
use inox_messenger::implement_message;

use crate::{Resource, ResourceId, ResourceTrait, SerializableResource};

pub enum ResourceEvent<T>
where
    T: ResourceTrait,
{
    Created(Resource<T>),
    Changed(ResourceId),
    Destroyed(ResourceId),
}

implement_message!(
    ResourceEvent<ResourceTrait>,
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

    fn message_from_command_parser(_command_parser: CommandParser) -> Option<Self> {
        None
    }
}

#[derive(Clone)]
pub enum SerializableResourceEvent<T>
where
    T: SerializableResource + ?Sized,
{
    Load(PathBuf, Option<<T as ResourceTrait>::OnCreateData>),
}
implement_message!(
    SerializableResourceEvent<SerializableResource>,
    message_from_command_parser,
    compare_and_discard
);

impl<T> SerializableResourceEvent<T>
where
    T: SerializableResource,
{
    fn compare_and_discard(&self, other: &Self) -> bool {
        match self {
            Self::Load(path, _on_create_data) => match other {
                Self::Load(other_path, _other_on_create_data) => path == other_path,
            },
        }
    }
    fn message_from_command_parser(command_parser: CommandParser) -> Option<Self> {
        if command_parser.has("load_file") {
            let values = command_parser.get_values_of::<String>("load_file");
            let path = PathBuf::from(values[0].as_str());
            if <T as SerializableResource>::is_matching_extension(path.as_path()) {
                return Some(SerializableResourceEvent::<T>::Load(
                    path.as_path().to_path_buf(),
                    None,
                ));
            }
        }
        None
    }
}

#[derive(Clone)]
pub enum ReloadEvent {
    Reload(PathBuf),
}
implement_message!(
    ReloadEvent,
    message_from_command_parser,
    compare_and_discard
);

impl ReloadEvent {
    fn compare_and_discard(&self, other: &Self) -> bool {
        match self {
            Self::Reload(path) => match other {
                Self::Reload(other_path) => path == other_path,
            },
        }
    }
    fn message_from_command_parser(command_parser: CommandParser) -> Option<Self> {
        if command_parser.has("reload_file") {
            let values = command_parser.get_values_of::<String>("reload_file");
            return Some(ReloadEvent::Reload(PathBuf::from(values[0].as_str())));
        }
        None
    }
}
