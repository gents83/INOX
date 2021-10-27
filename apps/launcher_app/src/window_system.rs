use nrg_core::System;
use nrg_messenger::{send_global_event, MessengerRw};
use nrg_platform::{Window, WindowEvent};
use nrg_resources::ConfigBase;
use nrg_serialize::read_from_file;

use crate::config::Config;

pub struct WindowSystem {
    window: Window,
    global_messenger: MessengerRw,
}

impl WindowSystem {
    pub fn new(window: Window, global_messenger: &MessengerRw) -> Self {
        Self {
            window,
            global_messenger: global_messenger.clone(),
        }
    }
}

impl System for WindowSystem {
    fn read_config(&mut self, plugin_name: &str) {
        let mut config = Config::default();
        config = read_from_file(config.get_filepath(plugin_name).as_path());

        send_global_event(
            &self.global_messenger,
            WindowEvent::RequestChangeTitle(config.title.clone()),
        );
        send_global_event(
            &self.global_messenger,
            WindowEvent::RequestChangeSize(config.width, config.height),
        );
        send_global_event(
            &self.global_messenger,
            WindowEvent::RequestChangePos(config.pos_x, config.pos_y),
        );
        send_global_event(
            &self.global_messenger,
            WindowEvent::RequestChangeVisible(true),
        );
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
