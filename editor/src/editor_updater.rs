use std::{
    any::TypeId,
    collections::VecDeque,
    path::PathBuf,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use super::config::*;
use super::widgets::*;

use nrg_core::*;
use nrg_events::*;
use nrg_graphics::*;
use nrg_gui::*;
use nrg_messenger::{read_messages, MessageChannel, MessengerRw};
use nrg_platform::*;
use nrg_resources::{SharedData, SharedDataRw};
use nrg_serialize::*;

pub struct EditorUpdater {
    id: SystemId,
    frame_seconds: VecDeque<Instant>,
    shared_data: SharedDataRw,
    global_messenger: MessengerRw,
    config: Config,
    fps_text: Uid,
    fps_widget_id: Uid,
    canvas_id: Uid,
    node_id: Uid,
    main_menu_id: Uid,
    history_panel_id: Uid,
    widgets: Vec<Arc<RwLock<Box<dyn Widget>>>>,
    message_channel: MessageChannel,
}

impl EditorUpdater {
    pub fn new(shared_data: SharedDataRw, global_messenger: MessengerRw, config: &Config) -> Self {
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
            config: config.clone(),
            widgets: Vec::new(),
            fps_text: INVALID_UID,
            fps_widget_id: INVALID_UID,
            canvas_id: INVALID_UID,
            node_id: INVALID_UID,
            main_menu_id: INVALID_UID,
            history_panel_id: INVALID_UID,
            message_channel,
        }
    }

    pub fn registered_event_types(&self, history: &mut EventsHistory) {
        history.register_event_as_undoable::<TextEvent>(&self.global_messenger);
        history.register_event_as_undoable::<CheckboxEvent>(&self.global_messenger);
    }
}

impl System for EditorUpdater {
    fn id(&self) -> SystemId {
        self.id
    }

    fn init(&mut self) {
        self.load_pipelines();
        self.create_screen();

        self.global_messenger
            .write()
            .unwrap()
            .register_messagebox::<DialogEvent>(self.message_channel.get_messagebox());

        let mut history_panel = HistoryPanel::new(&self.shared_data, &self.global_messenger);
        let history = history_panel.get_history();
        self.registered_event_types(history);
        self.history_panel_id = history_panel.id();

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

        self.widgets.push(Arc::new(RwLock::new(Box::new(canvas))));
        self.widgets.push(Arc::new(RwLock::new(Box::new(widget))));
        self.widgets
            .push(Arc::new(RwLock::new(Box::new(main_menu))));
        self.widgets
            .push(Arc::new(RwLock::new(Box::new(history_panel))));

        /*
        let node = GraphNode::new(&self.shared_data, &self.global_messenger);
        self.node_id = node.id();
        self.widgets.push(Arc::new(RwLock::new(Box::new(node))));
        */
    }

    fn run(&mut self) -> (bool, Vec<Job>) {
        self.update_events();

        self.update_fps_counter();

        let jobs = self.update_widgets();

        (true, jobs)
    }
    fn uninit(&mut self) {
        for w in self.widgets.iter() {
            w.write().unwrap().uninit();
        }
    }
}

impl EditorUpdater {
    pub fn get_widget<W>(&mut self, uid: Uid) -> Option<&mut W>
    where
        W: Widget,
    {
        let mut result: Option<&mut W> = None;
        self.widgets.iter_mut().for_each(|w| {
            if w.read().unwrap().id() == uid {
                unsafe {
                    let mut data = w.write().unwrap();
                    let ptr = data.as_mut();
                    let widget = ptr as *mut dyn Widget as *mut W;
                    result = Some(&mut *widget);
                }
            }
        });
        result
    }
    fn create_screen(&mut self) {
        let window = SharedData::get_unique_resource::<Window>(&self.shared_data);

        Screen::create(
            window.get().get_width(),
            window.get().get_heigth(),
            window.get().get_scale_factor(),
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
        if let Some(widget) = self.get_widget::<Panel>(self.fps_widget_id) {
            if let Some(text) = widget.node_mut().get_child::<Text>(text_id) {
                let str = format!("FPS: {}", num_fps);
                text.set_text(str.as_str());
            }
        }

        self
    }
    fn update_widgets(&mut self) -> Vec<Job> {
        nrg_profiler::scoped_profile!("update_widgets");
        let mut jobs = Vec::new();

        let size = Screen::get_size();
        let mut is_visible = false;
        let entire_screen = Screen::get_draw_area();
        let draw_area = {
            if let Some(main_menu) = self.get_widget::<MainMenu>(self.main_menu_id) {
                is_visible = main_menu.show_history();
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
        if let Some(history_panel) = self.get_widget::<HistoryPanel>(self.history_panel_id) {
            history_panel.set_visible(is_visible);
        }

        for (i, w) in self.widgets.iter_mut().enumerate() {
            let job = {
                let job_name = format!("widget[{}]", i);
                let widget = w.clone();
                if widget.read().unwrap().id() == self.main_menu_id {
                    Job::new(job_name.as_str(), move || {
                        widget.write().unwrap().update(entire_screen, entire_screen);
                    })
                } else {
                    Job::new(job_name.as_str(), move || {
                        widget.write().unwrap().update(draw_area, entire_screen);
                    })
                }
            };
            jobs.push(job);
        }
        /*
        for (_, w) in self.widgets.iter_mut().enumerate() {
            if w.read().unwrap().id() == self.node_id {
                let widget = w.clone();
                let job = Job::new("Treeview", move || {
                    widget.write().unwrap().update(draw_area);
                });
                jobs.push(job);
            }
        }
        */

        jobs
    }

    fn load_pipelines(&mut self) {
        for pipeline_data in self.config.pipelines.iter() {
            PipelineInstance::create(&self.shared_data, pipeline_data);
        }

        let pipeline_id = PipelineInstance::find_id_from_name(
            &self.shared_data,
            self.config.pipelines.first().unwrap().name.as_str(),
        );
        FontInstance::create_from_path(
            &self.shared_data,
            pipeline_id,
            self.config.fonts.first().unwrap(),
        );
    }

    fn load_node(&mut self, name: &str) {
        let dir = "./data/widgets/";
        if let Ok(dir) = std::fs::read_dir(dir) {
            dir.for_each(|entry| {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if !path.is_dir() && path.to_str().unwrap().contains(name) {
                        if let Some(node_widget_index) = self
                            .widgets
                            .iter()
                            .position(|w| w.read().unwrap().id() == self.node_id)
                        {
                            let old_node = self.widgets.swap_remove(node_widget_index);
                            old_node.write().unwrap().uninit();
                        }
                        let new_node =
                            GraphNode::load(&self.shared_data, &self.global_messenger, path);
                        self.node_id = new_node.id();

                        self.widgets.push(Arc::new(RwLock::new(Box::new(new_node))));
                    }
                }
            });
        }
    }

    fn save_node(&mut self, name: &str) {
        if let Some(node) = self.get_widget::<GraphNode>(self.node_id) {
            node.node_mut().set_name(name);
            let mut path = PathBuf::from(name);
            if path.extension().is_none() {
                path.set_extension("widget");
            }
            let filepath = PathBuf::from(format!("./data/widgets/{}", path.to_str().unwrap()));
            serialize_to_file(node, filepath);
        }
    }

    fn update_events(&mut self) {
        nrg_profiler::scoped_profile!("update_events");

        read_messages(self.message_channel.get_listener(), |msg| {
            if msg.type_id() == TypeId::of::<DialogEvent>() {
                let event = msg.as_any().downcast_ref::<DialogEvent>().unwrap();
                if let DialogEvent::Confirmed(_widget_id, requester_uid, text) = event {
                    let mut should_load = false;
                    let mut should_save = false;
                    if let Some(menu) = self.get_widget::<MainMenu>(self.main_menu_id) {
                        should_load = menu.is_open_uid(*requester_uid);
                        should_save = menu.is_save_uid(*requester_uid);
                    }
                    if should_load {
                        self.load_node(text);
                    } else if should_save {
                        self.save_node(text);
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
