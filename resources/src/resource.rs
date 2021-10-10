use nrg_messenger::implement_message;
use nrg_serialize::Uid;
use std::{
    any::Any,
    path::PathBuf,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

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
pub type Handle<T> = Option<Resource<T>>;

pub trait ResourceCastTo {
    fn of_type<T: ResourceData>(&self) -> Resource<T>;
}

impl ResourceCastTo for GenericResource {
    #[inline]
    fn of_type<T: ResourceData>(&self) -> Resource<T> {
        let any = Arc::into_raw(self.clone().as_any());
        Arc::downcast(unsafe { Arc::from_raw(any) }).unwrap()
    }
}

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
    resources: Vec<Resource<T>>,
}

impl<T> Default for Storage<T>
where
    T: ResourceData,
{
    fn default() -> Self {
        Self {
            resources: Vec::new(),
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
        self.resources.iter_mut().for_each(|resource| {
            if Arc::strong_count(resource) == 1 && Arc::weak_count(resource) == 0 {
                to_remove.push(resource.id());
            }
        });
        for id in to_remove {
            self.remove(id);
        }
    }
    #[inline]
    fn remove(&mut self, resource_id: ResourceId) {
        self.resources.retain(|r| r.id() != resource_id);
    }
    #[inline]
    fn has(&self, resource_id: ResourceId) -> bool {
        self.resources.iter().any(|r| r.id() == resource_id)
    }
    #[inline]
    fn count(&self) -> usize {
        self.resources.len()
    }
}

impl<T> Storage<T>
where
    T: ResourceData + Sized + 'static,
{
    #[inline]
    pub fn add(&mut self, resource: Resource<T>) {
        self.resources.push(resource);
    }

    #[inline]
    pub fn resource(&self, resource_id: ResourceId) -> &Resource<T> {
        if let Some(index) = self.resources.iter().position(|r| r.id() == resource_id) {
            &self.resources[index]
        } else {
            panic!("Resource {} not found", resource_id.to_simple());
        }
    }
    #[inline]
    pub fn get_at_index(&self, index: usize) -> &Resource<T> {
        &self.resources[index]
    }
    #[inline]
    pub fn get_index_of(&self, resource_id: ResourceId) -> Option<usize> {
        self.resources.iter().position(|h| h.id() == resource_id)
    }
    #[inline]
    pub fn resources(&self) -> &Vec<Resource<T>> {
        &self.resources
    }
    #[inline]
    pub fn match_resource<F>(&self, f: F) -> Option<&Resource<T>>
    where
        F: Fn(&T) -> bool,
    {
        for r in self.resources.iter() {
            if f(&r.as_ref().get()) {
                return Some(r);
            }
        }
        None
    }
    #[inline]
    pub fn for_each_resource<F>(&self, mut f: F)
    where
        F: FnMut(&Resource<T>),
    {
        self.resources.iter().for_each(|r| f(r));
    }
}
