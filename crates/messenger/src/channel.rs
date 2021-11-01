use crate::Message;
use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Arc, RwLock,
};

pub type MessageBox = Arc<RwLock<Sender<Box<dyn Message>>>>;
pub type Listener = Arc<RwLock<Receiver<Box<dyn Message>>>>;

pub struct MessageChannel {
    dispatcher: MessageBox,
    listener: Listener,
}

unsafe impl Send for MessageChannel {}
unsafe impl Sync for MessageChannel {}

impl Default for MessageChannel {
    #[inline]
    fn default() -> Self {
        let (dispatcher, listener) = MessageChannel::create_channel();
        Self {
            dispatcher,
            listener,
        }
    }
}

impl MessageChannel {
    #[inline]
    fn create_channel() -> (MessageBox, Listener) {
        let (sender, receiver) = channel();
        (
            Arc::new(RwLock::new(sender)),
            Arc::new(RwLock::new(receiver)),
        )
    }

    #[inline]
    pub fn get_listener(&self) -> Listener {
        self.listener.clone()
    }
    #[inline]
    pub fn get_messagebox(&self) -> MessageBox {
        self.dispatcher.clone()
    }
}
