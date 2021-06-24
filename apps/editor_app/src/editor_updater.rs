use std::{
    any::TypeId,
    collections::VecDeque,
    path::PathBuf,
    time::{Duration, Instant},
};

use super::config::*;
use super::nodes_registry::*;
use super::widgets::*;

use nrg_camera::Camera;
use nrg_core::*;
use nrg_graphics::{
    FontInstance, MaterialInstance, MeshData, MeshInstance, PipelineInstance, RenderPassInstance,
    ViewInstance,
};
use nrg_gui::*;
use nrg_math::{Vector2, Vector3, Zero};
use nrg_messenger::{read_messages, Message, MessageChannel, MessengerRw};
use nrg_platform::*;
use nrg_resources::{SharedData, SharedDataRw};
use nrg_serialize::*;

pub struct EditorUpdater {
    id: SystemId,
    frame_seconds: VecDeque<Instant>,
    shared_data: SharedDataRw,
    global_messenger: MessengerRw,
    job_handler: JobHandlerRw,
    config: Config,
    fps_text: Uid,
    properties_id: Uid,
    graph_id: Uid,
    main_menu_id: Uid,
    message_channel: MessageChannel,
    nodes_registry: NodesRegistry,
    camera: Camera,
    move_camera_with_mouse: bool,
    last_mouse_pos: Vector2,
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
            nodes_registry: NodesRegistry::new(&shared_data, &global_messenger),
            shared_data,
            global_messenger,
            job_handler,
            config: config.clone(),
            fps_text: INVALID_UID,
            properties_id: INVALID_UID,
            graph_id: INVALID_UID,
            main_menu_id: INVALID_UID,
            message_channel,
            camera: Camera::new([0., 2., -2.].into(), [0., 0., 0.].into()),
            move_camera_with_mouse: false,
            last_mouse_pos: Vector2::zero(),
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

    fn register_nodes(&mut self) -> &mut Self {
        self.nodes_registry.register::<GraphNode>();
        self.nodes_registry.register::<Icon>();
        self.nodes_registry.register::<TextBox>();
        self
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
        self.register_nodes();

        self.global_messenger
            .write()
            .unwrap()
            .register_messagebox::<KeyEvent>(self.message_channel.get_messagebox())
            .register_messagebox::<MouseEvent>(self.message_channel.get_messagebox())
            .register_messagebox::<WindowEvent>(self.message_channel.get_messagebox())
            .register_messagebox::<WidgetEvent>(self.message_channel.get_messagebox())
            .register_messagebox::<DialogEvent>(self.message_channel.get_messagebox())
            .register_messagebox::<NodesEvent>(self.message_channel.get_messagebox());

        let mut main_menu = MainMenu::new(&self.shared_data, &self.global_messenger);
        self.main_menu_id = main_menu.id();
        main_menu.fill_nodes_from_registry(&self.nodes_registry);
        Gui::get()
            .write()
            .unwrap()
            .get_root_mut()
            .add_child(Box::new(main_menu));

        let widget = PropertiesPanel::new(&self.shared_data, &self.global_messenger);
        self.properties_id = widget.id();
        Gui::get()
            .write()
            .unwrap()
            .get_root_mut()
            .add_child(Box::new(widget));

        let mut fps_text = Text::new(&self.shared_data, &self.global_messenger);
        fps_text
            .set_text("FPS: ")
            .horizontal_alignment(HorizontalAlignment::Right)
            .vertical_alignment(VerticalAlignment::Top);
        self.fps_text = fps_text.id();
        Gui::get()
            .write()
            .unwrap()
            .get_root_mut()
            .add_child(Box::new(fps_text));

        let graph = Graph::new(&self.shared_data, &self.global_messenger);
        self.graph_id = graph.id();
        Gui::get()
            .write()
            .unwrap()
            .get_root_mut()
            .add_child(Box::new(graph));
    }

    fn run(&mut self) -> bool {
        self.update_events()
            .update_camera()
            .update_fps_counter()
            .update_widgets();

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

        self.global_messenger
            .write()
            .unwrap()
            .unregister_messagebox::<KeyEvent>(self.message_channel.get_messagebox())
            .unregister_messagebox::<MouseEvent>(self.message_channel.get_messagebox())
            .unregister_messagebox::<WindowEvent>(self.message_channel.get_messagebox())
            .unregister_messagebox::<WidgetEvent>(self.message_channel.get_messagebox())
            .unregister_messagebox::<DialogEvent>(self.message_channel.get_messagebox())
            .unregister_messagebox::<NodesEvent>(self.message_channel.get_messagebox());
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
    fn update_camera(&mut self) -> &mut Self {
        if SharedData::has_resources_of_type::<ViewInstance>(&self.shared_data) {
            let view_id = SharedData::get_resourceid_at_index::<ViewInstance>(&self.shared_data, 0);
            let view = self.camera.get_view_matrix();
            let proj = nrg_math::perspective(
                nrg_math::Deg(45.),
                Screen::get_size().x / Screen::get_size().y,
                0.001,
                1000.0,
            );
            ViewInstance::update_view(&self.shared_data, view_id, view);
            ViewInstance::update_proj(&self.shared_data, view_id, proj);
        }
        self
    }
    fn update_fps_counter(&mut self) -> &mut Self {
        nrg_profiler::scoped_profile!("update_fps_counter");

        let now = Instant::now();
        let one_sec_before = now - Duration::from_secs(1);
        self.frame_seconds.push_back(now);
        self.frame_seconds.retain(|t| *t >= one_sec_before);

        let num_fps = self.frame_seconds.len();
        let text_id = self.fps_text;
        if let Some(text) = Gui::get()
            .read()
            .unwrap()
            .get_root()
            .get_child_mut::<Text>(text_id)
        {
            let str = format!("FPS: {}", num_fps);
            text.set_text(str.as_str());
        }

        self
    }
    fn update_widgets(&mut self) {
        nrg_profiler::scoped_profile!("update_widgets");

        Gui::update_widgets(&self.job_handler, true);
    }

    fn load_pipelines(&mut self) {
        for render_pass_data in self.config.render_passes.iter() {
            RenderPassInstance::create(&self.shared_data, render_pass_data);
        }

        for pipeline_data in self.config.pipelines.iter() {
            PipelineInstance::create(&self.shared_data, pipeline_data);
        }

        if let Some(pipeline_data) = self.config.pipelines.iter().find(|p| p.name.eq("UI")) {
            let pipeline_id =
                PipelineInstance::find_id_from_name(&self.shared_data, pipeline_data.name.as_str());
            if let Some(default_font_path) = self.config.fonts.first() {
                FontInstance::create_from_path(&self.shared_data, pipeline_id, default_font_path);
            }
        }

        if let Some(pipeline_data) = self.config.pipelines.iter().find(|p| p.name.eq("Grid")) {
            let pipeline_id =
                PipelineInstance::find_id_from_name(&self.shared_data, pipeline_data.name.as_str());
            let material_id =
                MaterialInstance::create_from_pipeline(&self.shared_data, pipeline_id);
            let mut mesh_data = MeshData::default();
            mesh_data.add_quad_default([-1., -1., 1., 1.].into(), 0.);
            let mesh_id = MeshInstance::create(&self.shared_data, mesh_data);
            MaterialInstance::add_mesh(&self.shared_data, material_id, mesh_id);
        }
    }

    fn load_graph(&mut self, filename: PathBuf) {
        if !filename.is_dir() && filename.exists() {
            Gui::get()
                .write()
                .unwrap()
                .get_root_mut()
                .remove_child(self.graph_id);
            let new_graph = Graph::load(&self.shared_data, &self.global_messenger, filename);
            self.graph_id = new_graph.id();
            Gui::get()
                .write()
                .unwrap()
                .get_root_mut()
                .add_child(Box::new(new_graph));
        }
    }

    fn save_graph(&mut self, mut filename: PathBuf) {
        if let Some(graph) = Gui::get()
            .read()
            .unwrap()
            .get_root()
            .get_child_mut::<Graph>(self.graph_id)
        {
            graph.node_mut().set_name(filename.to_str().unwrap());
            if filename.extension().is_none() {
                filename.set_extension("graph");
            }
            serialize_to_file(graph, filename);
        }
    }

    fn update_events(&mut self) -> &mut Self {
        nrg_profiler::scoped_profile!("update_events");

        read_messages(self.message_channel.get_listener(), |msg| {
            if msg.type_id() == TypeId::of::<NodesEvent>() {
                let event = msg.as_any().downcast_ref::<NodesEvent>().unwrap();
                let NodesEvent::Create(widget_name) = event;
                if let Some(graph) = Gui::get()
                    .read()
                    .unwrap()
                    .get_root()
                    .get_child_mut::<Graph>(self.graph_id)
                {
                    let mut widget = self.nodes_registry.create_from_name(widget_name.clone());
                    widget
                        .get_global_messenger()
                        .write()
                        .unwrap()
                        .register_messagebox::<WidgetEvent>(widget.get_messagebox())
                        .register_messagebox::<MouseEvent>(widget.get_messagebox());

                    widget
                        .state_mut()
                        .set_draggable(true)
                        .set_selectable(true)
                        .set_horizontal_alignment(HorizontalAlignment::Center)
                        .set_vertical_alignment(VerticalAlignment::Center);
                    graph.add_child(widget);
                }
            } else if msg.type_id() == TypeId::of::<DialogEvent>() {
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
                        self.load_graph(filename.clone());
                    } else if should_save {
                        println!("Saving {:?}", filename);
                        self.save_graph(filename.clone());
                    }
                }
            } else if msg.type_id() == TypeId::of::<MouseEvent>() {
                let event = msg.as_any().downcast_ref::<MouseEvent>().unwrap();
                if event.state == MouseState::Down && event.button == MouseButton::Left {
                    self.move_camera_with_mouse = true;
                    self.last_mouse_pos = [event.x as f32, event.y as f32].into();
                } else if event.state == MouseState::Up && event.button == MouseButton::Left {
                    self.move_camera_with_mouse = false;
                    self.last_mouse_pos = [event.x as f32, event.y as f32].into();
                }
                if event.state == MouseState::Move && self.move_camera_with_mouse {
                    let mut rotation_angle = Vector3::zero();

                    rotation_angle.y = event.x as f32 - self.last_mouse_pos.x;
                    rotation_angle.x = event.y as f32 - self.last_mouse_pos.y;

                    self.camera.rotate(rotation_angle * 0.01);

                    self.last_mouse_pos = [event.x as f32, event.y as f32].into();
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

                let mut movement = Vector3::zero();
                if event.code == Key::W {
                    movement.z += 0.1;
                } else if event.code == Key::S {
                    movement.z -= 0.1;
                } else if event.code == Key::A {
                    movement.x += 0.1;
                } else if event.code == Key::D {
                    movement.x -= 0.1;
                }
                self.camera.translate(movement);

                let mut rotation_angle = Vector3::zero();
                if event.code == Key::ArrowUp {
                    rotation_angle.x -= 0.1;
                } else if event.code == Key::ArrowDown {
                    rotation_angle.x += 0.1;
                } else if event.code == Key::ArrowRight {
                    rotation_angle.y += 0.1;
                } else if event.code == Key::ArrowLeft {
                    rotation_angle.y -= 0.1;
                }
                self.camera.rotate(rotation_angle);
            } else if msg.type_id() == TypeId::of::<WindowEvent>() {
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
            } else if msg.type_id() == TypeId::of::<WidgetEvent>() {
                self.move_camera_with_mouse = false;
                let event = msg.as_any().downcast_ref::<WidgetEvent>().unwrap();
                if let WidgetEvent::Released(widget_uid, _mouse) = *event {
                    self.global_messenger
                        .write()
                        .unwrap()
                        .get_dispatcher()
                        .write()
                        .unwrap()
                        .send(PropertiesEvent::GetProperties(widget_uid).as_boxed())
                        .ok();

                    if let Some(properties) = Gui::get()
                        .write()
                        .unwrap()
                        .get_root_mut()
                        .get_child_mut::<PropertiesPanel>(self.properties_id)
                    {
                        properties.reset();
                        properties.add_string(
                            "UID:",
                            widget_uid.to_simple().to_string().as_str(),
                            false,
                        );
                    }
                }
            }
        });
        self
    }
}
