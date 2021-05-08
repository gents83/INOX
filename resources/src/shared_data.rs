use std::{
    any::TypeId,
    collections::HashMap,
    sync::{Arc, RwLock},
};

use nrg_serialize::INVALID_UID;

use crate::{Data, Resource, ResourceId, ResourceRef, ResourceRefMut, ResourceTrait};

pub struct ResourceStorage {
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
    pub fn is_empty(&self) -> bool {
        self.stored.is_empty()
    }
    pub fn add_resource<T: 'static>(&mut self, resource: Resource<T>) -> ResourceId {
        let id = resource.id();
        self.stored.push(Arc::new(resource));
        id
    }
    pub fn remove_resource(&mut self, resource_id: ResourceId) {
        self.stored.retain(|resource| resource.id() != resource_id);
    }
    pub fn remove_resources(&mut self) {
        self.stored.clear();
    }
    pub fn has_resource(&self, resource_id: ResourceId) -> bool {
        if let Some(_index) = self
            .stored
            .iter()
            .position(|x| x.as_ref().id() == resource_id)
        {
            return true;
        }
        false
    }
    pub fn match_resource<T, F>(&self, f: F) -> ResourceId
    where
        T: 'static + Sized,
        F: Fn(&T) -> bool,
    {
        if let Some(item) = self.stored.iter().find(|&e| {
            let resource: &Arc<Resource<T>> =
                unsafe { &*(e as *const Arc<dyn ResourceTrait> as *const Arc<Resource<T>>) };
            f(&resource.borrow())
        }) {
            return item.id();
        }
        INVALID_UID
    }
    pub fn get_resource<T: 'static>(&self, resource_id: ResourceId) -> ResourceRef<T> {
        let item = self
            .stored
            .iter()
            .find(|&x| x.as_ref().id() == resource_id)
            .unwrap();
        let item: Arc<Resource<T>> = unsafe { std::mem::transmute_copy(item) };
        let res = Arc::into_raw(item);
        ResourceRef::new(unsafe { &*res })
    }
    pub fn get_resource_mut<T: 'static>(&self, resource_id: ResourceId) -> ResourceRefMut<T> {
        let item = self
            .stored
            .iter()
            .find(|&x| x.as_ref().id() == resource_id)
            .unwrap();
        let item: Arc<Resource<T>> = unsafe { std::mem::transmute_copy(item) };
        let res = Arc::into_raw(item);
        ResourceRefMut::new(unsafe { &*res })
    }
    pub fn get_resources_of_type<T: 'static>(&self) -> Vec<ResourceRef<T>> {
        let mut vec = Vec::new();
        self.stored.iter().for_each(|e| {
            let id = e.id();
            vec.push(self.get_resource(id));
        });
        vec
    }
    pub fn get_resources_of_type_mut<T: 'static>(&self) -> Vec<ResourceRefMut<T>> {
        let mut vec = Vec::new();
        self.stored.iter().for_each(|e| {
            let id = e.id();
            vec.push(self.get_resource_mut(id));
        });
        vec
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
}
unsafe impl Send for SharedData {}
unsafe impl Sync for SharedData {}

impl Data for SharedData {}

impl Default for SharedData {
    fn default() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }
}

impl SharedData {
    pub fn clear(&mut self) {
        for (&_t, rs) in self.resources.iter_mut() {
            //println!("Removing ResourceStorage with id {:?}", _t);
            rs.remove_resources();
        }
        self.resources.clear();
    }
    pub fn add_resource<T: 'static>(&mut self, data: T) -> ResourceId {
        /*if !self.resources.contains_key(&TypeId::of::<T>()) {
            println!(
                "Adding ResourceStorage {} with id {:?}",
                type_name::<T>(),
                TypeId::of::<T>()
            );
        }*/
        let vec = self
            .resources
            .entry(TypeId::of::<T>())
            .or_insert_with(ResourceStorage::default);
        vec.add_resource(Resource::new(data))
    }
    pub fn has_resource<T: 'static>(&self, resource_id: ResourceId) -> bool {
        if let Some(vec) = self.resources.get(&TypeId::of::<T>()) {
            return vec.has_resource(resource_id);
        }
        false
    }
    pub fn match_resource<T: 'static, F>(&self, f: F) -> ResourceId
    where
        T: 'static + Sized,
        F: Fn(&T) -> bool,
    {
        if let Some(vec) = self.resources.get(&TypeId::of::<T>()) {
            return vec.match_resource(f);
        }
        INVALID_UID
    }
    pub fn remove_resource<T: 'static>(&mut self, resource_id: ResourceId) {
        if let Some(vec) = self.resources.get_mut(&TypeId::of::<T>()) {
            vec.remove_resource(resource_id)
        }
    }
    pub fn remove_resources(&mut self, type_id: TypeId) {
        if let Some(vec) = self.resources.get_mut(&type_id) {
            vec.remove_resources();
            self.resources.remove_entry(&type_id);
        }
    }
    pub fn remove_resources_of_type<T: 'static>(&mut self) {
        self.remove_resources(TypeId::of::<T>());
    }
    pub fn get_resource<T: 'static>(&self, resource_id: ResourceId) -> ResourceRef<T> {
        let vec = self.resources.get(&TypeId::of::<T>()).unwrap();
        vec.get_resource(resource_id)
    }
    pub fn get_resource_mut<T: 'static>(&self, resource_id: ResourceId) -> ResourceRefMut<T> {
        let vec = self.resources.get(&TypeId::of::<T>()).unwrap();
        vec.get_resource_mut(resource_id)
    }
    pub fn get_resources_of_type<T: 'static>(&self) -> Vec<ResourceRef<T>> {
        let vec = self.resources.get(&TypeId::of::<T>()).unwrap();
        vec.get_resources_of_type()
    }
    pub fn get_resources_of_type_mut<T: 'static>(&self) -> Vec<ResourceRefMut<T>> {
        let vec = self.resources.get(&TypeId::of::<T>()).unwrap();
        vec.get_resources_of_type_mut()
    }
    pub fn get_unique_resource<T: 'static>(&self) -> ResourceRef<T> {
        let vec = self.resources.get(&TypeId::of::<T>()).unwrap();
        vec.get_unique_resource()
    }
    pub fn get_unique_resource_mut<T: 'static>(&self) -> ResourceRefMut<T> {
        let vec = self.resources.get(&TypeId::of::<T>()).unwrap();
        vec.get_unique_resource_mut()
    }
}

impl Drop for SharedData {
    fn drop(&mut self) {
        self.clear();
    }
}

pub type SharedDataRw = Arc<RwLock<SharedData>>;
