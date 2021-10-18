use std::env;
use std::path::PathBuf;
use std::{any::TypeId, path::Path};

use super::config::*;
use super::widgets::*;

use nrg_core::*;
use nrg_graphics::{Font, Pipeline, RenderPass};
use nrg_messenger::{read_messages, send_global_event, MessageChannel, MessengerRw};
use nrg_platform::WindowEvent;

use nrg_resources::{DataTypeResource, Resource, SharedDataRc};
use nrg_serialize::generate_random_uid;
use nrg_ui::{DialogEvent, DialogOp};

#[allow(dead_code)]
pub struct ContentBrowserUpdater {
    id: SystemId,
    shared_data: SharedDataRc,
    global_messenger: MessengerRw,
    config: Config,
    message_channel: MessageChannel,
    pipelines: Vec<Resource<Pipeline>>,
    render_passes: Vec<Resource<RenderPass>>,
    fonts: Vec<Resource<Font>>,
    content_browser: Option<ContentBrowser>,
}

impl ContentBrowserUpdater {
    pub fn new(shared_data: SharedDataRc, global_messenger: MessengerRw, config: &Config) -> Self {
        let message_channel = MessageChannel::default();

        Self {
            id: SystemId::new(),
            pipelines: Vec::new(),
            render_passes: Vec::new(),
            fonts: Vec::new(),
            shared_data,
            global_messenger,
            config: config.clone(),
            message_channel,
            content_browser: None,
        }
    }

    fn window_init(&self) -> &Self {
        send_global_event(
            &self.global_messenger,
            WindowEvent::RequestChangeTitle(self.config.title.clone()),
        );
        send_global_event(
            &self.global_messenger,
            WindowEvent::RequestChangeSize(self.config.width, self.config.height),
        );
        send_global_event(
            &self.global_messenger,
            WindowEvent::RequestChangePos(self.config.pos_x, self.config.pos_y),
        );
        send_global_event(
            &self.global_messenger,
            WindowEvent::RequestChangeVisible(true),
        );
        self
    }
}

impl System for ContentBrowserUpdater {
    fn id(&self) -> SystemId {
        self.id
    }

    fn should_run_when_not_focused(&self) -> bool {
        false
    }

    fn init(&mut self) {
        self.window_init();
        self.load_render_passes();

        self.global_messenger
            .write()
            .unwrap()
            .register_messagebox::<DialogEvent>(self.message_channel.get_messagebox());

        let args: Vec<String> = env::args().collect();
        let mut operation = DialogOp::Open;
        let mut folder = PathBuf::from(".");
        let mut extension = String::from("object_data");

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
                    send_global_event(
                        &self.global_messenger,
                        WindowEvent::RequestChangeTitle("Open File".to_string()),
                    );
                }
                DialogOp::Save => {
                    send_global_event(
                        &self.global_messenger,
                        WindowEvent::RequestChangeTitle("Save File".to_string()),
                    );
                }
                DialogOp::New => {
                    send_global_event(
                        &self.global_messenger,
                        WindowEvent::RequestChangeTitle("New File".to_string()),
                    );
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

    fn load_render_passes(&mut self) {
        for render_pass_data in self.config.render_passes.iter() {
            let render_pass = RenderPass::create_from_data(
                &self.shared_data,
                &self.global_messenger,
                generate_random_uid(),
                render_pass_data.clone(),
            );
            self.render_passes.push(render_pass);
        }
    }

    fn update_events(&mut self) -> &mut Self {
        nrg_profiler::scoped_profile!("update_events");

        read_messages(self.message_channel.get_listener(), |msg| {
            if msg.type_id() == TypeId::of::<DialogEvent>() {
                let event = msg.as_any().downcast_ref::<DialogEvent>().unwrap();
                match event {
                    DialogEvent::Request(operation, path) => {
                        self.create_content_browser(
                            *operation,
                            path.as_path(),
                            "object_data".to_string(),
                        );
                    }
                    DialogEvent::Confirmed(operation, filename) => {
                        let extension = filename.extension().unwrap().to_str().unwrap();
                        match operation {
                            DialogOp::Open => {
                                println!("Loading {:?}", filename);
                                if extension.contains("object_data") {
                                    //self.load_object(filename.as_path());
                                }
                            }
                            DialogOp::Save => {
                                println!("Saving {:?}", filename);
                                if extension.contains("object_data") {}
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
