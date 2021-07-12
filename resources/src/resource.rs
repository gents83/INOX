use nrg_messenger::implement_message;
use nrg_serialize::Uid;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use crate::{GenericRef, HandleCastFrom, HandleCastTo, ResourceRef};

#[derive(Clone)]
pub enum ResourceEvent {
    Reload(PathBuf),
}
implement_message!(ResourceEvent);

pub type ResourceId = Uid;

pub trait ResourceData: Send + Sync + 'static {
    fn id(&self) -> ResourceId;
}

pub struct Resource<T>
where
    T: ResourceData + ?Sized,
{
    data: RwLock<Box<T>>,
}
pub type GenericResource = Resource<dyn ResourceData>;

pub trait ResourceCastTo {
    fn to<T: ResourceData>(&mut self) -> &mut Resource<T>;
}
pub trait ResourceCastFrom {
    fn cast(&mut self) -> &mut GenericResource;
}

impl ResourceCastTo for GenericResource {
    fn to<T: ResourceData>(&mut self) -> &mut Resource<T> {
        let resource_data = self as *mut Resource<dyn ResourceData> as *mut Resource<T>;
        unsafe { &mut *resource_data }
    }
}

impl<T> ResourceCastFrom for Resource<T>
where
    T: ResourceData,
{
    fn cast(&mut self) -> &mut GenericResource {
        let resource_data = self as *mut Resource<T> as *mut Resource<dyn ResourceData>;
        unsafe { &mut *resource_data }
    }
}

impl<T: ResourceData> Resource<T> {
    pub fn new(data: Box<T>) -> Resource<T> {
        Self {
            data: RwLock::new(data),
        }
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

impl<T> AsRef<T> for Resource<T>
where
    T: ResourceData,
{
    fn as_ref(&self) -> &T {
        let resource = self.data.read().unwrap().as_ref() as *const T;
        unsafe { &*resource }
    }
}

impl<T> AsMut<T> for Resource<T>
where
    T: ResourceData,
{
    fn as_mut(&mut self) -> &mut T {
        let resource = self.data.write().unwrap().as_mut() as *mut T;
        unsafe { &mut *resource }
    }
}

pub trait TypedStorage {
    fn add(&mut self, handle: GenericRef, data: Box<dyn ResourceData>);
    fn resource(&mut self, resource_id: ResourceId) -> &mut GenericResource;
    fn get(&mut self, resource_id: ResourceId) -> GenericRef;
    fn remove_all(&mut self);
    fn flush(&mut self);
    fn remove(&mut self, resource_id: ResourceId);
    fn has(&self, resource_id: ResourceId) -> bool;
    fn handles(&mut self) -> Vec<GenericRef>;
    fn resources(&mut self) -> Vec<&mut GenericResource>;
    fn count(&self) -> usize;
}

pub struct ResourcePack<T>
where
    T: ResourceData,
{
    pub handle: ResourceRef<T>,
    pub resource: Resource<T>,
}
pub struct Storage<T>
where
    T: ResourceData,
{
    resources: HashMap<ResourceId, ResourcePack<T>>,
}

impl<T> Default for Storage<T>
where
    T: ResourceData,
{
    fn default() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }
}

impl<T> TypedStorage for Storage<T>
where
    T: ResourceData + Sized + 'static,
{
    fn add(&mut self, mut handle: GenericRef, mut data: Box<dyn ResourceData>) {
        let id = data.id();
        let resource_data = data.as_mut() as *mut dyn ResourceData as *mut T;
        let boxed = unsafe { Box::from_raw(resource_data) };
        let resource = Resource::new(boxed);
        let resource_pack = ResourcePack {
            handle: handle.from_generic(),
            resource,
        };
        self.resources.insert(id, resource_pack);
    }
    fn resource(&mut self, resource_id: ResourceId) -> &mut GenericResource
    where
        T: ResourceData,
    {
        if let Some(tuple) = self
            .resources
            .iter_mut()
            .find(|r| r.1.resource.id() == resource_id)
        {
            tuple.1.resource.cast()
        } else {
            panic!("Resource {} not found", resource_id.to_simple());
        }
    }
    fn get(&mut self, resource_id: ResourceId) -> GenericRef
    where
        T: ResourceData,
    {
        if let Some(tuple) = self
            .resources
            .iter_mut()
            .find(|r| r.1.resource.id() == resource_id)
        {
            tuple.1.handle.as_generic()
        } else {
            panic!("Resource {} not found", resource_id);
        }
    }

    fn remove_all(&mut self) {
        self.resources.clear();
    }

    fn flush(&mut self) {
        let mut to_remove = Vec::new();
        for tuple in self.resources.iter_mut() {
            if Arc::strong_count(tuple.1.handle.as_mut()) <= 1 {
                to_remove.push(*tuple.0);
            }
        }
        for id in to_remove {
            self.remove(id);
        }
    }
    fn remove(&mut self, resource_id: ResourceId) {
        self.resources.remove(&resource_id);
    }
    fn has(&self, resource_id: ResourceId) -> bool {
        self.resources.get(&resource_id).is_some()
    }

    fn handles(&mut self) -> Vec<GenericRef> {
        let mut handles = Vec::new();
        for tuple in self.resources.iter_mut() {
            handles.push(tuple.1.handle.clone());
        }
        handles.iter_mut().map(|h| h.as_generic()).collect()
    }

    fn resources(&mut self) -> Vec<&mut GenericResource> {
        let mut resources = Vec::new();
        for tuple in self.resources.iter_mut() {
            resources.push(tuple.1.resource.cast());
        }
        resources
    }
    fn count(&self) -> usize {
        self.resources.len()
    }
}
