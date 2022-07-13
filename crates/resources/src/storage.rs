use std::{
    any::{type_name, Any},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use inox_messenger::MessageHubRc;

use crate::{
    swap_resource, Handle, Resource, ResourceEvent, ResourceHandle, ResourceId, ResourceTrait,
    SharedData,
};

pub trait TypedStorage: Send + Sync + Any {
    fn remove_all(&mut self);
    fn has(&self, resource_id: &ResourceId) -> bool;
    fn flush(&mut self, shared_data: &SharedData, message_hub: &MessageHubRc);
    fn remove(
        &mut self,
        resource_id: &ResourceId,
        shared_data: &SharedData,
        message_hub: &MessageHubRc,
    );
    fn count(&self) -> usize;
}
pub type ResourceStorageRw = Arc<RwLock<Box<dyn TypedStorage>>>;
pub type ResourceStorage<T> = Arc<RwLock<Box<Storage<T>>>>;

pub trait StorageCastTo {
    fn of_type<T>(&self) -> ResourceStorage<T>
    where
        T: ResourceTrait + 'static;
}

impl StorageCastTo for ResourceStorageRw {
    #[inline]
    fn of_type<T>(&self) -> ResourceStorage<T>
    where
        T: ResourceTrait + 'static,
    {
        let lock = Arc::into_raw(self.clone());
        let ptr = lock as *const RwLock<Box<Storage<T>>>;
        Arc::downcast(unsafe { Arc::from_raw(ptr) }).unwrap()
    }
}

pub struct Storage<T>
where
    T: ResourceTrait,
{
    resources: Vec<Resource<T>>,
    pending: Vec<Resource<T>>,
}

impl<T> Default for Storage<T>
where
    T: ResourceTrait,
{
    fn default() -> Self {
        Self {
            resources: Vec::new(),
            pending: Vec::new(),
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
    fn flush(&mut self, shared_data: &SharedData, message_hub: &MessageHubRc) {
        let mut num_pending = self.pending.len() as i32 - 1;
        while num_pending >= 0 {
            let pending = self.pending.remove(num_pending as usize);
            if let Some(resource) = self.resources.iter_mut().find(|r| r.id() == pending.id()) {
                swap_resource(resource, &pending);
                message_hub.send_event(ResourceEvent::<T>::Changed(*resource.id()));
            } else {
                panic!(
                    "Trying to swap a Resource with id {} not found in storage {}",
                    pending.id(),
                    type_name::<T>()
                );
            }
            num_pending -= 1;
        }
        let mut to_remove = Vec::new();
        self.resources.iter_mut().for_each(|data| {
            /*
            inox_log::debug_log!(
                "[{:?}] Strong: {} Weak: {}",
                data.id(),
                Arc::strong_count(data),
                Arc::weak_count(data)
            );
            */
            if Arc::strong_count(data) == 1 && Arc::weak_count(data) == 0 {
                to_remove.push(*data.id());
            }
        });

        to_remove.iter().for_each(|id| {
            self.remove(id, shared_data, message_hub);
        });
    }
    #[inline]
    fn remove(
        &mut self,
        resource_id: &ResourceId,
        shared_data: &SharedData,
        message_hub: &MessageHubRc,
    ) {
        if let Some(index) = self.resources.iter().position(|r| r.id() == resource_id) {
            let resource = self.resources.remove(index);
            message_hub.send_event(ResourceEvent::<T>::Destroyed(*resource_id));
            resource
                .get_mut()
                .on_destroy(shared_data, message_hub, resource_id);
            //debug_log!("Resource {} destroyed", resource_id);
        }
    }
    #[inline]
    fn has(&self, resource_id: &ResourceId) -> bool {
        self.resources.iter().any(|r| r.id() == resource_id)
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
        self.resources.iter().find(|r| r.id() == id).unwrap().get()
    }
    #[inline]
    pub fn get_mut(&self, id: &ResourceId) -> RwLockWriteGuard<'_, T> {
        self.resources
            .iter()
            .find(|r| r.id() == id)
            .unwrap()
            .get_mut()
    }
    #[inline]
    pub fn resource(&self, id: &ResourceId) -> Handle<T> {
        if let Some(r) = self.resources.iter().find(|r| r.id() == id) {
            return Some(r.clone());
        }
        None
    }
    #[inline]
    pub fn add(
        &mut self,
        message_hub: &MessageHubRc,
        resource_id: ResourceId,
        data: T,
    ) -> Resource<T> {
        let handle = Arc::new(ResourceHandle::new(resource_id, data));
        if self.resources.iter().any(|r| r.id() == &resource_id) {
            self.pending.push(handle.clone());
        } else {
            message_hub.send_event(ResourceEvent::<T>::Created(handle.clone()));
            self.resources.push(handle.clone());
        }
        handle
    }
    #[inline]
    pub fn match_resource<F>(&self, f: F) -> Handle<T>
    where
        F: Fn(&T) -> bool,
    {
        for r in self.resources.iter() {
            if f(&r.get()) {
                return Some(r.clone());
            }
        }
        None
    }
    #[inline]
    pub fn for_each_resource<F>(&self, mut f: F)
    where
        F: FnMut(&Resource<T>, &T),
    {
        self.resources.iter().for_each(|r| f(r, &r.get()));
    }
    #[inline]
    pub fn for_each_resource_mut<F>(&self, mut f: F)
    where
        F: FnMut(&Resource<T>, &mut T),
    {
        self.resources.iter().for_each(|r| f(r, &mut r.get_mut()));
    }
    #[inline]
    pub fn get_index_of(&self, resource_id: &ResourceId) -> Option<usize> {
        self.resources.iter().position(|r| r.id() == resource_id)
    }
    #[inline]
    pub fn resource_at_index(&self, index: u32) -> Handle<T> {
        if let Some(r) = self.resources.get(index as usize) {
            return Some(r.clone());
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
