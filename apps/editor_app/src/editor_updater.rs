use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{any::TypeId, path::Path};

use super::config::*;
use super::widgets::*;

use nrg_camera::Camera;
use nrg_core::*;
use nrg_graphics::{
    FontInstance, FontRc, MaterialInstance, MaterialRc, MeshData, MeshInstance, MeshRc,
    PipelineInstance, PipelineRc, RenderPassInstance, RenderPassRc, ViewInstance,
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
    shared_data: SharedDataRw,
    global_messenger: MessengerRw,
    config: Config,
    message_channel: MessageChannel,
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
    main_menu: Option<MainMenu>,
    debug_info: Option<DebugInfo>,
    view3d: Option<View3D>,
    content_browser: Option<ContentBrowser>,
    show_debug_info: Arc<AtomicBool>,
}

impl EditorUpdater {
    pub fn new(shared_data: SharedDataRw, global_messenger: MessengerRw, config: &Config) -> Self {
        let message_channel = MessageChannel::default();

        global_messenger
            .write()
            .unwrap()
            .register_messagebox::<KeyEvent>(message_channel.get_messagebox());

        let mut camera = Camera::new([20., 20., -20.].into(), [0., 0., 0.].into(), true);
        camera.set_projection(45., config.width as _, config.height as _, 0.1, 1000.);

        Self {
            id: SystemId::new(),
            pipelines: Vec::new(),
            render_passes: Vec::new(),
            fonts: Vec::new(),
            shared_data,
            global_messenger,
            config: config.clone(),
            message_channel,
            camera,
            move_camera_with_mouse: false,
            last_mouse_pos: Vector2::zero(),
            grid_material: MaterialRc::default(),
            grid_mesh: MeshRc::default(),
            scene: SceneRc::default(),
            selected_object: INVALID_UID,
            main_menu: None,
            debug_info: None,
            view3d: None,
            content_browser: None,
            show_debug_info: Arc::new(AtomicBool::new(false)),
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

        self.global_messenger
            .write()
            .unwrap()
            .register_messagebox::<KeyEvent>(self.message_channel.get_messagebox())
            .register_messagebox::<MouseEvent>(self.message_channel.get_messagebox())
            .register_messagebox::<WindowEvent>(self.message_channel.get_messagebox())
            .register_messagebox::<DialogEvent>(self.message_channel.get_messagebox());

        self.create_main_menu().create_view3d().create_scene();
    }

    fn run(&mut self) -> bool {
        self.update_events().update_camera().update_widgets();

        self.scene
            .resource()
            .get_mut()
            .update_hierarchy(&self.shared_data);

        true
    }
    fn uninit(&mut self) {
        self.global_messenger
            .write()
            .unwrap()
            .unregister_messagebox::<KeyEvent>(self.message_channel.get_messagebox())
            .unregister_messagebox::<MouseEvent>(self.message_channel.get_messagebox())
            .unregister_messagebox::<WindowEvent>(self.message_channel.get_messagebox())
            .unregister_messagebox::<DialogEvent>(self.message_channel.get_messagebox());
    }
}

impl EditorUpdater {
    fn create_main_menu(&mut self) -> &mut Self {
        let main_menu = MainMenu::new(
            &self.shared_data,
            &self.global_messenger,
            self.show_debug_info.clone(),
        );
        self.main_menu = Some(main_menu);
        self
    }
    fn create_debug_info(&mut self) -> &mut Self {
        let debug_info = DebugInfo::new(&self.shared_data);
        self.debug_info = Some(debug_info);
        self.show_debug_info.store(true, Ordering::SeqCst);
        self
    }
    fn destroy_debug_info(&mut self) -> &mut Self {
        self.debug_info = None;
        self.show_debug_info.store(false, Ordering::SeqCst);
        self
    }
    fn create_view3d(&mut self) -> &mut Self {
        let view3d = View3D::new(&self.shared_data, &self.global_messenger);
        self.view3d = Some(view3d);
        self
    }
    fn create_content_browser(&mut self, operation: DialogOp, path: &Path) -> &mut Self {
        if self.content_browser.is_none() {
            let content_browser =
                ContentBrowser::new(&self.shared_data, &self.global_messenger, operation, path);
            self.content_browser = Some(content_browser);
        }
        self
    }
    fn destroy_content_browser(&mut self) -> &mut Self {
        self.content_browser = None;
        self
    }
    fn create_scene(&mut self) -> &mut Self {
        self.scene = SharedData::add_resource::<Scene>(&self.shared_data, Scene::default());
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
    fn update_widgets(&mut self) {
        nrg_profiler::scoped_profile!("update_widgets");

        let show_debug_info = self.show_debug_info.load(Ordering::SeqCst);
        let is_debug_info_created = self.debug_info.is_some();
        if show_debug_info && !is_debug_info_created {
            self.create_debug_info();
        } else if !show_debug_info && is_debug_info_created {
            self.destroy_debug_info();
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

    fn update_selected_object(&mut self, mouse_pos: &Vector2) -> &mut Self {
        self.selected_object = INVALID_UID;
        let view = self.camera.get_view_matrix();
        let proj = self.camera.get_proj_matrix();

        let screen_size: Vector2 = [self.config.width as f32, self.config.height as f32].into();
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
            if msg.type_id() == TypeId::of::<DialogEvent>() {
                let event = msg.as_any().downcast_ref::<DialogEvent>().unwrap();
                match event {
                    DialogEvent::Request(operation, path) => {
                        self.create_content_browser(*operation, path.as_path());
                    }
                    DialogEvent::Confirmed(operation, filename) => {
                        let extension = filename.extension().unwrap().to_str().unwrap();
                        match operation {
                            DialogOp::Open => {
                                println!("Loading {:?}", filename);
                                if extension.contains("object_data") {
                                    self.load_object(filename.as_path());
                                }
                            }
                            DialogOp::Save => {
                                println!("Saving {:?}", filename);
                                if extension.contains("object_data") {}
                            }
                            DialogOp::New => {}
                        }
                        self.destroy_content_browser();
                    }
                    _ => {
                        self.destroy_content_browser();
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
                    let is_debug_info_created = self.debug_info.is_some();
                    if !is_debug_info_created {
                        self.create_debug_info();
                    } else {
                        self.destroy_debug_info();
                    }
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
                if let WindowEvent::SizeChanged(width, height) = *event {
                    self.camera
                        .set_projection(45., width as _, height as _, 0.1, 1000.);
                }
            }
        });
        self
    }
}
