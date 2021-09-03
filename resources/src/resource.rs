use nrg_messenger::implement_message;
use nrg_serialize::Uid;
use std::{
    any::Any,
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::ResourceRef;

#[derive(Clone)]
pub enum ResourceEvent {
    Reload(PathBuf),
}
implement_message!(ResourceEvent);

pub type ResourceId = Uid;

pub trait ResourceData: Send + Sync + 'static {
    fn id(&self) -> ResourceId;

    #[inline]
    fn get_name(&self) -> String {
        self.id().to_simple().to_string()
    }
}

pub trait BaseResource: Send + Sync + Any {
    fn as_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
}

pub struct ResourceMutex<T>
where
    T: ResourceData,
{
    id: ResourceId,
    data: RwLock<T>,
}

impl<T> ResourceMutex<T>
where
    T: ResourceData,
{
    #[inline]
    pub fn new(data: T) -> Self {
        Self {
            id: data.id(),
            data: RwLock::new(data),
        }
    }

    #[inline]
    pub fn get(&self) -> RwLockReadGuard<'_, T> {
        self.data.read().unwrap()
    }

    #[inline]
    pub fn get_mut(&self) -> RwLockWriteGuard<'_, T> {
        self.data.write().unwrap()
    }
}

impl<T> BaseResource for ResourceMutex<T>
where
    T: ResourceData,
{
    #[inline]
    fn as_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}

impl<T> ResourceData for Resource<T>
where
    T: ResourceData,
{
    fn id(&self) -> ResourceId {
        self.id
    }
}

pub type Resource<T> = Arc<ResourceMutex<T>>;
pub type GenericResource = Arc<dyn BaseResource>;

pub trait TypedStorage {
    fn remove_all(&mut self);
    fn flush(&mut self);
    fn has(&self, resource_id: ResourceId) -> bool;
    fn remove(&mut self, resource_id: ResourceId);
    fn count(&self) -> usize;
}

pub struct Storage<T>
where
    T: ResourceData,
{
    handles: Vec<ResourceRef<T>>,
    resources: HashMap<ResourceId, Resource<T>>,
}

impl<T> Default for Storage<T>
where
    T: ResourceData,
{
    fn default() -> Self {
        Self {
            handles: Vec::new(),
            resources: HashMap::new(),
        }
    }
}

impl<T> TypedStorage for Storage<T>
where
    T: ResourceData + Sized + 'static,
{
    #[inline]
    fn remove_all(&mut self) {
        self.resources.clear();
    }

    #[inline]
    fn flush(&mut self) {
        let mut to_remove = Vec::new();
        self.handles.iter_mut().for_each(|handle| {
            if Arc::strong_count(handle) == 1 && Arc::weak_count(handle) == 0 {
                to_remove.push(handle.id());
            }
        });
        for id in to_remove {
            self.remove(id);
        }
    }
    #[inline]
    fn remove(&mut self, resource_id: ResourceId) {
        self.handles.retain(|handle| handle.id() != resource_id);
        self.resources.remove(&resource_id);
    }
    #[inline]
    fn has(&self, resource_id: ResourceId) -> bool {
        self.resources.contains_key(&resource_id)
    }
    #[inline]
    fn count(&self) -> usize {
        self.handles.len()
    }
}

impl<T> Storage<T>
where
    T: ResourceData + Sized + 'static,
{
    #[inline]
    pub fn add(&mut self, handle: ResourceRef<T>, resource: Resource<T>) {
        self.handles.push(handle);
        self.resources.insert(resource.id(), resource);
    }

    #[inline]
    pub fn resource(&self, resource_id: ResourceId) -> Resource<T> {
        if let Some(resource) = self.resources.get(&resource_id) {
            resource.clone()
        } else {
            panic!("Resource {} not found", resource_id.to_simple());
        }
    }
    #[inline]
    pub fn get(&self, resource_id: ResourceId) -> ResourceRef<T> {
        if let Some(handle) = self
            .handles
            .iter()
            .find(|handle| handle.id() == resource_id)
        {
            handle.clone()
        } else {
            panic!("Resource {} not found", resource_id.to_simple());
        }
    }
    #[inline]
    pub fn handles(&self) -> &Vec<ResourceRef<T>> {
        &self.handles
    }
    #[inline]
    pub fn resources(&self) -> &HashMap<ResourceId, Resource<T>> {
        &self.resources
    }
    #[inline]
    pub fn match_resource<F>(&self, f: F) -> Option<ResourceRef<T>>
    where
        F: Fn(&T) -> bool,
    {
        for (id, r) in self.resources.iter() {
            if f(&r.as_ref().get()) {
                return Some(self.get(*id));
            }
        }
        None
    }
}
