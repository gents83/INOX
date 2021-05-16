use nrg_messenger::{Listener, MessageBox, MessageChannel, MessengerRw};
use nrg_platform::MouseEvent;
use nrg_resources::SharedDataRw;
use nrg_serialize::{typetag, Deserialize, Serialize};

use crate::{WidgetEvent, WidgetGraphics, WidgetNode, WidgetState};

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
            message_channel: Self::create_widget_channel(&global_messenger),
            shared_data,
            global_messenger,
        }
    }
    fn create_widget_channel(global_messenger: &MessengerRw) -> MessageChannel {
        let message_channel = MessageChannel::default();
        global_messenger
            .write()
            .unwrap()
            .register_messagebox::<MouseEvent>(message_channel.get_messagebox());
        global_messenger
            .write()
            .unwrap()
            .register_messagebox::<WidgetEvent>(message_channel.get_messagebox());
        message_channel
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
