use sabi_core::System;

use sabi_messenger::{read_messages, GlobalMessenger, MessageChannel, MessengerRw};
use sabi_platform::WindowEvent;
use std::env;
use std::path::PathBuf;
use std::{any::TypeId, path::Path};

use sabi_resources::SharedDataRc;
use sabi_ui::{DialogEvent, DialogOp};

use crate::widgets::ContentBrowser;

#[allow(dead_code)]
pub struct ContentBrowserUpdater {
    shared_data: SharedDataRc,
    global_messenger: MessengerRw,
    message_channel: MessageChannel,
    content_browser: Option<ContentBrowser>,
}

impl ContentBrowserUpdater {
    pub fn new(shared_data: &SharedDataRc, global_messenger: &MessengerRw) -> Self {
        let message_channel = MessageChannel::default();

        Self {
            shared_data: shared_data.clone(),
            global_messenger: global_messenger.clone(),
            message_channel,
            content_browser: None,
        }
    }
}

impl System for ContentBrowserUpdater {
    fn read_config(&mut self, _plugin_name: &str) {}
    fn should_run_when_not_focused(&self) -> bool {
        false
    }

    fn init(&mut self) {
        self.global_messenger
            .write()
            .unwrap()
            .register_messagebox::<DialogEvent>(self.message_channel.get_messagebox());

        let args: Vec<String> = env::args().collect();
        let mut operation = DialogOp::Open;
        let mut folder = PathBuf::from(".");
        let mut extension = String::from("scene");

        let mut is_operation = false;
        let mut is_folder = false;
        let mut is_extension = false;
        (1..args.len()).for_each(|i| {
            if args[i].as_str() == "-folder" {
                is_folder = true;
            } else if args[i].as_str() == "-extension" {
                is_extension = true;
            } else if args[i].as_str() == "-operation" {
                is_operation = true;
            } else if is_operation {
                is_operation = false;
                operation = DialogOp::from(args[i].as_str());
            } else if is_folder {
                is_folder = false;
                folder = PathBuf::from(args[i].as_str())
            } else if is_extension {
                is_extension = false;
                extension = String::from(args[i].as_str())
            }
        });

        self.create_content_browser(operation, folder.as_path(), extension);
    }

    fn run(&mut self) -> bool {
        self.update_events();

        true
    }
    fn uninit(&mut self) {
        self.destroy_content_browser();

        self.global_messenger
            .write()
            .unwrap()
            .unregister_messagebox::<DialogEvent>(self.message_channel.get_messagebox());
    }
}

impl ContentBrowserUpdater {
    fn create_content_browser(
        &mut self,
        operation: DialogOp,
        path: &Path,
        extension: String,
    ) -> &mut Self {
        if self.content_browser.is_none() {
            match operation {
                DialogOp::Open => {
                    self.global_messenger
                        .send_event(WindowEvent::RequestChangeTitle("Open File".to_string()));
                }
                DialogOp::Save => {
                    self.global_messenger
                        .send_event(WindowEvent::RequestChangeTitle("Save File".to_string()));
                }
                DialogOp::New => {
                    self.global_messenger
                        .send_event(WindowEvent::RequestChangeTitle("New File".to_string()));
                }
            }

            let content_browser = ContentBrowser::new(
                &self.shared_data,
                &self.global_messenger,
                operation,
                path,
                extension,
            );
            self.content_browser = Some(content_browser);
        }
        self
    }
    fn destroy_content_browser(&mut self) -> &mut Self {
        self.content_browser = None;
        self
    }

    fn update_events(&mut self) -> &mut Self {
        sabi_profiler::scoped_profile!("update_events");

        read_messages(self.message_channel.get_listener(), |msg| {
            if msg.type_id() == TypeId::of::<DialogEvent>() {
                let event = msg.as_any().downcast_ref::<DialogEvent>().unwrap();
                match event {
                    DialogEvent::Request(operation, path) => {
                        self.create_content_browser(
                            *operation,
                            path.as_path(),
                            "scene".to_string(),
                        );
                    }
                    DialogEvent::Confirmed(operation, filename) => {
                        let extension = filename.extension().unwrap().to_str().unwrap();
                        match operation {
                            DialogOp::Open => {
                                debug_log(format!("Loading {:?}", filename).as_str());
                                if extension.contains("scene") {
                                    //self.load_object(filename.as_path());
                                }
                            }
                            DialogOp::Save => {
                                debug_log(format!("Saving {:?}", filename).as_str());
                                if extension.contains("scene") {}
                            }
                            DialogOp::New => {}
                        }
                    }
                    _ => {}
                }
            }
        });
        self
    }
}
