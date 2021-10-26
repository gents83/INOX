use std::{
    any::type_name,
    collections::HashMap,
    sync::{Arc, RwLock},
};

use nrg_messenger::{Message, MessengerRw};
use nrg_serialize::{generate_uid_from_string, Uid};

use crate::{
    Data, Handle, Resource, ResourceEventHandler, ResourceId, ResourceStorageRw, ResourceTrait,
    SerializableResource, Storage, StorageCastTo, TypedResourceEventHandler,
};

#[derive(Default)]
pub struct SharedData {
    storage: RwLock<HashMap<Uid, ResourceStorageRw>>,
    event_handlers: RwLock<HashMap<Uid, Box<dyn ResourceEventHandler>>>,
}
unsafe impl Send for SharedData {}
unsafe impl Sync for SharedData {}

impl Data for SharedData {}

impl SharedData {
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
        T: ResourceTrait,
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
            storage
                .write()
                .unwrap()
                .add(resource_id, data, storage.clone());
            return storage.read().unwrap().resource(&resource_id).unwrap();
        }
        panic!("Unable to find storage for type {:?}", type_name::<T>());
    }
    #[inline]
    pub fn get_resource<T: ResourceTrait>(&self, resource_id: &ResourceId) -> Handle<T> {
        let typeid = generate_uid_from_string(type_name::<T>());
        if let Some(rs) = self.storage.read().unwrap().get(&typeid) {
            let storage = rs.of_type::<T>();
            return storage.read().unwrap().resource(resource_id);
        }
        None
    }
    #[inline]
    pub fn get_resource_at_index<T: ResourceTrait>(&self, index: u32) -> Handle<T> {
        let typeid = generate_uid_from_string(type_name::<T>());
        if let Some(rs) = self.storage.read().unwrap().get(&typeid) {
            let storage = rs.of_type::<T>();
            return storage.read().unwrap().resource_at_index(index);
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
            return storage.read().unwrap().get_index_of(resource_id);
        }
        None
    }
    #[inline]
    fn clear(&mut self) {
        for (&_t, rs) in self.storage.read().unwrap().iter() {
            rs.write().unwrap().remove_all();
        }
        self.storage.write().unwrap().clear();
    }
    #[inline]
    pub fn flush_resources(&self) {
        for (_type_id, rs) in self.storage.read().unwrap().iter() {
            rs.write().unwrap().flush();
        }
    }
    #[inline]
    pub fn is_message_handled(&self, msg: &dyn Message) -> Option<Uid> {
        for (type_id, handler) in self.event_handlers.read().unwrap().iter() {
            if handler.is_handled(msg) {
                return Some(type_id.clone());
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
            return rs.read().unwrap().has(resource_id);
        }
        false
    }
    #[inline]
    pub fn has_resources_of_type<T: 'static>(&self) -> bool {
        let typeid = generate_uid_from_string(type_name::<T>());
        if let Some(rs) = self.storage.read().unwrap().get(&typeid) {
            return rs.read().unwrap().count() > 0;
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
            return storage.read().unwrap().for_each_resource(f);
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
            return storage.write().unwrap().for_each_resource_mut(f);
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
            let s = storage.write().unwrap();
            return s.match_resource(f);
        }
        None
    }
    #[inline]
    pub fn get_num_resources_of_type<T: ResourceTrait>(&self) -> usize {
        let typeid = generate_uid_from_string(type_name::<T>());
        if let Some(rs) = self.storage.read().unwrap().get(&typeid) {
            return rs.read().unwrap().count();
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
