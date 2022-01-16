use std::{
    any::{type_name, TypeId},
    collections::HashMap,
    sync::{Arc, RwLock},
};

use sabi_messenger::{Message, MessengerRw};
use sabi_serialize::{generate_uid_from_string, Uid};

use crate::{
    Handle, Resource, ResourceEventHandler, ResourceId, ResourceStorageRw, ResourceTrait,
    SerializableResource, Singleton, Storage, StorageCastTo, TypedResourceEventHandler,
};

#[derive(Default)]
pub struct SharedData {
    singletons: RwLock<Vec<RwLock<Box<dyn Singleton>>>>,
    storage: RwLock<HashMap<Uid, ResourceStorageRw>>,
    event_handlers: RwLock<HashMap<Uid, Box<dyn ResourceEventHandler>>>,
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
    pub fn register_type<T>(&self)
    where
        T: ResourceTrait,
    {
        let typeid = generate_uid_from_string(type_name::<T>());
        debug_assert!(
            self.storage.read().unwrap().get(&typeid).is_none(),
            "Type {} has been already registered",
            type_name::<T>()
        );
        //debug_log("Registering resource type: {:?}", type_name::<T>(),);
        self.storage.write().unwrap().insert(
            typeid,
            Arc::new(RwLock::new(Box::new(Storage::<T>::default()))),
        );
    }
    #[inline]
    pub fn register_type_serializable<T>(&self)
    where
        T: SerializableResource,
    {
        self.register_type::<T>();
        let typeid = generate_uid_from_string(type_name::<T>());
        debug_assert!(
            self.event_handlers.read().unwrap().get(&typeid).is_none(),
            "Type {} has been already registered",
            type_name::<T>()
        );
        //debug_log("Registering resource type: {:?}", type_name::<T>(),);
        self.event_handlers
            .write()
            .unwrap()
            .insert(typeid, Box::new(TypedResourceEventHandler::<T>::default()));
    }
    #[inline]
    pub fn unregister_type<T>(&self)
    where
        T: ResourceTrait,
    {
        let typeid = generate_uid_from_string(type_name::<T>());
        debug_assert!(
            self.storage.read().unwrap().get(&typeid).is_some(),
            "Type {} has never been registered",
            type_name::<T>()
        );
        //debug_log("Unregistering resource type: {:?}", type_name::<T>());
        if let Some(rs) = self.storage.write().unwrap().remove(&typeid) {
            rs.write().unwrap().remove_all();
        }
    }
    #[inline]
    pub fn unregister_type_serializable<T>(&self)
    where
        T: SerializableResource,
    {
        self.unregister_type::<T>();
        let typeid = generate_uid_from_string(type_name::<T>());
        debug_assert!(
            self.event_handlers.read().unwrap().get(&typeid).is_some(),
            "Type {} has never been registered",
            type_name::<T>()
        );
        //debug_log("Unregistering resource type: {:?}", type_name::<T>());
        self.event_handlers.write().unwrap().remove(&typeid);
    }
    #[inline]
    pub fn add_resource<T: ResourceTrait>(&self, resource_id: ResourceId, data: T) -> Resource<T> {
        let typeid = generate_uid_from_string(type_name::<T>());
        if let Some(rs) = self.storage.read().unwrap().get(&typeid) {
            let storage = rs.of_type::<T>();
            if let Ok(mut rs) = storage.write() {
                return rs.add(resource_id, data);
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
    pub fn get_resource<T: ResourceTrait>(&self, resource_id: &ResourceId) -> Handle<T> {
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
    pub fn get_resource_at_index<T: ResourceTrait>(&self, index: u32) -> Handle<T> {
        let typeid = generate_uid_from_string(type_name::<T>());
        if let Some(rs) = self.storage.read().unwrap().get(&typeid) {
            let storage = rs.of_type::<T>();
            if let Ok(storage) = storage.read() {
                return storage.resource_at_index(index);
            } else {
                panic!(
                    "Unable to write to storage {} in get_resource_at_index()",
                    type_name::<T>()
                );
            };
        }
        None
    }
    #[inline]
    pub fn get_index_of_resource<T: ResourceTrait>(
        &self,
        resource_id: &ResourceId,
    ) -> Option<usize> {
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
                panic!("Unable to write to storage {} in clear()", type_id);
            };
        }
        self.storage.write().unwrap().clear();
    }
    #[inline]
    pub fn flush_resources(&self) {
        for (type_id, rs) in self.storage.read().unwrap().iter() {
            if let Ok(mut rs) = rs.write() {
                rs.flush(self);
            } else {
                panic!(
                    "Unable to write to storage {} in flush_resources()",
                    type_id
                );
            };
        }
    }
    #[inline]
    pub fn is_message_handled(&self, msg: &dyn Message) -> Option<Uid> {
        for (type_id, handler) in self.event_handlers.read().unwrap().iter() {
            if handler.is_handled(msg) {
                return Some(*type_id);
            }
        }
        None
    }
    #[inline]
    pub fn handle_events(
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
        type_id: Uid,
        msg: &dyn Message,
    ) -> bool {
        if let Some(event_handler) = shared_data.event_handlers.read().unwrap().get(&type_id) {
            return event_handler.handle_event(shared_data, global_messenger, msg);
        }
        false
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
        T: ResourceTrait,
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
        T: ResourceTrait,
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
        T: ResourceTrait,
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
    pub fn get_num_resources_of_type<T: ResourceTrait>(&self) -> usize {
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
