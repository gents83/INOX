use std::{
    any::TypeId,
    collections::VecDeque,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use super::config::*;
use super::widget_registry::*;
use super::widgets::*;

use nrg_camera::Camera;
use nrg_core::*;
use nrg_graphics::{
    FontInstance, FontRc, MaterialInstance, MaterialRc, MeshData, MeshInstance, MeshRc,
    PipelineInstance, PipelineRc, RenderPassInstance, RenderPassRc, ViewInstance,
};
use nrg_gui::{
    BaseWidget, GraphNode, Gui, HorizontalAlignment, Icon, PropertiesEvent, PropertiesPanel,
    Screen, Text, TextBox, VerticalAlignment, WidgetCreator, WidgetDataGetter, WidgetEvent,
};
use nrg_math::{
    compute_distance_between_ray_and_oob, InnerSpace, MatBase, Matrix4, SquareMatrix, Vector2,
    Vector3, Vector4, Zero,
};
use nrg_messenger::{read_messages, Message, MessageChannel, MessengerRw};
use nrg_platform::*;
use nrg_resources::{
    DataTypeResource, FileResource, SerializableResource, SharedData, SharedDataRw,
};
use nrg_scene::{Object, ObjectId, Scene, SceneRc};
use nrg_serialize::*;
use nrg_ui::{DialogEvent, DialogOp};

pub struct EditorUpdater {
    id: SystemId,
    show_fps: bool,
    frame_seconds: VecDeque<Instant>,
    shared_data: SharedDataRw,
    global_messenger: MessengerRw,
    job_handler: JobHandlerRw,
    config: Config,
    fps_text: Uid,
    properties_id: Uid,
    graph_id: Uid,
    main_menu: Option<MainMenu>,
    debug_info: Option<DebugInfo>,
    message_channel: MessageChannel,
    nodes_registry: WidgetRegistry,
    camera: Camera,
    move_camera_with_mouse: bool,
    last_mouse_pos: Vector2,
    pipelines: Vec<PipelineRc>,
    render_passes: Vec<RenderPassRc>,
    fonts: Vec<FontRc>,
    grid_material: MaterialRc,
    grid_mesh: MeshRc,
    scene: SceneRc,
    selected_object: ObjectId,
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

        let mut camera = Camera::new([20., 20., -20.].into(), [0., 0., 0.].into(), true);
        camera.set_projection(45., Screen::get_size().x, Screen::get_size().y, 0.1, 1000.);

        Self {
            id: SystemId::new(),
            pipelines: Vec::new(),
            render_passes: Vec::new(),
            fonts: Vec::new(),
            show_fps: false,
            frame_seconds: VecDeque::default(),
            nodes_registry: WidgetRegistry::new(&shared_data, &global_messenger),
            shared_data,
            global_messenger,
            job_handler,
            config: config.clone(),
            fps_text: INVALID_UID,
            properties_id: INVALID_UID,
            graph_id: INVALID_UID,
            main_menu: None,
            debug_info: None,
            message_channel,
            camera,
            move_camera_with_mouse: false,
            last_mouse_pos: Vector2::zero(),
            grid_material: MaterialRc::default(),
            grid_mesh: MeshRc::default(),
            scene: SceneRc::default(),
            selected_object: INVALID_UID,
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

        self.create_main_menu()
            .create_fps_counter()
            .create_properties_panel()
            .create_graph();

        self.show_fps(self.show_fps);

        self.create_scene();
    }

    fn run(&mut self) -> bool {
        self.update_events()
            .update_camera()
            .update_fps_counter()
            .update_widgets();

        self.scene
            .resource()
            .get_mut()
            .update_hierarchy(&self.shared_data);

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
    fn create_main_menu(&mut self) -> &mut Self {
        let main_menu = MainMenu::new(&self.shared_data, &self.global_messenger);
        self.main_menu = Some(main_menu);
        self
    }
    fn create_debug_info(&mut self) -> &mut Self {
        let debug_info = DebugInfo::new(&self.shared_data);
        self.debug_info = Some(debug_info);
        self
    }
    fn destroy_debug_info(&mut self) -> &mut Self {
        self.debug_info = None;
        self
    }
    fn create_graph(&mut self) -> &mut Self {
        let graph = Graph::new(&self.shared_data, &self.global_messenger);
        self.graph_id = graph.id();
        Gui::get()
            .write()
            .unwrap()
            .get_root_mut()
            .add_child(Box::new(graph));
        self
    }
    fn create_properties_panel(&mut self) -> &mut Self {
        let properties_panel = PropertiesPanel::new(&self.shared_data, &self.global_messenger);
        self.properties_id = properties_panel.id();
        Gui::get()
            .write()
            .unwrap()
            .get_root_mut()
            .add_child(Box::new(properties_panel));

        self
    }
    fn create_fps_counter(&mut self) -> &mut Self {
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

        self
    }
    fn create_scene(&mut self) -> &mut Self {
        self.scene = SharedData::add_resource::<Scene>(&self.shared_data, Scene::default());
        self
    }
    fn create_screen(&mut self) -> &mut Self {
        Screen::create(
            self.config.width,
            self.config.height,
            self.config.scale_factor,
        );
        self
    }
    fn update_camera(&mut self) -> &mut Self {
        if SharedData::has_resources_of_type::<ViewInstance>(&self.shared_data) {
            let views = SharedData::get_resources_of_type::<ViewInstance>(&self.shared_data);
            let view = views.first().unwrap();
            let view_matrix = self.camera.get_view_matrix();
            let proj_matrix = self.camera.get_proj_matrix();
            view.resource().get_mut().update_view(view_matrix);
            view.resource().get_mut().update_proj(proj_matrix);
        }
        self
    }
    fn show_fps(&mut self, show_fps: bool) -> &mut Self {
        self.show_fps = show_fps;

        let text_id = self.fps_text;
        if let Some(text) = Gui::get()
            .read()
            .unwrap()
            .get_root()
            .get_child_mut::<Text>(text_id)
        {
            text.visible(show_fps);
        }
        self
    }
    fn update_fps_counter(&mut self) -> &mut Self {
        if !self.show_fps {
            return self;
        }

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

        if let Some(main_menu) = &mut self.main_menu {
            let show_debug_info = main_menu.show_debug_info();
            let is_debug_info_created = self.debug_info.is_some();
            if show_debug_info && !is_debug_info_created {
                self.create_debug_info();
            } else if !show_debug_info && is_debug_info_created {
                self.destroy_debug_info();
            }
        }

        Gui::update_widgets(&self.job_handler, true);
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

        if let Some(pipeline_data) = self.config.pipelines.iter().find(|p| p.name.eq("Grid")) {
            if let Some(pipeline) =
                PipelineInstance::find_from_name(&self.shared_data, pipeline_data.name.as_str())
            {
                self.grid_material =
                    MaterialInstance::create_from_pipeline(&self.shared_data, pipeline);
            }
            let mut mesh_data = MeshData::default();
            mesh_data.add_quad_default([-1., -1., 1., 1.].into(), 0.);
            let mesh = MeshInstance::create_from_data(&self.shared_data, mesh_data);
            mesh.resource()
                .get_mut()
                .set_material(self.grid_material.clone());
            self.grid_mesh = mesh;
        }
    }

    fn load_object(&mut self, filename: &Path) {
        if !filename.is_dir() && filename.exists() {
            self.scene.resource().get_mut().clear();
            let object = Object::create_from_file(&self.shared_data, filename);
            self.scene.resource().get_mut().add_object(object);
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

    fn update_selected_object(&mut self, mouse_pos: &Vector2) -> &mut Self {
        self.selected_object = INVALID_UID;
        let view = self.camera.get_view_matrix();
        let proj = self.camera.get_proj_matrix();

        let screen_size = Screen::get_size();
        // The ray Start and End positions, in Normalized Device Coordinates (Have you read Tutorial 4 ?)
        let ray_start = Vector4::new(0., 0., 0., 1.);
        let ray_end = Vector4::new(
            ((mouse_pos.x / screen_size.x) * 2.) - 1.,
            ((mouse_pos.y / screen_size.y) * 2.) - 1.,
            1.,
            1.,
        );

        let inv_proj = proj.invert().unwrap();
        let inv_view = view.invert().unwrap();

        let mut ray_start_camera = inv_proj * ray_start;
        ray_start_camera /= ray_start_camera.w;
        let mut ray_start_world = inv_view * ray_start_camera;
        ray_start_world /= ray_start_world.w;

        let mut ray_end_camera = inv_proj * ray_end;
        ray_end_camera /= ray_end_camera.w;
        let mut ray_end_world = inv_view * ray_end_camera;
        ray_end_world /= ray_end_world.w;

        let ray_dir_world = ray_end_world - ray_start_world;
        let ray_dir_world = ray_dir_world.normalize();

        if compute_distance_between_ray_and_oob(
            ray_start_world.xyz(),
            ray_dir_world.xyz(),
            [-5., -5., -5.].into(),
            [5., 5., 5.].into(),
            Matrix4::default_identity(),
        ) {
            println!("Inside");
        } else {
            println!("Outside");
        }

        self
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
                if let DialogEvent::Confirmed(operation, filename) = event {
                    let extension = filename.extension().unwrap().to_str().unwrap();
                    match operation {
                        DialogOp::Open => {
                            println!("Loading {:?}", filename);
                            if extension.contains("widget") {
                                self.load_graph(filename.clone());
                            } else if extension.contains("object_data") {
                                self.load_object(filename.as_path());
                            }
                        }
                        DialogOp::Save => {
                            println!("Saving {:?}", filename);
                            if extension.contains("widget") {
                                self.save_graph(filename.clone());
                            } else if extension.contains("object_data") {
                            }
                        }
                        DialogOp::New => {}
                    }
                }
            } else if msg.type_id() == TypeId::of::<MouseEvent>() {
                let event = msg.as_any().downcast_ref::<MouseEvent>().unwrap();
                if event.state == MouseState::Down && event.button == MouseButton::Left {
                    self.move_camera_with_mouse = true;
                    self.last_mouse_pos = [event.x as f32, event.y as f32].into();
                } else if event.state == MouseState::Up && event.button == MouseButton::Left {
                    let mouse_pos = [event.x as f32, event.y as f32].into();
                    self.update_selected_object(&mouse_pos);

                    self.move_camera_with_mouse = false;
                    self.last_mouse_pos = mouse_pos;
                }
                if event.state == MouseState::Move && self.move_camera_with_mouse {
                    let mut rotation_angle = Vector3::zero();

                    rotation_angle.x = event.y as f32 - self.last_mouse_pos.y;
                    rotation_angle.y = self.last_mouse_pos.x - event.x as f32;

                    self.camera.rotate(rotation_angle * 0.01);

                    self.last_mouse_pos = [event.x as f32, event.y as f32].into();
                }
            } else if msg.type_id() == TypeId::of::<KeyEvent>() {
                let event = msg.as_any().downcast_ref::<KeyEvent>().unwrap();

                if event.code == Key::F1 && event.state == InputState::JustPressed {
                    self.show_fps(!self.show_fps);
                }

                let mut movement = Vector3::zero();
                if event.code == Key::W {
                    movement.z += 1.;
                } else if event.code == Key::S {
                    movement.z -= 1.;
                } else if event.code == Key::A {
                    movement.x += 1.;
                } else if event.code == Key::D {
                    movement.x -= 1.;
                }
                self.camera.translate(movement);
            } else if msg.type_id() == TypeId::of::<WindowEvent>() {
                let event = msg.as_any().downcast_ref::<WindowEvent>().unwrap();
                match *event {
                    WindowEvent::SizeChanged(width, height) => {
                        Screen::change_size(width, height);
                        self.camera.set_projection(
                            45.,
                            Screen::get_size().x,
                            Screen::get_size().y,
                            0.1,
                            1000.,
                        );
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
