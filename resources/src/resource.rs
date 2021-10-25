use nrg_serialize::Uid;
use std::{
    any::Any,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

pub type ResourceId = Uid;

pub trait ResourceTrait: Send + Sync + 'static {}

pub trait GenericResourceTrait: Send + Sync + Any {
    fn as_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
}

#[derive(Clone)]
pub struct ResourceHandle<T>
where
    T: ResourceTrait,
{
    id: ResourceId,
    storage: ResourceStorage<T>,
}

impl<T> ResourceHandle<T>
where
    T: ResourceTrait,
{
    #[inline]
    pub fn new(id: ResourceId, storage: ResourceStorage<T>) -> Self {
        Self { id, storage }
    }

    #[inline]
    pub fn id(&self) -> &ResourceId {
        &self.id
    }

    #[inline]
    pub fn get<F, R>(&self, mut f: F) -> R
    where
        F: FnMut(&T) -> R,
    {
        let storage = self.storage.read().unwrap();
        let resource = storage.get(self.id());
        f(&resource)
    }

    #[inline]
    pub fn get_mut<F, R>(&self, mut f: F) -> R
    where
        F: FnMut(&mut T) -> R,
    {
        let storage = self.storage.read().unwrap();
        let mut resource = storage.get_mut(self.id());
        f(&mut resource)
    }

    #[inline]
    pub fn move_before(&self, other_id: &ResourceId) {
        let mut storage = self.storage.write().unwrap();
        storage.move_before_other(self.id(), other_id)
    }

    #[inline]
    pub fn move_after(&self, other_id: &ResourceId) {
        let mut storage = self.storage.write().unwrap();
        storage.move_after_other(self.id(), other_id)
    }
}

impl<T> GenericResourceTrait for ResourceHandle<T>
where
    T: ResourceTrait,
{
    #[inline]
    fn as_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}

pub type Resource<T> = Arc<ResourceHandle<T>>;
pub type GenericResource = Arc<dyn GenericResourceTrait>;
pub type Handle<T> = Option<Resource<T>>;

pub trait ResourceCastTo {
    fn of_type<T: ResourceTrait>(&self) -> Resource<T>;
}

impl ResourceCastTo for GenericResource {
    #[inline]
    fn of_type<T: ResourceTrait>(&self) -> Resource<T> {
        let any = Arc::into_raw(self.clone().as_any());
        Arc::downcast(unsafe { Arc::from_raw(any) }).unwrap()
    }
}

pub trait TypedStorage: Send + Sync + Any {
    fn remove_all(&mut self);
    fn flush(&mut self);
    fn has(&self, resource_id: &ResourceId) -> bool;
    fn remove(&mut self, resource_id: &ResourceId);
    fn count(&self) -> usize;
}
pub type ResourceStorageRw = Arc<RwLock<Box<dyn TypedStorage>>>;
pub type ResourceStorage<T> = Arc<RwLock<Box<Storage<T>>>>;

pub trait StorageCastTo {
    fn of_type<T: ResourceTrait>(&self) -> ResourceStorage<T>;
}

impl StorageCastTo for ResourceStorageRw {
    #[inline]
    fn of_type<T: ResourceTrait>(&self) -> ResourceStorage<T> {
        let lock = Arc::into_raw(self.clone());
        let ptr = lock as *const RwLock<Box<Storage<T>>>;
        Arc::downcast(unsafe { Arc::from_raw(ptr) }).unwrap()
    }
}

struct ResourceData<T>
where
    T: ResourceTrait,
{
    data: RwLock<T>,
    handle: Resource<T>,
}

pub struct Storage<T>
where
    T: ResourceTrait,
{
    resources: Vec<ResourceData<T>>,
}

impl<T> Default for Storage<T>
where
    T: ResourceTrait,
{
    fn default() -> Self {
        Self {
            resources: Vec::new(),
        }
    }
}

impl<T> TypedStorage for Storage<T>
where
    T: ResourceTrait + Sized + 'static,
{
    #[inline]
    fn remove_all(&mut self) {
        self.resources.clear();
    }

    #[inline]
    fn flush(&mut self) {
        let mut to_remove = Vec::new();
        self.resources.iter_mut().for_each(|data| {
            if Arc::strong_count(&data.handle) == 1 && Arc::weak_count(&data.handle) == 0 {
                to_remove.push(*data.handle.id());
            }
        });
        to_remove.iter().for_each(|id| {
            self.remove(id);
        });
    }
    #[inline]
    fn remove(&mut self, resource_id: &ResourceId) {
        self.resources.retain(|r| r.handle.id() != resource_id);
    }
    #[inline]
    fn has(&self, resource_id: &ResourceId) -> bool {
        self.resources.iter().any(|r| r.handle.id() == resource_id)
    }
    #[inline]
    fn count(&self) -> usize {
        self.resources.len()
    }
}

impl<T> Storage<T>
where
    T: ResourceTrait + Sized + 'static,
{
    #[inline]
    pub fn get(&self, id: &ResourceId) -> RwLockReadGuard<'_, T> {
        self.resources
            .iter()
            .find(|r| r.handle.id() == id)
            .unwrap()
            .data
            .read()
            .unwrap()
    }
    #[inline]
    pub fn get_mut(&self, id: &ResourceId) -> RwLockWriteGuard<'_, T> {
        self.resources
            .iter()
            .find(|r| r.handle.id() == id)
            .unwrap()
            .data
            .write()
            .unwrap()
    }
    #[inline]
    pub fn resource(&self, id: &ResourceId) -> Handle<T> {
        if let Some(r) = self.resources.iter().find(|r| r.handle.id() == id) {
            return Some(r.handle.clone());
        }
        None
    }
    #[inline]
    pub fn add(&mut self, id: ResourceId, data: T, storage: ResourceStorage<T>) {
        if let Some(resource) = self.resources.iter().find(|r| r.handle.id() == &id) {
            let mut resource_data = resource.data.write().unwrap();
            *resource_data = data;
        } else {
            self.resources.push(ResourceData {
                data: RwLock::new(data),
                handle: Arc::new(ResourceHandle::new(id, storage)),
            });
        }
    }
    #[inline]
    pub fn match_resource<F>(&self, f: F) -> Handle<T>
    where
        F: Fn(&T) -> bool,
    {
        for r in self.resources.iter() {
            if f(&r.data.read().unwrap()) {
                return Some(r.handle.clone());
            }
        }
        None
    }
    #[inline]
    pub fn for_each_resource<F>(&self, mut f: F)
    where
        F: FnMut(&Resource<T>, &T),
    {
        self.resources
            .iter()
            .for_each(|r| f(&r.handle, &r.data.read().unwrap()));
    }
    #[inline]
    pub fn for_each_resource_mut<F>(&self, mut f: F)
    where
        F: FnMut(&Resource<T>, &mut T),
    {
        self.resources
            .iter()
            .for_each(|r| f(&r.handle, &mut r.data.write().unwrap()));
    }
    #[inline]
    pub fn get_index_of(&self, resource_id: &ResourceId) -> Option<usize> {
        self.resources
            .iter()
            .position(|r| r.handle.id() == resource_id)
    }
    #[inline]
    pub fn resource_at_index(&self, index: u32) -> Handle<T> {
        if let Some(r) = self.resources.iter().nth(index as _) {
            return Some(r.handle.clone());
        }
        None
    }
    pub fn move_before_other(&mut self, resource_id: &ResourceId, other_id: &ResourceId) {
        if let Some(index) = self.get_index_of(resource_id) {
            if let Some(other_index) = self.get_index_of(other_id) {
                if index > other_index {
                    let r = self.resources.remove(index);
                    self.resources.insert(other_index, r);
                }
            }
        }
    }
    pub fn move_after_other(&mut self, resource_id: &ResourceId, other_id: &ResourceId) {
        if let Some(index) = self.get_index_of(resource_id) {
            if let Some(other_index) = self.get_index_of(other_id) {
                if index < other_index {
                    let r = self.resources.remove(index);
                    self.resources.insert(other_index, r);
                }
            }
        }
    }
}
