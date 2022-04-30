#![allow(dead_code)]

use crate::{implement_unique_system_uid, ContextRc, System};
use inox_messenger::{Listener, MessageHubRc};
use inox_platform::{Window, WindowEvent};
use inox_resources::{ConfigBase, ConfigEvent, SharedDataRc};
use inox_serialize::read_from_file;

use crate::config::Config;

pub struct WindowSystem {
    config: Config,
    window: Window,
    shared_data: SharedDataRc,
    message_hub: MessageHubRc,
    listener: Listener,
}

impl WindowSystem {
    pub fn new(window: Window, context: &ContextRc) -> Self {
        let listener = Listener::new(context.message_hub());
        Self {
            config: Config::default(),
            window,
            shared_data: context.shared_data().clone(),
            message_hub: context.message_hub().clone(),
            listener,
        }
    }

    fn handle_events(&mut self) {
        self.listener
            .process_messages(|e: &ConfigEvent<Config>| match e {
                ConfigEvent::Loaded(filename, config) => {
                    if filename == self.config.get_filename() {
                        self.config = config.clone();
                        self.message_hub
                            .send_event(WindowEvent::RequestChangeTitle(config.title.clone()));
                        self.message_hub.send_event(WindowEvent::RequestChangeSize(
                            config.width,
                            config.height,
                        ));
                        self.message_hub
                            .send_event(WindowEvent::RequestChangePos(config.pos_x, config.pos_y));
                        self.message_hub
                            .send_event(WindowEvent::RequestChangeVisible(true));
                    }
                }
            });
    }
}

implement_unique_system_uid!(WindowSystem);

impl System for WindowSystem {
    fn read_config(&mut self, plugin_name: &str) {
        self.listener.register::<ConfigEvent<Config>>();
        let message_hub = self.message_hub.clone();
        let filename = self.config.get_filename().to_string();
        read_from_file(
            self.config.get_filepath(plugin_name).as_path(),
            self.shared_data.serializable_registry(),
            Box::new(move |data: Config| {
                message_hub.send_event(ConfigEvent::Loaded(filename.clone(), data));
            }),
        );
    }
    fn should_run_when_not_focused(&self) -> bool {
        true
    }
    fn init(&mut self) {}
    fn run(&mut self) -> bool {
        self.handle_events();
        self.window.update()
    }
    fn uninit(&mut self) {
        self.listener.unregister::<ConfigEvent<Config>>();
    }
}
