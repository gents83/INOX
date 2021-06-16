use std::{
    any::TypeId,
    collections::VecDeque,
    path::PathBuf,
    time::{Duration, Instant},
};

use super::config::*;
use super::widgets::*;

use nrg_core::*;
use nrg_graphics::*;
use nrg_gui::*;
use nrg_messenger::{read_messages, Message, MessageChannel, MessengerRw};
use nrg_platform::*;
use nrg_resources::SharedDataRw;
use nrg_serialize::*;

pub struct EditorUpdater {
    id: SystemId,
    frame_seconds: VecDeque<Instant>,
    shared_data: SharedDataRw,
    global_messenger: MessengerRw,
    job_handler: JobHandlerRw,
    config: Config,
    fps_text: Uid,
    fps_widget_id: Uid,
    canvas_id: Uid,
    node_id: Uid,
    main_menu_id: Uid,
    message_channel: MessageChannel,
}

impl EditorUpdater {
    pub fn new(
        shared_data: SharedDataRw,
        global_messenger: MessengerRw,
        job_handler: JobHandlerRw,
        config: &Config,
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
            .register_messagebox::<KeyEvent>(message_channel.get_messagebox());
        global_messenger
            .write()
            .unwrap()
            .register_messagebox::<WidgetEvent>(message_channel.get_messagebox());
        Self {
            id: SystemId::new(),
            frame_seconds: VecDeque::default(),
            shared_data,
            global_messenger,
            job_handler,
            config: config.clone(),
            fps_text: INVALID_UID,
            fps_widget_id: INVALID_UID,
            canvas_id: INVALID_UID,
            node_id: INVALID_UID,
            main_menu_id: INVALID_UID,
            message_channel,
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
}

impl System for EditorUpdater {
    fn id(&self) -> SystemId {
        self.id
    }

    fn should_run_when_not_focused(&self) -> bool {
        false
    }

    fn init(&mut self) {
        self.window_init();
        self.load_pipelines();
        self.create_screen();

        self.global_messenger
            .write()
            .unwrap()
            .register_messagebox::<DialogEvent>(self.message_channel.get_messagebox());

        let main_menu = MainMenu::new(&self.shared_data, &self.global_messenger);
        self.main_menu_id = main_menu.id();

        let mut canvas = Canvas::new(&self.shared_data, &self.global_messenger);
        canvas.move_to_layer(-1.);
        self.canvas_id = canvas.id();

        let mut widget = Panel::new(&self.shared_data, &self.global_messenger);
        widget
            .position([300., 300.].into())
            .size([300., 800.].into())
            .selectable(false)
            .horizontal_alignment(HorizontalAlignment::Right)
            .space_between_elements(20)
            .fill_type(ContainerFillType::Vertical)
            .style(WidgetStyle::DefaultBackground)
            .move_to_layer(0.5);
        self.fps_widget_id = widget.id();

        let mut fps_text = Text::new(&self.shared_data, &self.global_messenger);
        fps_text.set_text("FPS: ");
        self.fps_text = widget.add_child(Box::new(fps_text));

        let mut checkbox = Checkbox::new(&self.shared_data, &self.global_messenger);
        checkbox.with_label("Checkbox");
        widget.add_child(Box::new(checkbox));

        let mut textbox = TextBox::new(&self.shared_data, &self.global_messenger);
        textbox
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .with_label("Sample:")
            .editable(true)
            .set_text("Ciao");
        widget.add_child(Box::new(textbox));

        Gui::get()
            .write()
            .unwrap()
            .get_root_mut()
            .add_child(Box::new(canvas));
        Gui::get()
            .write()
            .unwrap()
            .get_root_mut()
            .add_child(Box::new(widget));
        Gui::get()
            .write()
            .unwrap()
            .get_root_mut()
            .add_child(Box::new(main_menu));
        /*
        let node = GraphNode::new(&self.shared_data, &self.global_messenger);
        self.node_id = node.id();
        Gui::get()
            .write()
            .unwrap()
            .get_root_mut()
            .add_child(Box::new(node));
        */
    }

    fn run(&mut self) -> bool {
        self.update_events();

        self.update_fps_counter();

        self.update_widgets();

        true
    }
    fn uninit(&mut self) {
        Gui::get()
            .write()
            .unwrap()
            .get_root_mut()
            .propagate_on_children_mut(|w| {
                w.uninit();
            });
    }
}

impl EditorUpdater {
    fn create_screen(&mut self) {
        Screen::create(
            self.config.width,
            self.config.height,
            self.config.scale_factor,
        );
    }
    fn update_fps_counter(&mut self) -> &mut Self {
        nrg_profiler::scoped_profile!("update_fps_counter");

        let now = Instant::now();
        let one_sec_before = now - Duration::from_secs(1);
        self.frame_seconds.push_back(now);
        self.frame_seconds.retain(|t| *t >= one_sec_before);

        let num_fps = self.frame_seconds.len();
        let text_id = self.fps_text;
        if let Some(widget) = Gui::get()
            .read()
            .unwrap()
            .get_root()
            .get_child_mut::<Panel>(self.fps_widget_id)
        {
            if let Some(text) = widget.node().get_child_mut::<Text>(text_id) {
                let str = format!("FPS: {}", num_fps);
                text.set_text(str.as_str());
            }
        }

        self
    }
    fn update_widgets(&mut self) {
        nrg_profiler::scoped_profile!("update_widgets");

        let size = Screen::get_size();
        let entire_screen = Screen::get_draw_area();
        let draw_area = {
            if let Some(main_menu) = Gui::get()
                .read()
                .unwrap()
                .get_root()
                .get_child_mut::<MainMenu>(self.main_menu_id)
            {
                [
                    0.,
                    main_menu.state().get_size().y + DEFAULT_WIDGET_SIZE[1],
                    size.x,
                    size.y - (main_menu.state().get_size().y + DEFAULT_WIDGET_SIZE[1]),
                ]
                .into()
            } else {
                entire_screen
            }
        };

        Gui::get()
            .write()
            .unwrap()
            .get_root_mut()
            .get_children()
            .iter()
            .for_each(|w| {
                let widget = w.clone();
                let job_name = format!("widget[{}]", widget.read().unwrap().node().get_name());
                if widget.read().unwrap().id() == self.main_menu_id {
                    self.job_handler
                        .write()
                        .unwrap()
                        .add_job(job_name.as_str(), move || {
                            widget.write().unwrap().update(entire_screen, entire_screen);
                        })
                } else {
                    self.job_handler
                        .write()
                        .unwrap()
                        .add_job(job_name.as_str(), move || {
                            widget.write().unwrap().update(draw_area, entire_screen);
                        })
                }
            });
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

    fn load_node(&mut self, filename: PathBuf) {
        if !filename.is_dir() && filename.exists() {
            Gui::get()
                .write()
                .unwrap()
                .get_root_mut()
                .remove_child(self.node_id);
            let new_node = GraphNode::load(&self.shared_data, &self.global_messenger, filename);
            self.node_id = new_node.id();
            Gui::get()
                .write()
                .unwrap()
                .get_root_mut()
                .add_child(Box::new(new_node));
        }
    }

    fn save_node(&mut self, mut filename: PathBuf) {
        if let Some(node) = Gui::get()
            .read()
            .unwrap()
            .get_root()
            .get_child_mut::<GraphNode>(self.node_id)
        {
            node.node_mut().set_name(filename.to_str().unwrap());
            if filename.extension().is_none() {
                filename.set_extension("widget");
            }
            serialize_to_file(node, filename);
        }
    }

    fn update_events(&mut self) {
        nrg_profiler::scoped_profile!("update_events");

        read_messages(self.message_channel.get_listener(), |msg| {
            if msg.type_id() == TypeId::of::<DialogEvent>() {
                let event = msg.as_any().downcast_ref::<DialogEvent>().unwrap();
                if let DialogEvent::Confirmed(_widget_id, requester_uid, filename) = event {
                    let mut should_load = false;
                    let mut should_save = false;
                    if let Some(menu) = Gui::get()
                        .read()
                        .unwrap()
                        .get_root()
                        .get_child_mut::<MainMenu>(self.main_menu_id)
                    {
                        should_load = menu.is_open_uid(*requester_uid);
                        should_save = menu.is_save_uid(*requester_uid);
                    }
                    if should_load {
                        println!("Loading {:?}", filename);
                        self.load_node(filename.clone());
                    } else if should_save {
                        println!("Saving {:?}", filename);
                        self.save_node(filename.clone());
                    }
                }
            } else if msg.type_id() == TypeId::of::<KeyEvent>() {
                let event = msg.as_any().downcast_ref::<KeyEvent>().unwrap();
                if event.state == InputState::JustPressed && event.code == Key::F5 {
                    println!("Launch game");
                    let result = std::process::Command::new("nrg_game_app").spawn().is_ok();
                    if !result {
                        println!("Failed to execute process");
                    }
                }
            } else if msg.type_id() == TypeId::of::<WindowEvent>() {
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
