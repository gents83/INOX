use std::{
    any::TypeId,
    collections::HashMap,
    sync::{Arc, RwLock},
};

use nrg_serialize::INVALID_UID;

use crate::{Data, Resource, ResourceId, ResourceRc, ResourceTrait, ResourceTraitRc};

pub struct ResourceStorage {
    resources: Vec<ResourceTraitRc>,
}
unsafe impl Send for ResourceStorage {}
unsafe impl Sync for ResourceStorage {}

impl Default for ResourceStorage {
    #[inline]
    fn default() -> Self {
        Self {
            resources: Vec::new(),
        }
    }
}

impl ResourceStorage {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }
    #[inline]
    pub fn add_resource<T: 'static + ResourceTrait>(&mut self, data: T) -> ResourceId {
        let resource = Resource::new(data);
        let id = resource.id();
        self.resources.push(Arc::new(Box::new(resource)));
        id
    }
    #[inline]
    pub fn remove_resource(&mut self, resource_id: ResourceId) {
        self.resources
            .retain(|resource| resource.id() != resource_id);
    }
    #[inline]
    pub fn remove_resources(&mut self) {
        self.resources.clear();
    }
    #[inline]
    pub fn has_resource(&self, resource_id: ResourceId) -> bool {
        if let Some(_index) = self
            .resources
            .iter()
            .position(|x| x.as_ref().id() == resource_id)
        {
            return true;
        }
        false
    }
    #[inline]
    pub fn match_resource<T, F>(&self, f: F) -> ResourceId
    where
        T: 'static + ResourceTrait + Sized,
        F: Fn(&T) -> bool,
    {
        if let Some(item) = self.resources.iter().find(|&e| {
            let resource: &ResourceRc<T> =
                unsafe { &*(e as *const ResourceTraitRc as *const ResourceRc<T>) };
            f(&resource.get())
        }) {
            return item.id();
        }
        INVALID_UID
    }
    #[inline]
    pub fn get_resource<T: 'static + ResourceTrait>(
        &self,
        resource_id: ResourceId,
    ) -> ResourceRc<T> {
        let item = self
            .resources
            .iter()
            .find(|&x| x.id() == resource_id)
            .unwrap();
        let resource: &ResourceRc<T> =
            unsafe { &*(item as *const ResourceTraitRc as *const ResourceRc<T>) };
        resource.clone()
    }
    #[inline]
    pub fn get_resources_of_type<T: 'static + ResourceTrait>(&self) -> Vec<ResourceRc<T>> {
        let mut vec = Vec::new();
        self.resources.iter().for_each(|e| {
            let resource: &ResourceRc<T> =
                unsafe { &*(e as *const ResourceTraitRc as *const ResourceRc<T>) };
            vec.push(resource.clone());
        });
        vec
    }
}

pub struct SharedData {
    resources: HashMap<TypeId, ResourceStorage>,
}
unsafe impl Send for SharedData {}
unsafe impl Sync for SharedData {}

impl Data for SharedData {}

impl Default for SharedData {
    #[inline]
    fn default() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }
}

impl SharedData {
    #[inline]
    pub fn clear(&mut self) {
        for (&_t, rs) in self.resources.iter_mut() {
            rs.remove_resources();
        }
        self.resources.clear();
    }
    #[inline]
    pub fn add_resource<T: 'static + ResourceTrait>(&mut self, data: T) -> ResourceId {
        let rs = self
            .resources
            .entry(TypeId::of::<T>())
            .or_insert_with(ResourceStorage::default);
        rs.add_resource(data)
    }
    #[inline]
    pub fn remove_resource<T: 'static + ResourceTrait>(&mut self, resource_id: ResourceId) {
        if let Some(vec) = self.resources.get_mut(&TypeId::of::<T>()) {
            vec.remove_resource(resource_id)
        }
    }
    #[inline]
    pub fn remove_resources(&mut self, type_id: TypeId) {
        if let Some(vec) = self.resources.get_mut(&type_id) {
            vec.remove_resources();
            self.resources.remove_entry(&type_id);
        }
    }
    #[inline]
    pub fn remove_resources_of_type<T: 'static>(&mut self) {
        self.remove_resources(TypeId::of::<T>());
    }
    #[inline]
    pub fn has_resource<T: 'static>(shared_data: &SharedDataRw, resource_id: ResourceId) -> bool {
        let data = shared_data.read().unwrap();
        if let Some(rs) = data.resources.get(&TypeId::of::<T>()) {
            return rs.has_resource(resource_id);
        }
        false
    }
    #[inline]
    pub fn has_resources_of_type<T: 'static>(shared_data: &SharedDataRw) -> bool {
        let data = shared_data.read().unwrap();
        data.resources.contains_key(&TypeId::of::<T>())
    }
    #[inline]
    pub fn match_resource<T, F>(shared_data: &SharedDataRw, f: F) -> ResourceId
    where
        T: 'static + ResourceTrait + Sized,
        F: Fn(&T) -> bool,
    {
        let data = shared_data.read().unwrap();
        if let Some(rs) = data.resources.get(&TypeId::of::<T>()) {
            return rs.match_resource(f);
        }
        INVALID_UID
    }
    #[inline]
    pub fn get_resource<T: 'static + ResourceTrait>(
        shared_data: &SharedDataRw,
        resource_id: ResourceId,
    ) -> ResourceRc<T> {
        let data = shared_data.read().unwrap();
        let rs = data.resources.get(&TypeId::of::<T>()).unwrap();
        rs.get_resource(resource_id)
    }
    #[inline]
    pub fn get_resources_of_type<T: 'static + ResourceTrait>(
        shared_data: &SharedDataRw,
    ) -> Vec<ResourceRc<T>> {
        let data = shared_data.read().unwrap();
        let rs = data.resources.get(&TypeId::of::<T>()).unwrap();
        rs.get_resources_of_type()
    }
    #[inline]
    pub fn get_num_resources_of_type<T: 'static + ResourceTrait>(
        shared_data: &SharedDataRw,
    ) -> usize {
        let data = shared_data.read().unwrap();
        let rs = data.resources.get(&TypeId::of::<T>()).unwrap();
        rs.get_resources_of_type::<T>().len()
    }
    #[inline]
    pub fn get_resourceid_at_index<T: 'static + ResourceTrait>(
        shared_data: &SharedDataRw,
        index: usize,
    ) -> ResourceId {
        let data = shared_data.read().unwrap();
        let rs = data.resources.get(&TypeId::of::<T>()).unwrap();
        let vec = rs.get_resources_of_type::<T>();
        debug_assert!(index < vec.len());
        let resource_id = vec[index].get().id();
        resource_id
    }
}

impl Drop for SharedData {
    #[inline]
    fn drop(&mut self) {
        self.clear();
    }
}

pub type SharedDataRw = Arc<RwLock<SharedData>>;
