use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{any::TypeId, path::Path};

use super::config::*;
use super::widgets::*;

use nrg_core::*;
use nrg_graphics::{
    FontInstance, FontRc, MaterialInstance, MaterialRc, MeshData, MeshInstance, MeshRc,
    PipelineInstance, PipelineRc, RenderPassInstance, RenderPassRc, TextureInstance, ViewInstance,
};
use nrg_messenger::{read_messages, Message, MessageChannel, MessengerRw};
use nrg_platform::{InputState, Key, KeyEvent, MouseEvent, WindowEvent};
use nrg_resources::{
    DataTypeResource, FileResource, SerializableResource, SharedData, SharedDataRw,
};
use nrg_scene::{Hitbox, Object, Scene, SceneRc, Transform};
use nrg_ui::{DialogEvent, DialogOp, UIPropertiesRegistry, UIWidget};

pub struct EditorUpdater {
    id: SystemId,
    shared_data: SharedDataRw,
    global_messenger: MessengerRw,
    config: Config,
    message_channel: MessageChannel,
    pipelines: Vec<PipelineRc>,
    render_passes: Vec<RenderPassRc>,
    fonts: Vec<FontRc>,
    grid_material: MaterialRc,
    grid_mesh: MeshRc,
    scene: SceneRc,
    main_menu: Option<MainMenu>,
    debug_info: Option<DebugInfo>,
    view3d: Option<View3D>,
    properties: Option<Properties>,
    content_browser: Option<ContentBrowser>,
    show_debug_info: Arc<AtomicBool>,
    ui_registry: Arc<UIPropertiesRegistry>,
}

impl EditorUpdater {
    pub fn new(shared_data: SharedDataRw, global_messenger: MessengerRw, config: &Config) -> Self {
        let message_channel = MessageChannel::default();

        global_messenger
            .write()
            .unwrap()
            .register_messagebox::<KeyEvent>(message_channel.get_messagebox());

        Self {
            id: SystemId::new(),
            pipelines: Vec::new(),
            render_passes: Vec::new(),
            fonts: Vec::new(),
            shared_data,
            global_messenger,
            config: config.clone(),
            message_channel,
            grid_material: MaterialRc::default(),
            grid_mesh: MeshRc::default(),
            scene: SceneRc::default(),
            main_menu: None,
            debug_info: None,
            view3d: None,
            properties: None,
            content_browser: None,
            show_debug_info: Arc::new(AtomicBool::new(false)),
            ui_registry: Arc::new(Self::create_registry()),
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

    fn create_registry() -> UIPropertiesRegistry {
        let mut ui_registry = UIPropertiesRegistry::default();

        ui_registry.register::<PipelineInstance>();
        ui_registry.register::<FontInstance>();
        ui_registry.register::<MaterialInstance>();
        ui_registry.register::<MeshInstance>();
        ui_registry.register::<TextureInstance>();
        ui_registry.register::<ViewInstance>();

        ui_registry.register::<UIWidget>();

        ui_registry.register::<Scene>();
        ui_registry.register::<Object>();
        ui_registry.register::<Transform>();
        ui_registry.register::<Hitbox>();
        ui_registry
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

        self.create_main_menu()
            .create_view3d()
            .create_properties()
            .create_scene();
    }

    fn run(&mut self) -> bool {
        self.update_events().update_view3d().update_widgets();

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
        let debug_info = DebugInfo::new(&self.shared_data, self.ui_registry.clone());
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
    fn create_properties(&mut self) -> &mut Self {
        let properties = Properties::new(&self.shared_data, self.ui_registry.clone());
        self.properties = Some(properties);
        self
    }
    fn update_view3d(&mut self) -> &mut Self {
        if let Some(view) = &mut self.view3d {
            view.update();
        }
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

                if let Some(view) = &mut self.view3d {
                    view.handle_keyboard_event(event);
                }
            }
        });
        self
    }
}
