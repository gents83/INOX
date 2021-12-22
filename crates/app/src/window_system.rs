use sabi_core::System;
use sabi_messenger::{GlobalMessenger, MessengerRw};
use sabi_platform::{Window, WindowEvent};
use sabi_resources::{ConfigBase, SharedDataRc};
use sabi_serialize::read_from_file;

use crate::config::Config;

pub struct WindowSystem {
    window: Window,
    shared_data: SharedDataRc,
    global_messenger: MessengerRw,
}

impl WindowSystem {
    pub fn new(window: Window, shared_data: &SharedDataRc, global_messenger: &MessengerRw) -> Self {
        Self {
            window,
            shared_data: shared_data.clone(),
            global_messenger: global_messenger.clone(),
        }
    }
}

impl System for WindowSystem {
    fn read_config(&mut self, plugin_name: &str) {
        let mut config = Config::default();
        config = read_from_file(
            config.get_filepath(plugin_name).as_path(),
            &self.shared_data.serializable_registry(),
        );

        self.global_messenger
            .send_event(WindowEvent::RequestChangeTitle(config.title.clone()));
        self.global_messenger
            .send_event(WindowEvent::RequestChangeSize(config.width, config.height));
        self.global_messenger
            .send_event(WindowEvent::RequestChangePos(config.pos_x, config.pos_y));
        self.global_messenger
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
