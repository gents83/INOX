use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, RwLock},
};

use inox_uid::{generate_random_uid, Uid};

use crate::Message;

pub type MessageHubRc = Arc<MessageHub>;

pub type ListenerId = Uid;
type MessageId = Uid;

pub struct Listener {
    id: ListenerId,
    message_hub: MessageHubRc,
}

impl Listener {
    pub fn new(message_hub: &MessageHubRc) -> Self {
        Self {
            id: generate_random_uid(),
            message_hub: message_hub.clone(),
        }
    }
    pub fn message_hub(&self) -> &MessageHubRc {
        &self.message_hub
    }
    pub fn register<T>(&self) -> &Self
    where
        T: Message + 'static,
    {
        self.message_hub.register_listener::<T>(&self.id);
        self
    }
    pub fn unregister<T>(&self) -> &Self
    where
        T: Message + 'static,
    {
        self.message_hub.unregister_listener::<T>(&self.id);
        self
    }
    #[inline]
    pub fn process_messages<T, F>(&self, f: F) -> &Self
    where
        F: FnMut(&T),
        T: Message + 'static,
    {
        self.message_hub.process_messages(&self.id, f);
        self
    }
}

struct ListenerData {
    id: ListenerId,
    messages: RwLock<Vec<MessageId>>,
}
impl ListenerData {
    fn new(id: &ListenerId) -> Self {
        Self {
            id: *id,
            messages: RwLock::new(Vec::new()),
        }
    }
}

trait MsgType: Send + Sync + Any {
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
    fn add_listener(&self, listener_id: &ListenerId);
    fn remove_listener(&self, listener_id: &ListenerId);
    fn has_listeners(&self) -> bool;
    fn flush(&self);
    fn message_from_string(&self, s: &str);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub type MessageFromStrFn<T> = dyn Fn(&str) -> Option<T>;

#[derive(Default)]
pub struct MessageType<T>
where
    T: Message,
{
    msg_from_str: Option<Box<MessageFromStrFn<T>>>,
    new_messages: RwLock<Vec<T>>,
    messages: RwLock<HashMap<MessageId, T>>,
    listeners: RwLock<Vec<ListenerData>>,
}

impl<T> MessageType<T>
where
    T: Message,
{
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&str) -> Option<T> + 'static,
    {
        Self {
            msg_from_str: Some(Box::new(f)),
            new_messages: RwLock::new(Vec::new()),
            messages: RwLock::new(HashMap::new()),
            listeners: RwLock::new(Vec::new()),
        }
    }
}

impl<T> MsgType for MessageType<T>
where
    T: Message + 'static,
{
    fn add_listener(&self, listener_id: &ListenerId) {
        //inox_log::debug_log!("Adding listener for {}", type_name::<T>());
        self.listeners
            .write()
            .unwrap()
            .push(ListenerData::new(listener_id));
    }
    fn remove_listener(&self, listener_id: &ListenerId) {
        self.listeners
            .write()
            .unwrap()
            .retain(|l| l.id != *listener_id);
    }
    fn has_listeners(&self) -> bool {
        !self.listeners.read().unwrap().is_empty()
    }
    fn flush(&self) {
        //inox_log::debug_log!("Flushing messages for {}", type_name::<T>());
        //inox_log::debug_log!("From {}", self.messages.read().unwrap().len());
        self.messages.write().unwrap().retain(|msg_id, _| {
            self.listeners
                .read()
                .unwrap()
                .iter()
                .any(|l| l.messages.read().unwrap().contains(msg_id))
        });
        //inox_log::debug_log!("to {}", self.messages.read().unwrap().len());
        for msg in self.new_messages.write().unwrap().drain(..) {
            self.messages.write().unwrap().retain(|msg_id, other| {
                let discard = msg.compare_and_discard(other);
                if discard {
                    self.listeners
                        .read()
                        .unwrap()
                        .iter()
                        .for_each(|l| l.messages.write().unwrap().retain(|id| id != msg_id));
                }
                !discard
            });
            let msg_id = generate_random_uid();
            self.listeners
                .read()
                .unwrap()
                .iter()
                .for_each(|l| l.messages.write().unwrap().push(msg_id));
            self.messages.write().unwrap().insert(msg_id, msg);
        }
    }
    fn message_from_string(&self, s: &str) {
        if let Some(f) = &self.msg_from_str {
            if let Some(msg) = f(s) {
                //inox_log::debug_log!("Message from string {}", s);
                self.send_event(msg);
            }
        }
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl<T> MessageType<T>
where
    T: Message,
{
    pub fn send_event(&self, msg: T) {
        self.new_messages
            .write()
            .unwrap()
            .retain(|other| !msg.compare_and_discard(other));
        self.new_messages.write().unwrap().push(msg);
    }
    pub fn process_messages<F>(&self, listener_id: &ListenerId, mut f: F)
    where
        F: FnMut(&T),
    {
        if let Some(listener) = self
            .listeners
            .read()
            .unwrap()
            .iter()
            .find(|l| l.id == *listener_id)
        {
            if listener.messages.read().unwrap().is_empty() {
                return;
            }
            let mut messages = Vec::new();
            {
                let mut listener_messages = listener.messages.write().unwrap();
                messages.append(listener_messages.as_mut());
                listener_messages.clear();
            }
            messages.iter().for_each(|msg_id| {
                if let Some(msg) = self.messages.read().unwrap().get(msg_id) {
                    f(msg);
                }
            });
        }
    }
}
unsafe impl<T> Send for MessageType<T> where T: Message {}
unsafe impl<T> Sync for MessageType<T> where T: Message {}

#[derive(Default)]
pub struct MessageHub {
    registered_types: RwLock<HashMap<TypeId, Box<dyn MsgType>>>,
}

impl Drop for MessageHub {
    fn drop(&mut self) {
        self.registered_types
            .write()
            .unwrap()
            .retain(|_, t| t.has_listeners());
        self.registered_types
            .read()
            .unwrap()
            .iter()
            .for_each(|(_, t)| {
                println!("Message type {:?} is still registered ", t.name());
            });
        debug_assert!(
            self.registered_types.read().unwrap().is_empty(),
            "Some message types are still registered",
        );
    }
}

unsafe impl Send for MessageHub {}
unsafe impl Sync for MessageHub {}

impl MessageHub {
    #[inline]
    pub fn register_type<T>(&self) -> &Self
    where
        T: Message + 'static,
    {
        let typeid = TypeId::of::<T>();
        self.registered_types
            .write()
            .unwrap()
            .entry(typeid)
            .or_insert_with(|| Box::new(MessageType::<T>::new(|s| T::from_string(s))));
        self
    }
    #[inline]
    pub fn unregister_type<T>(&self) -> &Self
    where
        T: Message + 'static,
    {
        let typeid = TypeId::of::<T>();
        self.registered_types.write().unwrap().remove(&typeid);
        self
    }

    #[inline]
    pub fn register_listener<T>(&self, listener_id: &ListenerId) -> &Self
    where
        T: Message + 'static,
    {
        self.register_type::<T>();
        let typeid = TypeId::of::<T>();
        if let Some(entry) = self.registered_types.write().unwrap().get_mut(&typeid) {
            let msg_type = entry.as_any_mut().downcast_mut::<MessageType<T>>().unwrap();
            msg_type.add_listener(listener_id);
        }
        self
    }
    #[inline]
    pub fn unregister_listener<T>(&self, listener_id: &ListenerId) -> &Self
    where
        T: Message + 'static,
    {
        let typeid = TypeId::of::<T>();
        if let Some(entry) = self.registered_types.write().unwrap().get_mut(&typeid) {
            let msg_type = entry.as_any_mut().downcast_mut::<MessageType<T>>().unwrap();
            msg_type.remove_listener(listener_id);
        }
        self
    }

    pub fn process_messages<T, F>(&self, listener_id: &ListenerId, f: F)
    where
        F: FnMut(&T),
        T: Message + 'static,
    {
        let typeid = TypeId::of::<T>();
        if let Some(entry) = self.registered_types.read().unwrap().get(&typeid) {
            let msg_type = entry.as_any().downcast_ref::<MessageType<T>>().unwrap();
            msg_type.process_messages(listener_id, f);
        }
    }

    pub fn flush(&self) {
        self.registered_types
            .read()
            .unwrap()
            .iter()
            .for_each(|(_, msg_type)| {
                msg_type.flush();
            });
    }

    pub fn send_from_string(&self, s: String) {
        self.registered_types
            .read()
            .unwrap()
            .iter()
            .for_each(|(_, t)| {
                t.message_from_string(s.as_str());
            });
    }

    pub fn send_event<T>(&self, msg: T)
    where
        T: Message + 'static,
    {
        let typeid = TypeId::of::<T>();
        if let Some(entry) = self.registered_types.read().unwrap().get(&typeid) {
            let msg_type = entry.as_any().downcast_ref::<MessageType<T>>().unwrap();
            msg_type.send_event(msg);
        }
    }
}
