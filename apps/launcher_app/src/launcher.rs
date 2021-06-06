use std::{
    any::TypeId,
    path::{Path, PathBuf},
    process::Command,
};

use nrg_core::{App, JobHandlerRw, PhaseWithSystems, System, SystemId};
use nrg_graphics::{FontInstance, MaterialInstance, PipelineInstance, TextureInstance};
use nrg_gui::{
    BaseWidget, ContainerFillType, Gui, HorizontalAlignment, Icon, Panel, Screen,
    VerticalAlignment, WidgetDataGetter, WidgetEvent, WidgetStyle,
};
use nrg_math::Vector2;
use nrg_messenger::{read_messages, Message, MessageChannel, MessengerRw};
use nrg_platform::{WindowEvent, DEFAULT_DPI};
use nrg_resources::{ConfigBase, SharedDataRw};
use nrg_serialize::{deserialize_from_file, Uid, INVALID_UID};

use crate::config::Config;

const LAUNCHER_UPDATE_PHASE: &str = "LAUNCHER_UPDATE_PHASE";

#[repr(C)]
pub struct Launcher {
    id: SystemId,
}

impl Default for Launcher {
    fn default() -> Self {
        Self {
            id: SystemId::new(),
        }
    }
}

impl Launcher {
    pub fn prepare(&mut self, app: &mut App) {
        let mut update_phase = PhaseWithSystems::new(LAUNCHER_UPDATE_PHASE);
        let system = LauncherSystem::new(
            app.get_shared_data(),
            app.get_global_messenger(),
            app.get_job_handler(),
        );
        self.id = system.id();
        update_phase.add_system(system);
        app.create_phase_before(update_phase, "RENDERING_UPDATE");
    }

    pub fn unprepare(&mut self, app: &mut App) {
        let update_phase: &mut PhaseWithSystems = app.get_phase_mut(LAUNCHER_UPDATE_PHASE);
        update_phase.remove_system(&self.id);
        app.destroy_phase(LAUNCHER_UPDATE_PHASE);
    }
}

struct LauncherSystem {
    id: SystemId,
    config: Config,
    shared_data: SharedDataRw,
    global_messenger: MessengerRw,
    job_handler: JobHandlerRw,
    message_channel: MessageChannel,
    node_editor_id: Uid,
    game_id: Uid,
}

impl LauncherSystem {
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
            .register_messagebox::<WindowEvent>(message_channel.get_messagebox());
        global_messenger
            .write()
            .unwrap()
            .register_messagebox::<WidgetEvent>(message_channel.get_messagebox());
        Self {
            id: SystemId::new(),
            config: Config::default(),
            shared_data,
            global_messenger,
            job_handler,
            message_channel,
            node_editor_id: INVALID_UID,
            game_id: INVALID_UID,
        }
    }

    fn load_pipelines(&mut self) {
        for pipeline_data in self.config.pipelines.iter() {
            PipelineInstance::create(&self.shared_data, pipeline_data);
        }

        if let Some(pipeline_data) = self.config.pipelines.first() {
            let pipeline_id =
                PipelineInstance::find_id_from_name(&self.shared_data, pipeline_data.name.as_str());
            if let Some(default_font_path) = self.config.fonts.first() {
                FontInstance::create_from_path(&self.shared_data, pipeline_id, default_font_path);
            }
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
            }
            if msg.type_id() == TypeId::of::<WidgetEvent>() {
                let event = msg.as_any().downcast_ref::<WidgetEvent>().unwrap();
                if let WidgetEvent::Released(widget_id, _mouse_pos) = *event {
                    if widget_id == self.node_editor_id {
                        println!("Launch editor");
                        let result = Command::new("nrg_editor").spawn().is_ok();
                        if !result {
                            println!("Failed to execute process");
                        }
                    } else if widget_id == self.game_id {
                        println!("Launch game");
                        let result = Command::new("nrg_game").spawn().is_ok();
                        if !result {
                            println!("Failed to execute process");
                        }
                    }
                }
            }
        });
    }

    fn add_content(&mut self) -> &Self {
        let mut background = Panel::new(&self.shared_data, &self.global_messenger);
        background
            .vertical_alignment(VerticalAlignment::Stretch)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .fill_type(ContainerFillType::Horizontal)
            .space_between_elements((10. * Screen::get_scale_factor()) as u32)
            .use_space_before_and_after(true)
            .style(WidgetStyle::FullActive);

        let texture_id =
            TextureInstance::create_from_path(&self.shared_data, &Path::new("textures/NRG.png"));
        MaterialInstance::add_texture(
            &self.shared_data,
            background.graphics().get_material_id(),
            texture_id,
        );

        self.node_editor_id = background.add_child(Box::new(
            self.add_button(PathBuf::from("icons/editor.png").as_path(), "Node Editor"),
        ));

        Gui::get()
            .write()
            .unwrap()
            .get_root_mut()
            .add_child(Box::new(background));

        self
    }

    fn add_button(&self, icon_path: &Path, text: &str) -> Icon {
        let size: Vector2 = [200., 200.].into();

        let mut icon = Icon::new(&self.shared_data, &self.global_messenger);
        icon.size(size * Screen::get_scale_factor())
            .style(WidgetStyle::DefaultButton)
            .border_style(WidgetStyle::DefaultBorder)
            .border_width(5.)
            .selectable(true)
            .collapsed()
            .set_text(text)
            .set_texture(icon_path);

        icon
    }
}

impl System for LauncherSystem {
    fn id(&self) -> nrg_core::SystemId {
        self.id
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

        Gui::update_widgets(&self.job_handler);

        true
    }

    fn uninit(&mut self) {}
}
