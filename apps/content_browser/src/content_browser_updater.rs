use std::path::PathBuf;
use std::{any::TypeId, path::Path};

use super::config::*;
use super::widgets::*;

use nrg_core::*;
use nrg_graphics::{Font, Pipeline, RenderPass};
use nrg_messenger::{read_messages, Message, MessageChannel, MessengerRw};
use nrg_platform::WindowEvent;
use nrg_resources::DATA_FOLDER;
use nrg_resources::{DataTypeResource, Resource, SharedDataRw};
use nrg_ui::{DialogEvent, DialogOp};

#[allow(dead_code)]
pub struct ContentBrowserUpdater {
    id: SystemId,
    shared_data: SharedDataRw,
    global_messenger: MessengerRw,
    config: Config,
    message_channel: MessageChannel,
    pipelines: Vec<Resource<Pipeline>>,
    render_passes: Vec<Resource<RenderPass>>,
    fonts: Vec<Resource<Font>>,
    content_browser: Option<ContentBrowser>,
}

impl ContentBrowserUpdater {
    pub fn new(shared_data: SharedDataRw, global_messenger: MessengerRw, config: &Config) -> Self {
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

    fn send_event(&self, event: Box<dyn Message>) {
        self.global_messenger
            .read()
            .unwrap()
            .get_dispatcher()
            .write()
            .unwrap()
            .send(event)
            .ok();
    }

    fn window_init(&self) -> &Self {
        self.send_event(WindowEvent::RequestChangeTitle(self.config.title.clone()).as_boxed());
        self.send_event(
            WindowEvent::RequestChangeSize(self.config.width, self.config.height).as_boxed(),
        );
        self.send_event(
            WindowEvent::RequestChangePos(self.config.pos_x, self.config.pos_y).as_boxed(),
        );
        self.send_event(WindowEvent::RequestChangeVisible(true).as_boxed());
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
        self.load_pipelines();

        self.global_messenger
            .write()
            .unwrap()
            .register_messagebox::<DialogEvent>(self.message_channel.get_messagebox());

        self.create_content_browser(
            DialogOp::Open,
            PathBuf::from(DATA_FOLDER).as_path(),
            "object_data".to_string(),
        );
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

    fn load_pipelines(&mut self) {
        for render_pass_data in self.config.render_passes.iter() {
            self.render_passes.push(RenderPass::create_from_data(
                &self.shared_data,
                render_pass_data.clone(),
            ));
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
