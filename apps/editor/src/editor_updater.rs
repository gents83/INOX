use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{any::TypeId, path::Path};

use crate::EditorEvent;

use super::config::*;
use super::widgets::*;

use nrg_core::*;
use nrg_graphics::{Font, Material, Mesh, MeshData, Pipeline, RenderPass, Texture, View};
use nrg_messenger::{read_messages, Message, MessageChannel, MessengerRw};
use nrg_platform::{InputState, Key, KeyEvent, MouseEvent, WindowEvent};
use nrg_resources::{
    DataTypeResource, FileResource, Resource, ResourceData, SerializableResource, SharedData,
    SharedDataRw,
};
use nrg_scene::{Hitbox, Object, Scene, Transform};
use nrg_ui::{DialogEvent, DialogOp, UIPropertiesRegistry, UIWidget};

#[allow(dead_code)]
pub struct EditorUpdater {
    id: SystemId,
    shared_data: SharedDataRw,
    global_messenger: MessengerRw,
    config: Config,
    message_channel: MessageChannel,
    pipelines: Vec<Resource<Pipeline>>,
    render_passes: Vec<Resource<RenderPass>>,
    fonts: Vec<Resource<Font>>,
    default_material: Resource<Material>,
    wireframe_material: Resource<Material>,
    grid_material: Resource<Material>,
    grid_mesh: Resource<Mesh>,
    scene: Resource<Scene>,
    main_menu: Option<MainMenu>,
    toolbar: Option<Toolbar>,
    debug_info: Option<DebugInfo>,
    view3d: Option<View3D>,
    properties: Option<Properties>,
    hierarchy: Option<Hierarchy>,
    content_browser: Option<ContentBrowser>,
    show_debug_info: Arc<AtomicBool>,
    ui_registry: Arc<UIPropertiesRegistry>,
}

impl EditorUpdater {
    pub fn new(shared_data: SharedDataRw, global_messenger: MessengerRw, config: &Config) -> Self {
        let message_channel = MessageChannel::default();

        nrg_scene::register_resource_types(&shared_data);
        crate::resources::register_resource_types(&shared_data);

        let default_material =
            Material::create_from_file(&shared_data, config.default_material.as_path());
        let wireframe_material =
            Material::create_from_file(&shared_data, config.wireframe_material.as_path());
        let grid_material =
            Material::create_from_file(&shared_data, config.grid_material.as_path());

        let mut mesh_data = MeshData::default();
        mesh_data.add_quad_default([-1., -1., 1., 1.].into(), 0.);
        let grid_mesh = Mesh::create_from_data(&shared_data, mesh_data);
        grid_mesh.get_mut().set_material(grid_material.clone());

        let scene = SharedData::add_resource::<Scene>(&shared_data, Scene::default());

        Self {
            id: SystemId::new(),
            pipelines: Vec::new(),
            render_passes: Vec::new(),
            fonts: Vec::new(),
            shared_data,
            global_messenger,
            config: config.clone(),
            message_channel,
            default_material,
            wireframe_material,
            grid_material,
            grid_mesh,
            scene,
            main_menu: None,
            toolbar: None,
            debug_info: None,
            view3d: None,
            properties: None,
            hierarchy: None,
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

        ui_registry.register::<Pipeline>();
        ui_registry.register::<Font>();
        ui_registry.register::<Material>();
        ui_registry.register::<Mesh>();
        ui_registry.register::<Texture>();
        ui_registry.register::<View>();

        ui_registry.register::<UIWidget>();

        ui_registry.register::<Scene>();
        ui_registry.register::<Object>();
        ui_registry.register::<Transform>();
        ui_registry.register::<Hitbox>();
        ui_registry
    }
}

impl Drop for EditorUpdater {
    fn drop(&mut self) {
        crate::resources::unregister_resource_types(&self.shared_data);
        nrg_scene::unregister_resource_types(&self.shared_data);
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
            .register_messagebox::<EditorEvent>(self.message_channel.get_messagebox())
            .register_messagebox::<DialogEvent>(self.message_channel.get_messagebox());

        self.create_main_menu()
            .create_toolbar()
            .create_hierarchy()
            .create_properties()
            .create_view3d();
    }

    fn run(&mut self) -> bool {
        self.update_events().update_view3d().update_widgets();

        self.scene.get_mut().update_hierarchy(&self.shared_data);

        true
    }
    fn uninit(&mut self) {
        self.global_messenger
            .write()
            .unwrap()
            .unregister_messagebox::<KeyEvent>(self.message_channel.get_messagebox())
            .unregister_messagebox::<MouseEvent>(self.message_channel.get_messagebox())
            .unregister_messagebox::<WindowEvent>(self.message_channel.get_messagebox())
            .unregister_messagebox::<EditorEvent>(self.message_channel.get_messagebox())
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
    fn create_toolbar(&mut self) -> &mut Self {
        let toolbar = Toolbar::new(&self.shared_data, &self.global_messenger);
        self.toolbar = Some(toolbar);
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
        let view3d = View3D::new(
            &self.shared_data,
            &self.global_messenger,
            &self.default_material,
            &self.wireframe_material,
        );
        self.view3d = Some(view3d);
        self
    }
    fn create_hierarchy(&mut self) -> &mut Self {
        let hierarchy = Hierarchy::new(&self.shared_data, &self.global_messenger, self.scene.id());
        self.hierarchy = Some(hierarchy);
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
    fn create_content_browser(
        &mut self,
        operation: DialogOp,
        path: &Path,
        extension: String,
    ) -> &mut Self {
        if self.content_browser.is_none() {
            let content_browser = ContentBrowser::new(
                &self.shared_data,
                &self.global_messenger,
                operation,
                path,
                extension,
            );
            self.content_browser = Some(content_browser);
        }
        self
    }
    fn destroy_content_browser(&mut self) -> &mut Self {
        self.content_browser = None;
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
            self.render_passes.push(RenderPass::create_from_data(
                &self.shared_data,
                render_pass_data.clone(),
            ));
        }

        for pipeline_data in self.config.pipelines.iter() {
            self.pipelines.push(Pipeline::create_from_data(
                &self.shared_data,
                pipeline_data.clone(),
            ));
        }

        if let Some(default_font_path) = self.config.fonts.first() {
            self.fonts
                .push(Font::create_from_file(&self.shared_data, default_font_path));
        }
    }

    fn load_object(&mut self, filename: &Path) {
        if !filename.is_dir() && filename.exists() {
            self.scene.get_mut().clear();
            let object = Object::create_from_file(&self.shared_data, filename);
            self.scene.get_mut().add_object(object);
        }
    }

    fn update_events(&mut self) -> &mut Self {
        nrg_profiler::scoped_profile!("update_events");

        read_messages(self.message_channel.get_listener(), |msg| {
            if msg.type_id() == TypeId::of::<DialogEvent>() {
                let event = msg.as_any().downcast_ref::<DialogEvent>().unwrap();
                match event {
                    DialogEvent::Request(operation, path) => {
                        self.create_content_browser(
                            *operation,
                            path.as_path(),
                            "object_data".to_string(),
                        );
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
            } else if msg.type_id() == TypeId::of::<EditorEvent>() {
                let event = msg.as_any().downcast_ref::<EditorEvent>().unwrap();
                match *event {
                    EditorEvent::Selected(object_id) => {
                        if let Some(properties) = &mut self.properties {
                            properties.select_object(object_id);
                        }
                        if let Some(hierarchy) = &mut self.hierarchy {
                            hierarchy.select_object(object_id);
                        }
                        if let Some(view3d) = &mut self.view3d {
                            view3d.select_object(object_id);
                        }
                    }
                    EditorEvent::ChangeMode(mode) => {
                        if let Some(view3d) = &mut self.view3d {
                            view3d.change_edit_mode(
                                mode,
                                self.default_material.get().pipeline().as_ref().unwrap(),
                            );
                        }
                    }
                }
            }
        });
        self
    }
}
