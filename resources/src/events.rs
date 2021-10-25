use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
};

use nrg_messenger::{implement_message, Message, MessengerRw};

use crate::{Resource, ResourceTrait, SerializableResource, SharedDataRc};

pub trait Function<T>: Fn(&Resource<T>)
where
    T: ResourceTrait,
{
    fn as_boxed(&self) -> Box<dyn Function<T>>;
}
impl<F, T> Function<T> for F
where
    F: 'static + Fn(&Resource<T>) + Clone,
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

#[derive(Clone)]
pub struct LoadResourceEvent<T>
where
    T: ResourceTrait,
{
    path: PathBuf,
    on_loaded: Option<Box<dyn Function<T>>>,
    resource_type: PhantomData<T>,
}
unsafe impl<T> Send for LoadResourceEvent<T> where T: ResourceTrait {}
unsafe impl<T> Sync for LoadResourceEvent<T> where T: ResourceTrait {}

impl<T> LoadResourceEvent<T>
where
    T: ResourceTrait,
{
    pub fn new(path: &Path, f: Option<Box<dyn Function<T>>>) -> Self {
        Self {
            resource_type: PhantomData::<T>::default(),
            path: path.to_path_buf(),
            on_loaded: if let Some(f) = f {
                Some(Box::new(f))
            } else {
                None
            },
        }
    }

    pub fn call(&self, resource: &Resource<T>) {
        if let Some(on_loaded_callback) = &self.on_loaded {
            on_loaded_callback.as_ref()(resource);
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
                let resource = T::create_from_file(shared_data, global_messenger, e.path.as_path());
                e.call(&resource);
                return true;
            }
        }
        false
    }
}
