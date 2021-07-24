use std::any::TypeId;

use nrg_core::{System, SystemId};
use nrg_graphics::{
    FontInstance, FontRc, PipelineInstance, PipelineRc, RenderPassInstance, RenderPassRc,
};
use nrg_messenger::{read_messages, Message, MessageChannel, MessengerRw};
use nrg_platform::{WindowEvent, DEFAULT_DPI};
use nrg_resources::{ConfigBase, DataTypeResource, FileResource, SharedDataRw};
use nrg_serialize::deserialize_from_file;
use nrg_ui::Screen;

use crate::config::Config;

pub struct MainSystem {
    id: SystemId,
    config: Config,
    shared_data: SharedDataRw,
    global_messenger: MessengerRw,
    message_channel: MessageChannel,
    pipelines: Vec<PipelineRc>,
    render_passes: Vec<RenderPassRc>,
    fonts: Vec<FontRc>,
}

impl MainSystem {
    pub fn new(shared_data: SharedDataRw, global_messenger: MessengerRw) -> Self {
        let message_channel = MessageChannel::default();

        global_messenger
            .write()
            .unwrap()
            .register_messagebox::<WindowEvent>(message_channel.get_messagebox());
        Self {
            id: SystemId::new(),
            config: Config::default(),
            shared_data,
            global_messenger,
            message_channel,
            pipelines: Vec::new(),
            render_passes: Vec::new(),
            fonts: Vec::new(),
        }
    }

    fn load_pipelines(&mut self) {
        for render_pass_data in self.config.render_passes.iter() {
            self.render_passes
                .push(RenderPassInstance::create_from_data(
                    &self.shared_data,
                    render_pass_data.clone(),
                ));
        }

        for pipeline_data in self.config.pipelines.iter() {
            self.pipelines.push(PipelineInstance::create_from_data(
                &self.shared_data,
                pipeline_data.clone(),
            ));
        }

        if let Some(default_font_path) = self.config.fonts.first() {
            self.fonts.push(FontInstance::create_from_file(
                &self.shared_data,
                default_font_path,
            ));
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

    fn window_init(&self) {
        self.send_event(WindowEvent::RequestChangeTitle(self.config.title.clone()).as_boxed());
        self.send_event(
            WindowEvent::RequestChangeSize(self.config.width, self.config.height).as_boxed(),
        );
        self.send_event(
            WindowEvent::RequestChangePos(self.config.pos_x, self.config.pos_y).as_boxed(),
        );
        self.send_event(WindowEvent::RequestChangeVisible(true).as_boxed());
    }

    fn process_messages(&mut self) {
        read_messages(self.message_channel.get_listener(), |msg| {
            if msg.type_id() == TypeId::of::<WindowEvent>() {
                let event = msg.as_any().downcast_ref::<WindowEvent>().unwrap();
                match *event {
                    WindowEvent::SizeChanged(width, height) => {
                        Screen::change_size(width, height);
                    }
                    WindowEvent::DpiChanged(x, _y) => {
                        Screen::change_scale_factor(x / DEFAULT_DPI);
                    }
                    _ => {}
                }
            }
        });
    }
}

impl System for MainSystem {
    fn id(&self) -> nrg_core::SystemId {
        self.id
    }

    fn should_run_when_not_focused(&self) -> bool {
        false
    }
    fn init(&mut self) {
        let path = self.config.get_filepath();
        deserialize_from_file(&mut self.config, path);

        self.window_init();
        self.load_pipelines();

        Screen::create(
            self.config.width,
            self.config.height,
            self.config.scale_factor,
        );
    }

    fn run(&mut self) -> bool {
        self.process_messages();
        true
    }

    fn uninit(&mut self) {}
}
