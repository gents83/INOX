use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
};

use nrg_messenger::{implement_message, Message, MessengerRw};

use crate::{ResourceTrait, SerializableResource, SharedDataRc};

#[derive(Clone)]
pub struct LoadResourceEvent<T>
where
    T: ResourceTrait,
{
    path: PathBuf,
    resource_type: PhantomData<T>,
}
unsafe impl<T> Send for LoadResourceEvent<T> where T: ResourceTrait {}
unsafe impl<T> Sync for LoadResourceEvent<T> where T: ResourceTrait {}

impl<T> LoadResourceEvent<T>
where
    T: ResourceTrait,
{
    pub fn new(path: &Path) -> Self {
        Self {
            resource_type: PhantomData::<T>::default(),
            path: path.to_path_buf(),
        }
    }
}

#[derive(Clone)]
pub struct UpdateResourceEvent {
    pub path: PathBuf,
}
implement_message!(LoadResourceEvent<ResourceTrait>);
implement_message!(UpdateResourceEvent);

pub trait ResourceEventHandler {
    fn is_handled(&self, msg: &dyn Message) -> bool;
    fn as_boxed(&self) -> Box<dyn ResourceEventHandler>;
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
    fn as_boxed(&self) -> Box<dyn ResourceEventHandler> {
        Box::new(self.clone())
    }

    fn handle_event(
        &self,
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
        msg: &dyn Message,
    ) -> bool {
        if let Some(e) = msg.as_any().downcast_ref::<LoadResourceEvent<T>>() {
            if T::is_matching_extension(e.path.as_path()) {
                T::create_from_file(shared_data, global_messenger, e.path.as_path());
                return true;
            }
        }
        false
    }
}
