use nrg_messenger::implement_message;
use nrg_serialize::Uid;
use std::{
    any::Any,
    path::PathBuf,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::{GenericRef, HandleCastTo, ResourceRef};

#[derive(Clone)]
pub enum ResourceEvent {
    Reload(PathBuf),
}
implement_message!(ResourceEvent);

pub type ResourceId = Uid;

pub trait ResourceData: Send + Sync + 'static {
    fn id(&self) -> ResourceId;
}

pub trait BaseResource: Send + Sync + Any {
    fn as_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
}

pub struct ResourceMutex<T>
where
    T: ResourceData,
{
    data: RwLock<T>,
}

impl<T> ResourceMutex<T>
where
    T: ResourceData,
{
    pub fn new(data: T) -> Self {
        Self {
            data: RwLock::new(data),
        }
    }

    pub fn get(&self) -> RwLockReadGuard<'_, T> {
        self.data.read().unwrap()
    }

    pub fn get_mut(&self) -> RwLockWriteGuard<'_, T> {
        self.data.write().unwrap()
    }
}

impl<T> BaseResource for ResourceMutex<T>
where
    T: ResourceData,
{
    fn as_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}

impl<T> ResourceData for Resource<T>
where
    T: ResourceData,
{
    fn id(&self) -> ResourceId {
        self.data.read().unwrap().id()
    }
}

pub type Resource<T> = Arc<ResourceMutex<T>>;
pub type GenericResource = Arc<dyn BaseResource>;

pub trait ResourceCastTo {
    fn of_type<T: ResourceData>(self) -> Resource<T>;
}

impl ResourceCastTo for GenericResource {
    fn of_type<T: ResourceData>(self) -> Resource<T> {
        let any = Arc::into_raw(self.as_any());
        Arc::downcast(unsafe { Arc::from_raw(any) }).unwrap()
    }
}

pub trait TypedStorage {
    fn as_any(self: Box<Self>) -> Box<dyn Any>;
    fn add(&mut self, handle: GenericRef, data: GenericResource);
    fn resource(&self, resource_id: ResourceId) -> GenericResource;
    fn get(&self, resource_id: ResourceId) -> GenericRef;
    fn remove_all(&mut self);
    fn flush(&mut self);
    fn remove(&mut self, resource_id: ResourceId);
    fn has(&self, resource_id: ResourceId) -> bool;
    fn handles(&self) -> Vec<GenericRef>;
    fn resources(&self) -> Vec<GenericResource>;
    fn count(&self) -> usize;
}

pub struct Storage<T>
where
    T: ResourceData,
{
    handles: Vec<ResourceRef<T>>,
    resources: Vec<Resource<T>>,
}

impl<T> Default for Storage<T>
where
    T: ResourceData,
{
    fn default() -> Self {
        Self {
            handles: Vec::new(),
            resources: Vec::new(),
        }
    }
}

impl<T> TypedStorage for Storage<T>
where
    T: ResourceData + Sized + 'static,
{
    fn as_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
    fn add(&mut self, handle: GenericRef, resource: GenericResource) {
        self.handles.push(handle.of_type::<T>());
        self.resources.push(resource.of_type::<T>());
    }
    fn resource(&self, resource_id: ResourceId) -> GenericResource
    where
        T: ResourceData,
    {
        if let Some(resource) = self.resources.iter().find(|r| r.id() == resource_id) {
            resource.clone()
        } else {
            panic!("Resource {} not found", resource_id.to_simple());
        }
    }
    fn get(&self, resource_id: ResourceId) -> GenericRef
    where
        T: ResourceData,
    {
        if let Some(handle) = self.handles.iter().find(|h| h.id() == resource_id) {
            handle.clone()
        } else {
            panic!("Resource {} not found", resource_id);
        }
    }

    fn remove_all(&mut self) {
        self.resources.clear();
    }

    fn flush(&mut self) {
        let mut to_remove = Vec::new();
        for handle in self.handles.iter_mut() {
            if Arc::strong_count(&handle) <= 1 {
                to_remove.push(handle.id());
            }
        }
        for id in to_remove {
            self.remove(id);
        }
    }
    fn remove(&mut self, resource_id: ResourceId) {
        self.handles.retain(|h| h.id() != resource_id);
        self.resources.retain(|r| r.id() != resource_id);
    }
    fn has(&self, resource_id: ResourceId) -> bool {
        self.handles.iter().any(|h| h.id() == resource_id)
    }

    fn handles(&self) -> Vec<GenericRef> {
        let mut handles = Vec::new();
        for handle in self.handles.iter() {
            handles.push(handle.clone() as _);
        }
        handles
    }

    fn resources(&self) -> Vec<GenericResource> {
        let mut resources = Vec::new();
        for resource in self.resources.iter() {
            resources.push(resource.clone() as _);
        }
        resources
    }
    fn count(&self) -> usize {
        self.resources.len()
    }
}
