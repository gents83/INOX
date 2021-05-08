use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use super::config::*;
use super::widgets::*;

use nrg_core::*;
use nrg_events::*;
use nrg_graphics::*;
use nrg_gui::*;
use nrg_platform::*;
use nrg_resources::SharedDataRw;
use nrg_serialize::*;

pub struct EditorUpdater {
    id: SystemId,
    frame_seconds: VecDeque<Instant>,
    shared_data: SharedDataRw,
    config: Config,
    is_ctrl_pressed: bool,
    history: EventsHistory,
    fps_text_widget_id: Uid,
    main_menu: MainMenu,
    canvas: Canvas,
    widget: Panel,
    history_panel: HistoryPanel,
    node: GraphNode,
}

impl EditorUpdater {
    fn register(&mut self) {
        self.history.register_event_as_undoable::<TextEvent>();
        self.history.register_event_as_undoable::<CheckboxEvent>();
    }

    pub fn new(shared_data: &SharedDataRw, config: &Config) -> Self {
        let mut updater = Self {
            id: SystemId::new(),
            frame_seconds: VecDeque::default(),
            shared_data: shared_data.clone(),
            config: config.clone(),
            is_ctrl_pressed: false,
            history_panel: HistoryPanel::default(),
            history: EventsHistory::default(),
            main_menu: MainMenu::default(),
            canvas: Canvas::default(),
            widget: Panel::default(),
            fps_text_widget_id: INVALID_UID,
            node: GraphNode::default(),
        };
        updater.register();

        updater
    }
}

impl System for EditorUpdater {
    fn id(&self) -> SystemId {
        self.id
    }

    fn init(&mut self) {
        self.load_pipelines();
        self.create_screen();

        self.node.init(&self.shared_data);

        self.main_menu.init(&self.shared_data);
        self.canvas.init(&self.shared_data);
        self.canvas.move_to_layer(0.);
        self.history_panel.init(&self.shared_data);

        self.widget.init(&self.shared_data);
        self.widget
            .position([300., 300.].into())
            .size([300., 800.].into())
            .selectable(false)
            .horizontal_alignment(HorizontalAlignment::Right)
            .space_between_elements(20)
            .fill_type(ContainerFillType::Vertical)
            .style(WidgetStyle::DefaultBackground)
            .move_to_layer(0.5);

        let mut fps_text = Text::default();
        fps_text.init(&self.shared_data);
        fps_text.set_text("FPS: ");
        self.fps_text_widget_id = self.widget.add_child(Box::new(fps_text));

        let mut checkbox = Checkbox::default();
        checkbox.init(&self.shared_data);
        checkbox.with_label(&self.shared_data, "Checkbox");
        self.widget.add_child(Box::new(checkbox));

        let mut textbox = TextBox::default();
        textbox.init(&self.shared_data);
        textbox
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .with_label("Sample:")
            .editable(true)
            .set_text("Ciao");
        self.widget.add_child(Box::new(textbox));

        /*
                let dir = "./data/widgets/";
                if let Ok(dir) = std::fs::read_dir(dir) {
                    for entry in dir {
                        if let Ok(entry) = entry {
                            let path = entry.path();
                            if !path.is_dir() {
                                let mut boxed_node = Box::new(GraphNode::default());
                                deserialize_from_file(&mut boxed_node, path);
                                boxed_node.as_mut().init(&self.shared_data);
                                self.canvas.add_child(boxed_node);
                            }
                        }
                    }
                }
        */
    }

    fn run(&mut self) -> bool {
        Screen::update();

        self.update_keyboard_input();
        self.update_widgets();

        self.update_fps_counter();

        let read_data = self.shared_data.read().unwrap();
        let events_rw = &mut *read_data.get_unique_resource_mut::<EventsRw>();
        self.history.update(events_rw);
        true
    }
    fn uninit(&mut self) {
        /*
                let childrens = self.canvas.get_data_mut().node.get_children();
                for child in childrens {
                    let filepath = PathBuf::from(format!(
                        "./data/widgets/{}.widget",
                        child.id().to_simple().to_string()
                    ));
                    serialize_to_file(child, filepath);
                }
        */
        self.node.uninit(&self.shared_data);
        self.canvas.uninit(&self.shared_data);
        self.widget.uninit(&self.shared_data);
        self.main_menu.uninit(&self.shared_data);
        self.history_panel.uninit(&self.shared_data);
    }
}

impl EditorUpdater {
    fn create_screen(&mut self) {
        let read_data = self.shared_data.read().unwrap();
        let window = &*read_data.get_unique_resource::<Window>();
        let events_rw = &mut *read_data.get_unique_resource_mut::<EventsRw>();

        Screen::create(
            window.get_width(),
            window.get_heigth(),
            window.get_scale_factor(),
            events_rw.clone(),
        );
    }
    fn update_fps_counter(&mut self) -> &mut Self {
        nrg_profiler::scoped_profile!("update_fps_counter");

        let now = Instant::now();
        let one_sec_before = now - Duration::from_secs(1);
        self.frame_seconds.retain(|t| *t >= one_sec_before);
        self.frame_seconds.push_back(now);

        if let Some(widget) = self
            .widget
            .get_data_mut()
            .node
            .get_child::<Text>(self.fps_text_widget_id)
        {
            let str = format!("FPS: {}", self.frame_seconds.len());
            widget.set_text(str.as_str());
        }
        self
    }
    fn update_widgets(&mut self) {
        nrg_profiler::scoped_profile!("update_widgets");

        self.main_menu.update(&self.shared_data);

        let draw_area = [
            0.,
            self.main_menu.get_size().y + DEFAULT_WIDGET_SIZE[1],
            Screen::get_size().x,
            Screen::get_size().y - (self.main_menu.get_size().y + DEFAULT_WIDGET_SIZE[1]),
        ]
        .into();

        self.node.update(draw_area, &self.shared_data);

        {
            nrg_profiler::scoped_profile!("widget.update");
            self.widget.update(draw_area, &self.shared_data);
        }

        {
            nrg_profiler::scoped_profile!("history_panel.update");
            self.history_panel
                .set_visible(self.main_menu.show_history());
            self.history_panel
                .update(draw_area, &self.shared_data, &mut self.history);
        }

        {
            nrg_profiler::scoped_profile!("canvas.update");
            self.canvas.update(draw_area, &self.shared_data);
        }
    }

    fn load_pipelines(&mut self) {
        for pipeline_data in self.config.pipelines.iter() {
            PipelineInstance::create(&self.shared_data, pipeline_data);
        }

        let pipeline_id = PipelineInstance::find_id_from_name(&self.shared_data, "UI");
        FontInstance::create_from_path(
            &self.shared_data,
            pipeline_id,
            self.config.fonts.first().unwrap(),
        );
    }

    fn update_keyboard_input(&mut self) {
        nrg_profiler::scoped_profile!("update_keyboard_input");

        let read_data = self.shared_data.read().unwrap();
        let events_rw = &mut *read_data.get_unique_resource_mut::<EventsRw>();
        let events = events_rw.read().unwrap();

        if let Some(key_events) = events.read_all_events::<KeyEvent>() {
            for event in key_events.iter() {
                if event.code == Key::Control {
                    if event.state == InputState::Pressed || event.state == InputState::JustPressed
                    {
                        self.is_ctrl_pressed = true;
                    } else if event.state == InputState::Released
                        || event.state == InputState::JustReleased
                    {
                        self.is_ctrl_pressed = false;
                    }
                } else if self.is_ctrl_pressed
                    && event.code == Key::Z
                    && event.state == InputState::JustPressed
                {
                    self.history.undo_last_event();
                } else if self.is_ctrl_pressed
                    && event.code == Key::Y
                    && event.state == InputState::JustPressed
                {
                    self.history.redo_last_event();
                } else if event.state == InputState::JustPressed && event.code == Key::F5 {
                    println!("Launch game");
                    let result = std::process::Command::new("nrg_game_app").spawn().is_ok();
                    if !result {
                        println!("Failed to execute process");
                    }
                }
            }
        }
    }
}
