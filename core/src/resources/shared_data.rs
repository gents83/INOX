use std::{
    any::*,
    collections::HashMap,
    sync::{Arc, RwLock},
};

use super::data::*;
use super::resource::*;

struct ResourceStorage {
    stored: Vec<Arc<dyn ResourceTrait>>,
}
unsafe impl Send for ResourceStorage {}
unsafe impl Sync for ResourceStorage {}

impl Default for ResourceStorage {
    fn default() -> Self {
        Self { stored: Vec::new() }
    }
}

impl ResourceStorage {
    pub fn add_resource<T: 'static>(&mut self, resource: Resource<T>) -> ResourceId {
        let id = resource.id();
        self.stored.push(Arc::new(resource));
        id
    }
    pub fn remove_resources(&mut self) {
        self.stored.clear();
    }

    pub fn get_resource<T: 'static>(&self, resource_id: ResourceId) -> ResourceRef<T> {
        let item = self
            .stored
            .iter()
            .find(|&x| {
                let item: Arc<Resource<T>> = unsafe { std::mem::transmute_copy(x) };
                let res = unsafe { &*Arc::into_raw(item) };
                res.id() == resource_id
            })
            .unwrap();
        let item: Arc<Resource<T>> = unsafe { std::mem::transmute_copy(item) };
        let res = Arc::into_raw(item);
        ResourceRef::new(unsafe { &*res })
    }
    pub fn get_unique_resource<T: 'static>(&self) -> ResourceRef<T> {
        debug_assert!(
            self.stored.len() == 1,
            "Trying to get unique resource but multiple resource of same type exists"
        );
        let item = self.stored.first().unwrap();
        let item: Arc<Resource<T>> = unsafe { std::mem::transmute_copy(item) };
        let res = Arc::into_raw(item);
        ResourceRef::new(unsafe { &*res })
    }
    pub fn get_unique_resource_mut<T: 'static>(&self) -> ResourceRefMut<T> {
        debug_assert!(
            self.stored.len() == 1,
            "Trying to get unique resource but multiple resource of same type exists"
        );
        let item = self.stored.first().unwrap();
        let item: Arc<Resource<T>> = unsafe { std::mem::transmute_copy(item) };
        let res = Arc::into_raw(item);
        ResourceRefMut::new(unsafe { &*res })
    }
}

pub struct SharedData {
    resources: HashMap<TypeId, ResourceStorage>,
    types_to_remove: Vec<TypeId>,
}
unsafe impl Send for SharedData {}
unsafe impl Sync for SharedData {}

impl Data for SharedData {}

impl Default for SharedData {
    fn default() -> Self {
        Self {
            resources: HashMap::new(),
            types_to_remove: Vec::new(),
        }
    }
}

impl SharedData {
    pub fn add_resource<T: 'static>(&mut self, data: T) -> ResourceId {
        let vec = self
            .resources
            .entry(TypeId::of::<T>())
            .or_insert_with(ResourceStorage::default);
        vec.add_resource(Resource::new(data))
    }
    pub fn get_resource<T: 'static>(&self, resource_id: ResourceId) -> ResourceRef<T> {
        let vec = self.resources.get(&TypeId::of::<T>()).unwrap();
        vec.get_resource(resource_id)
    }

    pub fn request_remove_resources_of_type<T: 'static>(&mut self) {
        self.types_to_remove.push(TypeId::of::<T>());
    }
    pub fn get_unique_resource<T: 'static>(&self) -> ResourceRef<T> {
        let vec = self.resources.get(&TypeId::of::<T>()).unwrap();
        vec.get_unique_resource()
    }
    pub fn get_unique_resource_mut<T: 'static>(&self) -> ResourceRefMut<T> {
        let vec = self.resources.get(&TypeId::of::<T>()).unwrap();
        vec.get_unique_resource_mut()
    }

    pub fn process_pending_requests(&mut self) {
        for typeid in self.types_to_remove.iter() {
            let vec = self.resources.get_mut(&typeid).unwrap();
            vec.remove_resources();
            self.resources.remove_entry(&typeid);
        }
        self.types_to_remove.clear();
    }
}

impl Drop for SharedData {
    fn drop(&mut self) {
        self.process_pending_requests();
        self.resources.clear();
    }
}

pub type SharedDataRw = Arc<RwLock<SharedData>>;
