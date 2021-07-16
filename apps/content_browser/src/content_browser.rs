use std::{any::TypeId, path::PathBuf};

use nrg_core::{App, JobHandlerRw, PhaseWithSystems, System, SystemId};
use nrg_graphics::{
    FontInstance, FontRc, PipelineInstance, PipelineRc, RenderPassInstance, RenderPassRc,
};
use nrg_gui::{
    BaseWidget, DialogEvent, FolderDialog, Gui, HorizontalAlignment, Screen, VerticalAlignment,
    WidgetCreator,
};
use nrg_messenger::{read_messages, Message, MessageChannel, MessengerRw};
use nrg_platform::{WindowEvent, DEFAULT_DPI};
use nrg_resources::{ConfigBase, DataTypeResource, FileResource, SharedDataRw, DATA_FOLDER};
use nrg_serialize::{deserialize_from_file, Uid, INVALID_UID};

use crate::config::Config;

const CONTENT_BROWSER_UPDATE_PHASE: &str = "CONTENT_BROWSER_UPDATE_PHASE";

#[repr(C)]
pub struct ContentBrowser {
    id: SystemId,
}

impl Default for ContentBrowser {
    fn default() -> Self {
        Self {
            id: SystemId::new(),
        }
    }
}

impl ContentBrowser {
    pub fn prepare(&mut self, app: &mut App) {
        let mut update_phase = PhaseWithSystems::new(CONTENT_BROWSER_UPDATE_PHASE);
        let system = ContentBrowserSystem::new(
            app.get_shared_data(),
            app.get_global_messenger(),
            app.get_job_handler(),
        );
        self.id = system.id();
        update_phase.add_system(system);
        app.create_phase_before(update_phase, "RENDERING_UPDATE");
    }

    pub fn unprepare(&mut self, app: &mut App) {
        let update_phase: &mut PhaseWithSystems = app.get_phase_mut(CONTENT_BROWSER_UPDATE_PHASE);
        update_phase.remove_system(&self.id);
        app.destroy_phase(CONTENT_BROWSER_UPDATE_PHASE);
    }
}

struct ContentBrowserSystem {
    id: SystemId,
    config: Config,
    shared_data: SharedDataRw,
    global_messenger: MessengerRw,
    job_handler: JobHandlerRw,
    message_channel: MessageChannel,
    pipelines: Vec<PipelineRc>,
    render_passes: Vec<RenderPassRc>,
    fonts: Vec<FontRc>,
    folder_dialog_id: Uid,
}

impl ContentBrowserSystem {
    pub fn new(
        shared_data: SharedDataRw,
        global_messenger: MessengerRw,
        job_handler: JobHandlerRw,
    ) -> Self {
        Gui::create(
            shared_data.clone(),
            global_messenger.clone(),
            job_handler.clone(),
        );

        let message_channel = MessageChannel::default();

        global_messenger
            .write()
            .unwrap()
            .register_messagebox::<WindowEvent>(message_channel.get_messagebox())
            .register_messagebox::<DialogEvent>(message_channel.get_messagebox());
        Self {
            id: SystemId::new(),
            config: Config::default(),
            shared_data,
            global_messenger,
            job_handler,
            message_channel,
            pipelines: Vec::new(),
            render_passes: Vec::new(),
            fonts: Vec::new(),
            folder_dialog_id: INVALID_UID,
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
                        Gui::invalidate_all_widgets();
                    }
                    WindowEvent::DpiChanged(x, _y) => {
                        Screen::change_scale_factor(x / DEFAULT_DPI);
                        Gui::invalidate_all_widgets();
                    }
                    _ => {}
                }
            } else if msg.type_id() == TypeId::of::<DialogEvent>() {
                let event = msg.as_any().downcast_ref::<DialogEvent>().unwrap();
                match &event {
                    DialogEvent::Confirmed(widget_id, _requester_uid, _text) => {
                        if *widget_id == self.folder_dialog_id {
                            self.send_event(WindowEvent::Close.as_boxed());
                        }
                    }
                    DialogEvent::Canceled(widget_id) => {
                        if *widget_id == self.folder_dialog_id {
                            self.send_event(WindowEvent::Close.as_boxed());
                        }
                    }
                }
            }
        });
    }

    fn add_content(&mut self) -> &mut Self {
        let mut folder_dialog = FolderDialog::new(&self.shared_data, &self.global_messenger);
        folder_dialog
            .vertical_alignment(VerticalAlignment::Stretch)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .set_title("Open")
            .set_folder(PathBuf::from(DATA_FOLDER).as_path())
            .editable(false);
        self.folder_dialog_id = folder_dialog.id();

        Gui::get()
            .write()
            .unwrap()
            .get_root_mut()
            .add_child(Box::new(folder_dialog));

        self
    }
}

impl System for ContentBrowserSystem {
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

        self.add_content();
    }

    fn run(&mut self) -> bool {
        self.process_messages();

        Gui::update_widgets(&self.job_handler, false);

        true
    }

    fn uninit(&mut self) {}
}
