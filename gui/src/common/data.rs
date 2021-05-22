use nrg_messenger::{Listener, Message, MessageBox, MessageChannel, MessengerRw};
use nrg_resources::SharedDataRw;
use nrg_serialize::{typetag, Deserialize, Serialize};

use crate::{WidgetGraphics, WidgetNode, WidgetState};

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct WidgetData {
    node: WidgetNode,
    graphics: WidgetGraphics,
    state: WidgetState,
    initialized: bool,
    #[serde(skip)]
    message_channel: MessageChannel,
    #[serde(skip)]
    shared_data: SharedDataRw,
    #[serde(skip)]
    global_messenger: MessengerRw,
}

impl WidgetData {
    pub fn new(shared_data: SharedDataRw, global_messenger: MessengerRw) -> Self {
        Self {
            node: WidgetNode::default(),
            graphics: WidgetGraphics::new(&shared_data),
            state: WidgetState::default(),
            initialized: false,
            message_channel: MessageChannel::default(),
            shared_data,
            global_messenger,
        }
    }
    pub fn load_override(
        &mut self,
        shared_data: SharedDataRw,
        global_messenger: MessengerRw,
    ) -> &mut Self {
        self.graphics_mut().load_override(&shared_data);
        self.message_channel = MessageChannel::default();
        self.shared_data = shared_data;
        self.global_messenger = global_messenger;
        self
    }
    #[inline]
    pub fn register_to_listen_event<Msg>(&mut self)
    where
        Msg: Message,
    {
        self.global_messenger
            .write()
            .unwrap()
            .register_messagebox::<Msg>(self.message_channel.get_messagebox());
    }
    #[inline]
    pub fn unregister_to_listen_event<Msg>(&mut self)
    where
        Msg: Message,
    {
        self.global_messenger
            .write()
            .unwrap()
            .unregister_messagebox::<Msg>(self.message_channel.get_messagebox());
    }

    #[inline]
    pub fn get_shared_data(&self) -> &SharedDataRw {
        &self.shared_data
    }
    #[inline]
    pub fn get_global_messenger(&self) -> &MessengerRw {
        &self.global_messenger
    }
    #[inline]
    pub fn get_global_dispatcher(&self) -> nrg_messenger::MessageBox {
        self.global_messenger.read().unwrap().get_dispatcher()
    }
    #[inline]
    pub fn get_listener(&self) -> Listener {
        self.message_channel.get_listener()
    }
    #[inline]
    pub fn get_messagebox(&self) -> MessageBox {
        self.message_channel.get_messagebox()
    }
    #[inline]
    pub fn node(&self) -> &WidgetNode {
        &self.node
    }
    #[inline]
    pub fn node_mut(&mut self) -> &mut WidgetNode {
        &mut self.node
    }
    #[inline]
    pub fn graphics(&self) -> &WidgetGraphics {
        &self.graphics
    }
    #[inline]
    pub fn graphics_mut(&mut self) -> &mut WidgetGraphics {
        &mut self.graphics
    }
    #[inline]
    pub fn state(&self) -> &WidgetState {
        &self.state
    }
    #[inline]
    pub fn state_mut(&mut self) -> &mut WidgetState {
        &mut self.state
    }
    #[inline]
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    #[inline]
    pub fn mark_as_initialized(&mut self) {
        self.initialized = true;
    }
}

#[typetag::serde(tag = "data")]
pub trait WidgetDataGetter {
    fn load_override(&mut self, shared_data: SharedDataRw, global_messenger: MessengerRw);
    fn get_shared_data(&self) -> &SharedDataRw;
    fn get_global_messenger(&self) -> &MessengerRw;
    fn get_global_dispatcher(&self) -> nrg_messenger::MessageBox;
    fn get_messagebox(&self) -> MessageBox;
    fn get_listener(&self) -> Listener;
    fn node(&self) -> &WidgetNode;
    fn node_mut(&mut self) -> &mut WidgetNode;
    fn graphics(&self) -> &WidgetGraphics;
    fn graphics_mut(&mut self) -> &mut WidgetGraphics;
    fn state(&self) -> &WidgetState;
    fn state_mut(&mut self) -> &mut WidgetState;
    fn is_initialized(&self) -> bool;
    fn mark_as_initialized(&mut self);
}
