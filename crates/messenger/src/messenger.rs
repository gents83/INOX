use std::{
    any::TypeId,
    collections::HashMap,
    marker::PhantomData,
    sync::{Arc, RwLock},
};

use crate::{Listener, Message, MessageBox, MessageChannel, MessageFromString};

pub type MessengerRw = Arc<RwLock<Messenger>>;

trait MsgType: Send + Sync {
    fn type_id(&self) -> TypeId;
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
    fn from_string(&self, s: String) -> Option<Box<dyn Message>>;
}

#[derive(Default)]
struct MessageType<T>
where
    T: MessageFromString,
{
    msg_type: PhantomData<T>,
}
impl<T> MsgType for MessageType<T>
where
    T: MessageFromString,
{
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
    fn from_string(&self, s: String) -> Option<Box<dyn Message>> {
        T::from_string(s)
    }
}
unsafe impl<T> Send for MessageType<T> where T: MessageFromString {}
unsafe impl<T> Sync for MessageType<T> where T: MessageFromString {}

#[derive(Default)]
pub struct Messenger {
    message_channel: MessageChannel,
    messageboxes: HashMap<TypeId, Vec<MessageBox>>,
    registered_types: HashMap<TypeId, Box<dyn MsgType>>,
}

unsafe impl Send for Messenger {}
unsafe impl Sync for Messenger {}

impl Messenger {
    #[inline]
    pub fn get_dispatcher(&self) -> MessageBox {
        self.message_channel.get_messagebox()
    }

    #[inline]
    pub fn register_type<T>(&mut self)
    where
        T: MessageFromString + 'static,
    {
        let typeid = TypeId::of::<T>();
        self.registered_types.entry(typeid).or_insert_with(|| {
            Box::new(MessageType::<T> {
                msg_type: PhantomData::<T>::default(),
            })
        });
    }

    #[inline]
    pub fn register_messagebox<T>(&mut self, messagebox: MessageBox) -> &mut Self
    where
        T: MessageFromString + 'static,
    {
        self.register_type::<T>();
        let typeid = TypeId::of::<T>();
        let messageboxes = self.messageboxes.entry(typeid).or_insert_with(Vec::new);
        let index = messageboxes
            .iter()
            .position(|e| std::ptr::eq(e.as_ref(), messagebox.as_ref()));
        if index.is_none() {
            messageboxes.push(messagebox);
        }
        self
    }
    #[inline]
    pub fn unregister_messagebox<T>(&mut self, messagebox: MessageBox) -> &mut Self
    where
        T: Message + 'static,
    {
        let typeid = TypeId::of::<T>();
        self.unregister_messagebox_for_typeid(typeid, messagebox);
        self
    }

    #[inline]
    fn unregister_messagebox_for_typeid(&mut self, typeid: TypeId, messagebox: MessageBox) {
        let messageboxes = self.messageboxes.entry(typeid).or_insert_with(Vec::new);
        messageboxes.retain(|e| !std::ptr::eq(e.as_ref(), messagebox.as_ref()));
    }

    pub fn process_messages<F>(&self, mut f: F)
    where
        F: FnMut(&dyn Message),
    {
        read_messages(self.message_channel.get_listener(), |msg: &dyn Message| {
            f(msg);
            if let Some(messageboxes) = self.messageboxes.get(&msg.type_id()) {
                for messagebox in messageboxes.iter() {
                    let _ = messagebox.write().unwrap().send(msg.as_boxed());
                }
            }
        });
    }

    pub fn send_from_string(&self, s: String) {
        let dispatcher = self.get_dispatcher();
        self.registered_types.iter().for_each(|(_, t)| {
            if let Some(boxed_msg) = t.from_string(s.clone()) {
                dispatcher.write().unwrap().send(boxed_msg).ok();
            }
        });
    }
}

#[inline]
pub fn read_messages<F>(listener: Listener, mut f: F)
where
    F: FnMut(&dyn Message),
{
    while let Ok(msg) = listener.read().unwrap().try_recv() {
        f(msg.as_ref());
    }
}

pub trait GlobalMessenger {
    fn send_boxed(&self, msg: Box<dyn Message>);
    fn send_event<Event: Message>(&self, event: Event);
    fn send_event_from_string(&self, s: String);
}

impl GlobalMessenger for MessengerRw {
    fn send_boxed(&self, msg: Box<dyn Message>) {
        self.read()
            .unwrap()
            .get_dispatcher()
            .write()
            .unwrap()
            .send(msg)
            .ok();
    }

    fn send_event<Event: Message>(&self, event: Event) {
        self.send_boxed(event.as_boxed());
    }

    fn send_event_from_string(&self, s: String) {
        self.read().unwrap().send_from_string(s);
    }
}
