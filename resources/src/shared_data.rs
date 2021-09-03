use std::{
    any::type_name,
    collections::HashMap,
    sync::{Arc, RwLock},
};

use nrg_serialize::{generate_uid_from_string, Uid};

use crate::{
    Data, ResourceData, ResourceHandle, ResourceId, ResourceMutex, ResourceRef, Storage,
    TypedStorage,
};

pub struct SharedData {
    storage: HashMap<Uid, Box<dyn TypedStorage>>,
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
    pub fn register_type<T>(&mut self)
    where
        T: ResourceData,
    {
        let typeid = generate_uid_from_string(type_name::<T>());
        debug_assert!(
            self.storage.get(&typeid).is_none(),
            "Type {} has been already registered",
            type_name::<T>()
        );
        //println!("Registering resource type: {:?}", type_name::<T>(),);
        self.storage
            .insert(typeid, Box::new(Storage::<T>::default()));
    }
    #[inline]
    pub fn unregister_type<T>(&mut self)
    where
        T: ResourceData,
    {
        let typeid = generate_uid_from_string(type_name::<T>());
        debug_assert!(
            self.storage.get(&typeid).is_some(),
            "Type {} has never been registered",
            type_name::<T>()
        );
        //println!("Unegistering resource type: {:?}", type_name::<T>());
        if let Some(mut rs) = self.storage.remove(&typeid) {
            rs.as_mut().remove_all();
        }
    }
    #[inline]
    pub fn get_storage<T>(&self) -> &Storage<T>
    where
        T: ResourceData,
    {
        let typeid = generate_uid_from_string(type_name::<T>());
        if let Some(rs) = self.storage.get(&typeid) {
            let storage = rs.as_ref() as *const dyn TypedStorage as *const Storage<T>;
            unsafe { &*storage }
        } else {
            panic!("Type {} has not been registered", type_name::<T>());
        }
    }
    #[inline]
    pub fn get_storage_mut<T>(&mut self) -> &mut Storage<T>
    where
        T: ResourceData,
    {
        let typeid = generate_uid_from_string(type_name::<T>());
        if let Some(rs) = self.storage.get_mut(&typeid) {
            let storage = rs.as_mut() as *mut dyn TypedStorage as *mut Storage<T>;
            unsafe { &mut *storage }
        } else {
            panic!("Type {} has not been registered", type_name::<T>());
        }
    }
    #[inline]
    pub fn add_resource<T: ResourceData>(shared_data: &SharedDataRw, data: T) -> ResourceRef<T> {
        let handle = Arc::new(ResourceHandle::new(data.id(), shared_data.clone()));
        let mut shared_data = shared_data.write().unwrap();
        shared_data
            .get_storage_mut::<T>()
            .add(handle.clone(), Arc::new(ResourceMutex::new(data)));
        handle
    }
    #[inline]
    pub fn get_resource<T: ResourceData>(
        shared_data: &SharedDataRw,
        resource_id: ResourceId,
    ) -> ResourceRef<T> {
        let shared_data = shared_data.read().unwrap();
        shared_data.get_storage::<T>().get(resource_id)
    }
    #[inline]
    pub fn get_resources_of_type<T: ResourceData>(
        shared_data: &SharedDataRw,
    ) -> Vec<ResourceRef<T>> {
        if !Self::has_resources_of_type::<T>(shared_data) {
            return Vec::new();
        }
        let shared_data = shared_data.read().unwrap();
        shared_data.get_storage::<T>().handles().clone()
    }
    #[inline]
    fn clear(&mut self) {
        for (&_t, rs) in self.storage.iter_mut() {
            rs.remove_all();
        }
        self.storage.clear();
    }
    #[inline]
    pub fn flush_resources(&mut self, print_types: bool) {
        for (type_id, rs) in self.storage.iter_mut() {
            if print_types {
                println!("Flushing {}", type_id);
            }
            rs.flush();
        }
    }
    #[inline]
    pub fn has_resource<T: 'static>(shared_data: &SharedDataRw, resource_id: ResourceId) -> bool {
        let shared_data = shared_data.read().unwrap();
        let typeid = generate_uid_from_string(type_name::<T>());
        if let Some(rs) = shared_data.storage.get(&typeid) {
            return rs.has(resource_id);
        }
        false
    }
    #[inline]
    pub fn has_resources_of_type<T: 'static>(shared_data: &SharedDataRw) -> bool {
        let shared_data = shared_data.read().unwrap();
        let typeid = generate_uid_from_string(type_name::<T>());
        if shared_data.storage.contains_key(&typeid) {
            return shared_data.storage[&typeid].count() > 0;
        }
        false
    }
    #[inline]
    pub fn match_resource<T, F>(shared_data: &SharedDataRw, f: F) -> Option<ResourceRef<T>>
    where
        T: ResourceData,
        F: Fn(&T) -> bool,
    {
        let shared_data = shared_data.read().unwrap();
        shared_data.get_storage::<T>().match_resource(f)
    }
    #[inline]
    pub fn get_num_resources_of_type<T: ResourceData>(shared_data: &SharedDataRw) -> usize {
        if !Self::has_resources_of_type::<T>(shared_data) {
            return 0;
        }
        let shared_data = shared_data.read().unwrap();
        let typeid = generate_uid_from_string(type_name::<T>());
        let rs = shared_data.storage.get(&typeid).unwrap();
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
