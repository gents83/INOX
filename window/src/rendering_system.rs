use std::any::TypeId;

use nrg_core::*;
use nrg_graphics::*;
use nrg_messenger::{read_messages, MessageChannel, MessengerRw};
use nrg_platform::WindowEvent;

pub struct RenderingSystem {
    id: SystemId,
    renderer: RendererRw,
    is_enabled: bool,
    message_channel: MessageChannel,
}

impl RenderingSystem {
    pub fn new(renderer: RendererRw, global_messenger: &MessengerRw) -> Self {
        let message_channel = MessageChannel::default();
        global_messenger
            .write()
            .unwrap()
            .register_messagebox::<WindowEvent>(message_channel.get_messagebox());
        Self {
            id: SystemId::new(),
            renderer,
            message_channel,
            is_enabled: false,
        }
    }
}

unsafe impl Send for RenderingSystem {}
unsafe impl Sync for RenderingSystem {}

impl System for RenderingSystem {
    fn id(&self) -> SystemId {
        self.id
    }
    fn init(&mut self) {}

    fn run(&mut self) -> bool {
        let state = self.renderer.read().unwrap().state();
        if state != RendererState::Prepared {
            return true;
        }

        read_messages(self.message_channel.get_listener(), |msg| {
            if msg.type_id() == TypeId::of::<WindowEvent>() {
                let e = msg.as_any().downcast_ref::<WindowEvent>().unwrap();
                match e {
                    WindowEvent::Show => {
                        self.is_enabled = true;
                    }
                    WindowEvent::Hide => {
                        self.is_enabled = false;
                    }
                    _ => {}
                }
            }
        });

        if self.is_enabled {
            let mut renderer = self.renderer.write().unwrap();
            renderer.draw();
        }

        true
    }
    fn uninit(&mut self) {}
}
