use inox_core::System;
use inox_messenger::MessageHubRc;
use inox_platform::{Window, WindowEvent};
use inox_resources::{ConfigBase, SharedDataRc};
use inox_serialize::read_from_file;

use crate::config::Config;

pub struct WindowSystem {
    window: Window,
    shared_data: SharedDataRc,
    message_hub: MessageHubRc,
}

impl WindowSystem {
    pub fn new(window: Window, shared_data: &SharedDataRc, message_hub: &MessageHubRc) -> Self {
        Self {
            window,
            shared_data: shared_data.clone(),
            message_hub: message_hub.clone(),
        }
    }
}

impl System for WindowSystem {
    fn read_config(&mut self, plugin_name: &str) {
        let mut config = Config::default();
        config = read_from_file(
            config.get_filepath(plugin_name).as_path(),
            self.shared_data.serializable_registry(),
        );

        self.message_hub
            .send_event(WindowEvent::RequestChangeTitle(config.title.clone()));
        self.message_hub
            .send_event(WindowEvent::RequestChangeSize(config.width, config.height));
        self.message_hub
            .send_event(WindowEvent::RequestChangePos(config.pos_x, config.pos_y));
        self.message_hub
            .send_event(WindowEvent::RequestChangeVisible(true));
    }
    fn should_run_when_not_focused(&self) -> bool {
        true
    }
    fn init(&mut self) {}
    fn run(&mut self) -> bool {
        self.window.update()
    }
    fn uninit(&mut self) {}
}
