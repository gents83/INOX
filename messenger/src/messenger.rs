use std::{
    any::TypeId,
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{Listener, Message, MessageBox, MessageChannel};

pub type MessengerRw = Arc<RwLock<Messenger>>;
pub struct Messenger {
    message_channel: MessageChannel,
    messageboxes: HashMap<TypeId, Vec<MessageBox>>,
    registered_types: Vec<TypeId>,
}

unsafe impl Send for Messenger {}
unsafe impl Sync for Messenger {}

impl Default for Messenger {
    fn default() -> Self {
        Self {
            message_channel: MessageChannel::default(),
            messageboxes: HashMap::new(),
            registered_types: Vec::new(),
        }
    }
}

impl Messenger {
    pub fn get_dispatcher(&self) -> MessageBox {
        self.message_channel.get_messagebox()
    }

    pub fn register_type<T>(&mut self)
    where
        T: Message + 'static,
    {
        let typeid = TypeId::of::<T>();
        self.register_type_with_id(typeid);
    }
    fn register_type_with_id(&mut self, typeid: TypeId) {
        if !self.registered_types.contains(&typeid) {
            self.registered_types.push(typeid);
        }
    }

    pub fn register_messagebox<T>(&mut self, messagebox: MessageBox)
    where
        T: Message + 'static,
    {
        let typeid = TypeId::of::<T>();
        self.register_messagebox_for_typeid(typeid, messagebox);
    }

    fn register_messagebox_for_typeid(&mut self, typeid: TypeId, messagebox: MessageBox) {
        self.register_type_with_id(typeid);
        let messageboxes = self.messageboxes.entry(typeid).or_insert_with(Vec::new);
        messageboxes.push(messagebox);
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
}

pub fn read_messages<F>(listener: Listener, mut f: F)
where
    F: FnMut(&dyn Message),
{
    while let Ok(msg) = listener.read().unwrap().try_recv() {
        f(msg.as_ref());
    }
}
