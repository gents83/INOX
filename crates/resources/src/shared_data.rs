use std::{
    any::{type_name, TypeId},
    collections::HashMap,
    sync::{Arc, RwLock},
};

use inox_messenger::MessageHubRc;
use inox_uid::{generate_uid_from_string, Uid};

use crate::{
    DataTypeResource, EventHandler, Handle, LoadFunction, Resource, ResourceEvent,
    ResourceEventHandler, ResourceId, ResourceStorageRw, ResourceTrait, SerializableResource,
    SerializableResourceEvent, SerializableResourceEventHandler, Singleton, Storage, StorageCastTo,
};

#[derive(Default)]
pub struct SharedData {
    singletons: RwLock<Vec<RwLock<Box<dyn Singleton>>>>,
    storage: RwLock<HashMap<Uid, ResourceStorageRw>>,
    event_handlers: RwLock<HashMap<Uid, Box<dyn EventHandler>>>,
}
unsafe impl Send for SharedData {}
unsafe impl Sync for SharedData {}

impl SharedData {
    #[inline]
    pub fn register_singleton<T>(&self, singleton: T)
    where
        T: Singleton + 'static,
    {
        self.singletons
            .write()
            .unwrap()
            .push(RwLock::new(Box::new(singleton)));
    }
    #[inline]
    pub fn unregister_singleton<T>(&self)
    where
        T: Singleton + 'static,
    {
        self.singletons
            .write()
            .unwrap()
            .retain(|s| s.read().unwrap().as_ref().type_id() != TypeId::of::<T>());
    }
    #[inline]
    pub fn get_singleton<T>(&self) -> Option<&T>
    where
        T: Singleton,
    {
        if let Some(s) = self
            .singletons
            .read()
            .unwrap()
            .iter()
            .find(|s| s.read().unwrap().as_ref().type_id() == TypeId::of::<T>())
        {
            return Some(unsafe { &*(s.read().unwrap().as_ref() as *const _ as *const T) });
        }
        None
    }
    #[inline]
    #[allow(clippy::mut_from_ref)]
    pub fn get_singleton_mut<T>(&self) -> Option<&mut T>
    where
        T: Singleton,
    {
        if let Some(s) = self
            .singletons
            .read()
            .unwrap()
            .iter()
            .find(|s| s.read().unwrap().as_ref().type_id() == TypeId::of::<T>())
        {
            return Some(unsafe { &mut *(s.write().unwrap().as_mut() as *mut _ as *mut T) });
        }
        None
    }

    #[inline]
    pub fn register_type<T>(&self, message_hub: &MessageHubRc)
    where
        T: ResourceTrait + 'static + Clone,
    {
        let typeid = generate_uid_from_string(type_name::<T>());
        debug_assert!(
            self.storage.read().unwrap().get(&typeid).is_none(),
            "Type {} has been already registered",
            type_name::<T>()
        );
        message_hub.register_type::<ResourceEvent<T>>();
        //debug_log("Registering resource type: {:?}", type_name::<T>(),);
        self.storage
            .write()
            .unwrap()
            .insert(typeid, Arc::new(RwLock::new(Box::<Storage<T>>::default())));
        self.event_handlers.write().unwrap().insert(
            typeid,
            Box::new(ResourceEventHandler::<T>::new(message_hub)),
        );
    }
    #[inline]
    pub fn register_type_serializable<T>(&self, message_hub: &MessageHubRc)
    where
        T: SerializableResource + 'static,
        <T as DataTypeResource>::DataType: Send + Sync,
    {
        self.register_type::<T>(message_hub);
        let typeid = generate_uid_from_string(type_name::<T>());
        let mut event_handlers = self.event_handlers.write().unwrap();
        event_handlers.remove(&typeid);
        message_hub.register_type::<SerializableResourceEvent<T>>();
        //debug_log("Registering resource type: {:?}", type_name::<T>(),);
        event_handlers.insert(
            typeid,
            Box::new(SerializableResourceEventHandler::<T>::new(message_hub)),
        );
    }
    #[inline]
    pub fn unregister_type<T>(&self, message_hub: &MessageHubRc)
    where
        T: ResourceTrait + 'static,
    {
        let typeid = generate_uid_from_string(type_name::<T>());
        debug_assert!(
            self.storage.read().unwrap().get(&typeid).is_some(),
            "Type {} has never been registered",
            type_name::<T>()
        );
        message_hub.unregister_type::<ResourceEvent<T>>();
        //debug_log("Unregistering resource type: {:?}", type_name::<T>());
        if let Some(rs) = self.storage.write().unwrap().remove(&typeid) {
            rs.write().unwrap().remove_all();
        }
        self.event_handlers.write().unwrap().remove(&typeid);
    }
    #[inline]
    pub fn unregister_type_serializable<T>(&self, message_hub: &MessageHubRc)
    where
        T: SerializableResource + 'static,
    {
        self.unregister_type::<T>(message_hub);
        let typeid = generate_uid_from_string(type_name::<T>());
        message_hub.unregister_type::<SerializableResourceEvent<T>>();
        //debug_log("Unregistering resource type: {:?}", type_name::<T>());
        self.event_handlers.write().unwrap().remove(&typeid);
    }
    #[inline]
    pub fn add_resource<T>(
        &self,
        message_hub: &MessageHubRc,
        resource_id: ResourceId,
        data: T,
    ) -> Resource<T>
    where
        T: ResourceTrait + 'static,
    {
        let typeid = generate_uid_from_string(type_name::<T>());
        if let Some(rs) = self.storage.read().unwrap().get(&typeid) {
            let storage = rs.of_type::<T>();
            if let Ok(mut rs) = storage.write() {
                return rs.add(message_hub, resource_id, data);
            } else {
                panic!(
                    "Unable to write to storage {} in add_resource()",
                    type_name::<T>()
                );
            };
        }
        panic!("Unable to find storage for type {:?}", type_name::<T>());
    }
    #[inline]
    pub fn get_resource<T>(&self, resource_id: &ResourceId) -> Handle<T>
    where
        T: ResourceTrait + 'static,
    {
        let typeid = generate_uid_from_string(type_name::<T>());
        if let Some(rs) = self.storage.read().unwrap().get(&typeid) {
            let storage = rs.of_type::<T>();
            if let Ok(storage) = storage.read() {
                return storage.resource(resource_id);
            } else {
                panic!(
                    "Unable to write to storage {} in get_resource()",
                    type_name::<T>()
                );
            };
        }
        None
    }
    #[inline]
    pub fn get_index_of_resource<T>(&self, resource_id: &ResourceId) -> Option<usize>
    where
        T: ResourceTrait + 'static,
    {
        let typeid = generate_uid_from_string(type_name::<T>());
        if let Some(rs) = self.storage.read().unwrap().get(&typeid) {
            let storage = rs.of_type::<T>();
            if let Ok(storage) = storage.read() {
                return storage.get_index_of(resource_id);
            } else {
                panic!(
                    "Unable to write to storage {} in get_index_of_resource()",
                    type_name::<T>()
                );
            };
        }
        None
    }
    #[inline]
    fn clear(&mut self) {
        for (type_id, rs) in self.storage.read().unwrap().iter() {
            if let Ok(mut rs) = rs.write() {
                rs.remove_all();
            } else {
                panic!("Unable to write to storage {type_id} in clear()");
            };
        }
        self.storage.write().unwrap().clear();
    }
    #[inline]
    pub fn flush_resources(&self, message_hub: &MessageHubRc) {
        inox_profiler::scoped_profile!("shared_data::flush_resources");
        for (type_id, rs) in self.storage.read().unwrap().iter() {
            if let Ok(mut rs) = rs.write() {
                rs.flush(self, message_hub);
            } else {
                panic!("Unable to write to storage {type_id} in flush_resources()");
            };
        }
    }
    #[inline]
    pub fn handle_events(&self, f: impl LoadFunction) {
        inox_profiler::scoped_profile!("shared_data::flush_resources");
        self.event_handlers
            .write()
            .unwrap()
            .iter_mut()
            .for_each(|(_, handler)| {
                handler.handle_events(&f);
            });
    }
    #[inline]
    pub fn has<T: 'static>(&self, resource_id: &ResourceId) -> bool {
        let typeid = generate_uid_from_string(type_name::<T>());
        if let Some(rs) = self.storage.read().unwrap().get(&typeid) {
            if let Ok(storage) = rs.read() {
                return storage.has(resource_id);
            } else {
                panic!("Unable to write to storage {} in has()", type_name::<T>());
            };
        }
        false
    }
    #[inline]
    pub fn has_resources_of_type<T: 'static>(&self) -> bool {
        let typeid = generate_uid_from_string(type_name::<T>());
        if let Some(rs) = self.storage.read().unwrap().get(&typeid) {
            if let Ok(storage) = rs.read() {
                return storage.count() > 0;
            } else {
                panic!(
                    "Unable to read to storage {} in has_resources_of_type()",
                    type_name::<T>()
                );
            };
        }
        false
    }
    #[inline]
    pub fn for_each_resource<T, F>(&self, f: F)
    where
        T: ResourceTrait + 'static,
        F: FnMut(&Resource<T>, &T),
    {
        let typeid = generate_uid_from_string(type_name::<T>());
        if let Some(rs) = self.storage.read().unwrap().get(&typeid) {
            let storage = rs.of_type::<T>();
            if let Ok(storage) = storage.read() {
                storage.for_each_resource(f);
            } else {
                panic!(
                    "Unable to read to storage {} in for_each_resource()",
                    type_name::<T>()
                );
            };
        }
    }
    #[inline]
    pub fn for_each_resource_mut<T, F>(&self, f: F)
    where
        T: ResourceTrait + 'static,
        F: FnMut(&Resource<T>, &mut T),
    {
        let typeid = generate_uid_from_string(type_name::<T>());
        if let Some(rs) = self.storage.read().unwrap().get(&typeid) {
            let storage = rs.of_type::<T>();
            if let Ok(storage) = storage.write() {
                storage.for_each_resource_mut(f);
            } else {
                panic!(
                    "Unable to write to storage {} in for_each_resource_mut()",
                    type_name::<T>()
                );
            };
        }
    }
    #[inline]
    pub fn match_resource<T, F>(&self, f: F) -> Handle<T>
    where
        T: ResourceTrait + 'static,
        F: Fn(&T) -> bool,
    {
        let typeid = generate_uid_from_string(type_name::<T>());
        if let Some(rs) = self.storage.read().unwrap().get(&typeid) {
            let storage = rs.of_type::<T>();
            if let Ok(storage) = storage.write() {
                return storage.match_resource(f);
            } else {
                panic!(
                    "Unable to write to storage {} in match_resource()",
                    type_name::<T>()
                );
            };
        }
        None
    }
    #[inline]
    pub fn num_resources<T: ResourceTrait>(&self) -> usize {
        let typeid = generate_uid_from_string(type_name::<T>());
        if let Some(rs) = self.storage.read().unwrap().get(&typeid) {
            if let Ok(storage) = rs.read() {
                return storage.count();
            } else {
                panic!(
                    "Unable to read to storage {} in get_num_resources_of_type()",
                    type_name::<T>()
                );
            };
        }
        0
    }
}

impl Drop for SharedData {
    #[inline]
    fn drop(&mut self) {
        self.clear();
    }
}

pub type SharedDataRc = Arc<SharedData>;
