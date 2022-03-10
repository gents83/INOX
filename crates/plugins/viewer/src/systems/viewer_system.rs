use inox_commands::CommandParser;
use inox_core::{ContextRc, System};
use inox_graphics::{DrawEvent, Light, Material, Mesh, MeshData, Pipeline, Texture, View};
use inox_math::{Matrix4, VecBase, Vector2, Vector3};
use inox_messenger::Listener;

use inox_platform::{InputState, Key, KeyEvent, MouseEvent, WindowEvent};
use inox_resources::{DataTypeResource, Resource, ResourceEvent, SerializableResource};
use inox_scene::{Camera, Object, ObjectId, Scene, Script};
use inox_uid::generate_random_uid;
use std::{collections::HashMap, path::PathBuf};

use crate::widgets::{Hierarchy, Info, View3D};

pub struct ViewerSystem {
    context: ContextRc,
    listener: Listener,
    scene: Resource<Scene>,
    camera_object: Resource<Object>,
    last_mouse_pos: Vector2,
    _view_3d: Option<View3D>,
    _info: Option<Info>,
    _hierarchy: Option<Hierarchy>,
}

const FORCE_USE_DEFAULT_CAMERA: bool = false;

impl ViewerSystem {
    pub fn new(context: &ContextRc) -> Self {
        let listener = Listener::new(context.message_hub());
        let shared_data = context.shared_data();
        let message_hub = context.message_hub();

        inox_scene::register_resource_types(shared_data, message_hub);

        let scene_id = generate_random_uid();
        let scene = shared_data.add_resource::<Scene>(
            message_hub,
            scene_id,
            Scene::new(scene_id, shared_data, message_hub),
        );

        let camera_id = generate_random_uid();
        let camera_object = shared_data.add_resource::<Object>(
            message_hub,
            camera_id,
            Object::new(camera_id, shared_data, message_hub),
        );
        camera_object
            .get_mut()
            .set_position(Vector3::new(0.0, 0.0, -50.0));
        camera_object.get_mut().look_at(Vector3::new(0.0, 0.0, 0.0));
        let camera = camera_object
            .get_mut()
            .add_default_component::<Camera>(shared_data, message_hub);
        camera
            .get_mut()
            .set_parent(&camera_object)
            .set_active(false);

        Self {
            _view_3d: None,
            _info: None,
            _hierarchy: None,
            context: context.clone(),
            listener,
            scene,
            camera_object,
            last_mouse_pos: Vector2::default_zero(),
        }
    }
}

impl Drop for ViewerSystem {
    fn drop(&mut self) {
        inox_scene::unregister_resource_types(self.context.shared_data());
    }
}

impl System for ViewerSystem {
    fn read_config(&mut self, _plugin_name: &str) {}
    fn should_run_when_not_focused(&self) -> bool {
        false
    }

    fn init(&mut self) {
        self.check_command_line_arguments();

        self.listener
            .register::<KeyEvent>()
            .register::<MouseEvent>()
            .register::<WindowEvent>()
            .register::<ResourceEvent<Scene>>();

        self._view_3d = Some(View3D::new(
            self.context.shared_data(),
            self.context.message_hub(),
        ));
    }

    fn run(&mut self) -> bool {
        inox_profiler::scoped_profile!("viewer_system::run");
        self.update_events().update_view_from_camera();

        self.context
            .shared_data()
            .for_each_resource_mut(|_, s: &mut Script| {
                s.update();
            });

        let mut map: HashMap<ObjectId, Option<Matrix4>> = HashMap::new();
        self.context
            .shared_data()
            .for_each_resource(|r: &Resource<Object>, o: &Object| {
                let parent_transform = o.parent().map(|parent| parent.get().transform());
                map.insert(*r.id(), parent_transform);
            });
        self.context
            .shared_data()
            .for_each_resource_mut(|r, o: &mut Object| {
                if let Some(parent_transform) = map.remove(r.id()) {
                    o.update_transform(parent_transform);
                }
                if let Some(light) = o.component::<Light>() {
                    light.get_mut().set_position(o.get_position());
                }
            });

        self.context
            .shared_data()
            .for_each_resource(|_, l: &Light| {
                if l.is_active() {
                    self.context.message_hub().send_event(DrawEvent::Sphere(
                        l.data().position.into(),
                        l.data().range,
                        [l.data().color[0], l.data().color[1], l.data().color[2], 1.].into(),
                        true,
                    ));
                }
            });

        true
    }
    fn uninit(&mut self) {
        self.listener
            .unregister::<KeyEvent>()
            .unregister::<MouseEvent>()
            .unregister::<WindowEvent>()
            .unregister::<ResourceEvent<Scene>>();
    }
}

impl ViewerSystem {
    fn check_command_line_arguments(&mut self) -> &mut Self {
        let command_parser = CommandParser::from_command_line();
        if command_parser.has("load_file") {
            let values = command_parser.get_values_of::<String>("load_file");
            self.load_scene(values[0].as_str());
        } else {
            self.create_default_scene();
        }

        self
    }

    fn create_default_scene(&mut self) {
        let default_object = {
            let object_id = generate_random_uid();
            let object = self.context.shared_data().add_resource(
                self.context.message_hub(),
                object_id,
                Object::new(
                    object_id,
                    self.context.shared_data(),
                    self.context.message_hub(),
                ),
            );
            let mesh_id = generate_random_uid();
            let mesh = self.context.shared_data().add_resource(
                self.context.message_hub(),
                mesh_id,
                Mesh::new(
                    mesh_id,
                    self.context.shared_data(),
                    self.context.message_hub(),
                ),
            );
            let pipeline = Pipeline::request_load(
                self.context.shared_data(),
                self.context.message_hub(),
                PathBuf::from("pipelines\\Default.pipeline").as_path(),
                None,
            );
            let material = Material::duplicate_from_pipeline(
                self.context.shared_data(),
                self.context.message_hub(),
                &pipeline,
            );
            let texture = Texture::request_load(
                self.context.shared_data(),
                self.context.message_hub(),
                PathBuf::from("textures\\Test.png").as_path(),
                None,
            );
            material
                .get_mut()
                .set_texture(inox_graphics::TextureType::BaseColor, &texture);
            mesh.get_mut().set_material(material);
            let mut mesh_data = MeshData::default();
            mesh_data.add_quad_default([-10., -10., 10., 10.].into(), 0.);

            //println!("Quad Mesh {:?}", mesh.id());

            mesh.get_mut().set_mesh_data(mesh_data);
            object.get_mut().add_component(mesh);
            object.get_mut().set_position([-20., 0., 0.].into());
            object
        };
        let wireframe_object = {
            let object_id = generate_random_uid();
            let object = self.context.shared_data().add_resource(
                self.context.message_hub(),
                object_id,
                Object::new(
                    object_id,
                    self.context.shared_data(),
                    self.context.message_hub(),
                ),
            );
            let mesh_id = generate_random_uid();
            let mesh = self.context.shared_data().add_resource(
                self.context.message_hub(),
                mesh_id,
                Mesh::new(
                    mesh_id,
                    self.context.shared_data(),
                    self.context.message_hub(),
                ),
            );
            let pipeline = Pipeline::request_load(
                self.context.shared_data(),
                self.context.message_hub(),
                PathBuf::from("pipelines\\Wireframe.pipeline").as_path(),
                None,
            );
            let material = Material::duplicate_from_pipeline(
                self.context.shared_data(),
                self.context.message_hub(),
                &pipeline,
            );
            mesh.get_mut().set_material(material);
            let mut mesh_data = MeshData::default();
            mesh_data.add_quad_default([-10., -10., 10., 10.].into(), 0.);

            //println!("Wireframe Mesh {:?}", mesh.id());

            mesh.get_mut().set_mesh_data(mesh_data);
            object.get_mut().add_component(mesh);
            object.get_mut().set_position([20., 0., 0.].into());
            object
        };
        self.scene.get_mut().add_object(default_object);
        self.scene.get_mut().add_object(wireframe_object);
    }

    fn load_scene(&mut self, filename: &str) {
        if filename.ends_with(Scene::extension()) {
            self.scene.get_mut().clear();
            self.scene = Scene::request_load(
                self.context.shared_data(),
                self.context.message_hub(),
                PathBuf::from(filename).as_path(),
                None,
            );
        }
    }

    fn update_events(&mut self) -> &mut Self {
        inox_profiler::scoped_profile!("update_events");

        self.handle_keyboard_event();
        self.handle_mouse_event();
        self.listener
            .process_messages(|event: &WindowEvent| {
                if let WindowEvent::SizeChanged(width, height) = event {
                    self.context
                        .shared_data()
                        .for_each_resource_mut(|_, c: &mut Camera| {
                            c.set_projection(
                                c.fov_in_degrees(),
                                *width as _,
                                *height as _,
                                c.near_plane(),
                                c.far_plane(),
                            );
                        });
                }
            })
            .process_messages(|event: &ResourceEvent<Scene>| {
                if let ResourceEvent::<Scene>::Load(path, _option) = event {
                    if let Some(scene_path) = path.to_str() {
                        if scene_path.ends_with(Scene::extension()) {
                            self.scene.get_mut().clear();
                            self.scene = Scene::request_load(
                                self.context.shared_data(),
                                self.context.message_hub(),
                                PathBuf::from(scene_path).as_path(),
                                None,
                            );
                        }
                    }
                }
            });
        self
    }

    fn update_view_from_camera(&mut self) -> &mut Self {
        inox_profiler::scoped_profile!("update_view_from_camera");

        if let Some(view) = self
            .context
            .shared_data()
            .match_resource(|view: &View| view.view_index() == 0)
        {
            if self
                .context
                .shared_data()
                .get_num_resources_of_type::<Camera>()
                == 1
            {
                if let Some(camera) = self.camera_object.get().component::<Camera>() {
                    camera.get_mut().set_active(true);
                }
            } else if let Some(camera) = self.camera_object.get().component::<Camera>() {
                camera.get_mut().set_active(false);
            }

            if FORCE_USE_DEFAULT_CAMERA {
                self.context
                    .shared_data()
                    .for_each_resource_mut(|_, c: &mut Camera| {
                        if let Some(parent) = c.parent() {
                            if parent.id() == self.camera_object.id() {
                                c.set_active(true);
                            } else {
                                c.set_active(false);
                            }
                        }
                    });
            }

            self.context
                .shared_data()
                .for_each_resource(|_, c: &Camera| {
                    if c.is_active() {
                        let view_matrix = c.view_matrix();
                        let proj_matrix = c.proj_matrix();

                        view.get_mut()
                            .update_view(view_matrix)
                            .update_proj(proj_matrix);
                    }
                })
        }
        self
    }

    fn handle_keyboard_event(&mut self) {
        self.listener.process_messages(|event: &KeyEvent| {
            if event.code == Key::F1 && event.state == InputState::Released {
                if self._info.is_some() {
                    self._info = None;
                } else {
                    self._info = Some(Info::new(&self.context));
                }
            } else if event.code == Key::F2 && event.state == InputState::Released {
                if self._hierarchy.is_some() {
                    self._hierarchy = None;
                } else {
                    self._hierarchy = Some(Hierarchy::new(
                        self.context.shared_data(),
                        self.context.message_hub(),
                        self.scene.id(),
                    ));
                }
            }

            let mut movement = Vector3::default_zero();
            if event.code == Key::W {
                movement.z += 1.;
            } else if event.code == Key::S {
                movement.z -= 1.;
            } else if event.code == Key::A {
                movement.x -= 1.;
            } else if event.code == Key::D {
                movement.x += 1.;
            } else if event.code == Key::Q {
                movement.y += 1.;
            } else if event.code == Key::E {
                movement.y -= 1.;
            }
            if movement != Vector3::default_zero() {
                self.context
                    .shared_data()
                    .for_each_resource_mut(|_, c: &mut Camera| {
                        if c.is_active() {
                            let matrix = c.transform();
                            let translation = matrix.x.xyz().normalized() * movement.x
                                + matrix.y.xyz().normalized() * movement.y
                                + matrix.z.xyz().normalized() * movement.z;
                            c.translate(translation);
                        }
                    });
            }
        });
    }

    fn handle_mouse_event(&mut self) {
        self.listener.process_messages(|event: &MouseEvent| {
            let mut is_on_view3d = false;
            if let Some(view_3d) = &self._view_3d {
                is_on_view3d = view_3d.is_interacting();
            }
            if is_on_view3d {
                let mut rotation_angle = Vector3::default_zero();

                rotation_angle.x = event.normalized_y - self.last_mouse_pos.y;
                rotation_angle.y = event.normalized_x - self.last_mouse_pos.x;
                if rotation_angle != Vector3::default_zero() {
                    self.context
                        .shared_data()
                        .for_each_resource_mut(|_, c: &mut Camera| {
                            if c.is_active() {
                                c.rotate(rotation_angle * 5.);
                            }
                        });
                }
            }
            self.last_mouse_pos = Vector2::new(event.normalized_x as _, event.normalized_y as _);
        });
    }
}
