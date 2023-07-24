use std::marker::PhantomData;

use inox_messenger::{Listener, MessageHubRc};

use crate::{
    DataTypeResource, DataTypeResourceEvent, ResourceEvent, ResourceTrait, SerializableResource,
    SerializableResourceEvent, SharedDataRc,
};

pub trait DeserializeFunction: FnOnce(&SharedDataRc, &MessageHubRc) + Send + Sync {}
impl<F> DeserializeFunction for F where F: FnOnce(&SharedDataRc, &MessageHubRc) + Send + Sync {}

pub trait LoadFunction: Fn(Box<dyn DeserializeFunction>) + Send + Sync {}
impl<F> LoadFunction for F where F: Fn(Box<dyn DeserializeFunction>) + Clone + Send + Sync {}

pub trait EventHandler {
    fn handle_events(&self, f: &dyn LoadFunction);
}

pub struct ResourceEventHandler<T>
where
    T: ResourceTrait,
{
    marker: PhantomData<T>,
}

impl<T> ResourceEventHandler<T>
where
    T: ResourceTrait,
{
    pub fn new(_message_hub: &MessageHubRc) -> Self {
        ResourceEventHandler {
            marker: PhantomData::<T>,
        }
    }
}

impl<T> EventHandler for ResourceEventHandler<T>
where
    T: ResourceTrait,
{
    fn handle_events(&self, _f: &dyn LoadFunction) {
        //...
    }
}

pub struct SerializableResourceEventHandler<T>
where
    T: SerializableResource + 'static,
    <T as DataTypeResource>::DataType: Send + Sync,
{
    marker: PhantomData<T>,
    listener: Listener,
}

impl<T> Drop for SerializableResourceEventHandler<T>
where
    T: SerializableResource,
    <T as DataTypeResource>::DataType: Send + Sync,
{
    fn drop(&mut self) {
        self.listener.unregister::<SerializableResourceEvent<T>>();
        self.listener
            .message_hub()
            .unregister_type::<SerializableResourceEvent<T>>();
        self.listener
            .message_hub()
            .unregister_type::<DataTypeResourceEvent<T>>();
        self.listener
            .message_hub()
            .unregister_type::<ResourceEvent<T>>();
    }
}

impl<T> SerializableResourceEventHandler<T>
where
    T: SerializableResource,
    <T as DataTypeResource>::DataType: Send + Sync,
{
    pub fn new(message_hub: &MessageHubRc) -> Self {
        let listener = Listener::new(message_hub);
        message_hub.register_type::<ResourceEvent<T>>();
        message_hub.register_type::<DataTypeResourceEvent<T>>();
        message_hub.register_type::<SerializableResourceEvent<T>>();
        listener.register::<SerializableResourceEvent<T>>();
        Self {
            marker: PhantomData::<T>,
            listener,
        }
    }
}

impl<T> EventHandler for SerializableResourceEventHandler<T>
where
    T: SerializableResource,
    <T as DataTypeResource>::DataType: Send + Sync,
{
    fn handle_events(&self, f: &dyn LoadFunction) {
        self.listener
            .process_messages(|msg: &SerializableResourceEvent<T>| {
                let SerializableResourceEvent::<T>::Load(path, on_create_data) = msg;
                //inox_log::debug_log!("Received load event for: {:?}", path);
                if <T as SerializableResource>::is_matching_extension(path.as_path()) {
                    //inox_log::debug_log!("Handling it!");
                    let p = path.clone();
                    let on_create_data = on_create_data.clone();
                    f(Box::new(move |shared_data, message_hub| {
                        T::create_from_file(shared_data, message_hub, p.as_path(), on_create_data);
                    }));
                }
            });
    }
}
