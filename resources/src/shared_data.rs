use std::{
    any::TypeId,
    collections::HashMap,
    sync::{Arc, RwLock},
};

use nrg_serialize::INVALID_UID;

use crate::{
    Data, HandleCastFrom, HandleCastTo, ResourceCastTo, ResourceData, ResourceHandle, ResourceId,
    ResourceRef, Storage, TypedStorage,
};

pub struct SharedData {
    storage: HashMap<TypeId, Box<dyn TypedStorage>>,
}
unsafe impl Send for SharedData {}
unsafe impl Sync for SharedData {}

impl Data for SharedData {}

impl Default for SharedData {
    #[inline]
    fn default() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }
}

impl SharedData {
    #[inline]
    fn register_type<T>(&mut self)
    where
        T: ResourceData,
    {
        self.storage
            .insert(TypeId::of::<T>(), Box::new(Storage::<T>::default()));
    }
    #[inline]
    pub fn get_storage<T>(&mut self) -> &mut Storage<T>
    where
        T: ResourceData,
    {
        if let Some(rs) = self.storage.get_mut(&TypeId::of::<T>()) {
            let r = rs.as_mut();
            let c = r as *mut dyn TypedStorage as *mut Storage<T>;
            unsafe { &mut *c }
        } else {
            self.register_type::<T>();
            self.get_storage::<T>()
        }
    }
    #[inline]
    pub fn add_resource<T: ResourceData>(shared_data: &SharedDataRw, data: T) -> ResourceRef<T> {
        let mut handle = Box::new(Arc::new(ResourceHandle::new(
            data.id(),
            shared_data.clone(),
        )));
        let mut shared_data = shared_data.write().unwrap();
        shared_data
            .get_storage::<T>()
            .add(handle.as_generic().clone(), Box::new(data));
        handle
    }
    #[inline]
    pub fn get_resource<T: ResourceData>(
        shared_data: &SharedDataRw,
        resource_id: ResourceId,
    ) -> ResourceRef<T> {
        let mut shared_data = shared_data.write().unwrap();
        shared_data
            .get_storage::<T>()
            .get(resource_id)
            .from_generic()
    }
    #[inline]
    pub fn get_resources_of_type<T: ResourceData>(
        shared_data: &SharedDataRw,
    ) -> Vec<ResourceRef<T>> {
        let mut shared_data = shared_data.write().unwrap();
        shared_data
            .get_storage::<T>()
            .handles()
            .iter_mut()
            .map(|h| h.from_generic())
            .collect()
    }
    #[inline]
    fn clear(&mut self) {
        for (&_t, rs) in self.storage.iter_mut() {
            rs.remove_all();
        }
        self.storage.clear();
    }
    #[inline]
    pub fn flush_resources(&mut self) {
        for (_, rs) in self.storage.iter_mut() {
            rs.flush();
        }
    }
    #[inline]
    pub fn has_resource<T: 'static>(shared_data: &SharedDataRw, resource_id: ResourceId) -> bool {
        let shared_data = shared_data.read().unwrap();
        if let Some(rs) = shared_data.storage.get(&TypeId::of::<T>()) {
            return rs.has(resource_id);
        }
        false
    }
    #[inline]
    pub fn has_resources_of_type<T: 'static>(shared_data: &SharedDataRw) -> bool {
        let shared_data = shared_data.read().unwrap();
        shared_data.storage.contains_key(&TypeId::of::<T>())
    }
    #[inline]
    pub fn match_resource<T, F>(shared_data: &SharedDataRw, f: F) -> ResourceId
    where
        T: ResourceData,
        F: Fn(&T) -> bool,
    {
        let mut shared_data = shared_data.write().unwrap();
        if let Some(rs) = shared_data.storage.get_mut(&TypeId::of::<T>()) {
            let mut resources = rs.resources();
            for r in resources.iter_mut() {
                let resource = r.to::<T>();
                if f(resource.as_ref()) {
                    return resource.id();
                }
            }
        }
        INVALID_UID
    }
    #[inline]
    pub fn get_num_resources_of_type<T: ResourceData>(shared_data: &SharedDataRw) -> usize {
        let shared_data = shared_data.read().unwrap();
        let rs = shared_data.storage.get(&TypeId::of::<T>()).unwrap();
        rs.count()
    }
}

impl Drop for SharedData {
    #[inline]
    fn drop(&mut self) {
        self.clear();
    }
}

pub type SharedDataRw = Arc<RwLock<SharedData>>;
