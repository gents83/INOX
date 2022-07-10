use inox_commands::CommandParser;
use inox_core::{implement_unique_system_uid, ContextRc, System};
use inox_graphics::{
    create_quad, CullingPass, Material, MaterialData, Mesh, MeshData, MeshFlags, RendererRw,
    Texture, View,
};
use inox_log::debug_log;
use inox_math::{Mat4Ops, Matrix4, VecBase, Vector2, Vector3};
use inox_messenger::Listener;
use inox_platform::{InputState, Key, KeyEvent, MouseEvent, MouseState, WindowEvent};
use inox_resources::{DataTypeResource, Resource, SerializableResource, SerializableResourceEvent};
use inox_scene::{Camera, Object, Scene};
use inox_ui::UIWidget;
use inox_uid::generate_random_uid;
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crate::widgets::{Info, InfoParams, View3D};

pub struct ViewerSystem {
    context: ContextRc,
    renderer: RendererRw,
    listener: Listener,
    scene: Resource<Scene>,
    _camera_object: Resource<Object>,
    last_mouse_pos: Vector2,
    is_on_view3d: bool,
    view_3d: Option<View3D>,
    info: Option<Info>,
    last_frame: u64,
    camera_index: u32,
    update_culling_camera: Arc<AtomicBool>,
}

const FORCE_USE_DEFAULT_CAMERA: bool = false;
const CAMERA_SPEED: f32 = 50.;
const CAMERA_ROTATION_SPEED: f32 = 100.;

impl Drop for ViewerSystem {
    fn drop(&mut self) {
        inox_scene::unregister_resource_types(
            self.context.shared_data(),
            self.context.message_hub(),
        );
    }
}

implement_unique_system_uid!(ViewerSystem);

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
            .register::<SerializableResourceEvent<Scene>>();
    }

    fn run(&mut self) -> bool {
        inox_profiler::scoped_profile!("viewer_system::run");

        self.update_events().update_view_from_camera();

        if let Some(info) = &mut self.info {
            info.update();
        }

        let timer = self.context.global_timer();
        let current_frame = timer.current_frame();
        debug_assert!(self.last_frame != current_frame);
        self.last_frame = current_frame;
        /*
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
        */

        true
    }
    fn uninit(&mut self) {
        self.listener
            .unregister::<KeyEvent>()
            .unregister::<MouseEvent>()
            .unregister::<WindowEvent>()
            .unregister::<SerializableResourceEvent<Scene>>();
    }
}

impl ViewerSystem {
    pub fn new(context: &ContextRc, renderer: &RendererRw, use_3dview: bool) -> Self {
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

        let update_culling_camera = Arc::new(AtomicBool::new(true));
        let view_3d = if use_3dview {
            Some(View3D::new(shared_data, message_hub))
        } else {
            None
        };
        let info = Some(Info::new(
            context,
            InfoParams {
                is_active: true,
                scene_id: *scene.id(),
                renderer: renderer.clone(),
                update_culling_camera: update_culling_camera.clone(),
            },
        ));
        Self {
            renderer: renderer.clone(),
            last_frame: u64::MAX,
            is_on_view3d: false,
            view_3d,
            info,
            context: context.clone(),
            listener,
            scene,
            _camera_object: camera_object,
            camera_index: 0,
            update_culling_camera,
            last_mouse_pos: Vector2::default_zero(),
        }
    }

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
            let material = Material::new_resource(
                self.context.shared_data(),
                self.context.message_hub(),
                generate_random_uid(),
                MaterialData::default(),
                None,
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
            mesh.get_mut()
                .set_material(material)
                .set_flags(MeshFlags::Visible | MeshFlags::Opaque);

            let mut mesh_data = MeshData::default();
            let quad = create_quad([-10., -10., 10., 10.].into(), 0.);
            mesh_data.append_mesh_data_as_meshlet(quad);
            mesh_data.set_vertex_color([0.0, 0.0, 1.0, 1.0].into());

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

            let wireframe_mesh = self.context.shared_data().add_resource(
                self.context.message_hub(),
                mesh_id,
                Mesh::new(
                    mesh_id,
                    self.context.shared_data(),
                    self.context.message_hub(),
                ),
            );
            let wireframe_material = Material::new_resource(
                self.context.shared_data(),
                self.context.message_hub(),
                generate_random_uid(),
                MaterialData::default(),
                None,
            );
            wireframe_mesh
                .get_mut()
                .set_material(wireframe_material)
                .set_flags(MeshFlags::Visible | MeshFlags::Wireframe);

            let mut mesh_data = MeshData::default();
            let quad = create_quad([-10., -10., 10., 10.].into(), 0.);
            mesh_data.append_mesh_data_as_meshlet(quad);
            mesh_data.set_vertex_color([1.0, 1.0, 0.0, 1.0].into());

            //println!("Wireframe Mesh {:?}", mesh.id());

            wireframe_mesh.get_mut().set_mesh_data(mesh_data);
            object.get_mut().add_component(wireframe_mesh);
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
            if let Some(info) = &mut self.info {
                info.set_scene_id(self.scene.id());
            }
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
            .process_messages(|event: &SerializableResourceEvent<Scene>| {
                let SerializableResourceEvent::<Scene>::Load(path, _option) = event;
                debug_log!("Loading scene: {:?}", path);
                if let Some(scene_path) = path.to_str() {
                    if scene_path.ends_with(Scene::extension()) {
                        self.scene.get_mut().clear();
                        self.scene = Scene::request_load(
                            self.context.shared_data(),
                            self.context.message_hub(),
                            PathBuf::from(scene_path).as_path(),
                            None,
                        );

                        if let Some(info) = &mut self.info {
                            info.set_scene_id(self.scene.id());
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
            if FORCE_USE_DEFAULT_CAMERA || self.context.shared_data().num_resources::<Camera>() == 1
            {
                self.camera_index = 0;
            } else {
                self.camera_index = 1;
            }
            let mut index = 0;
            self.context
                .shared_data()
                .for_each_resource_mut(|_, c: &mut Camera| {
                    c.set_active(false);
                    if self.camera_index == index {
                        c.set_active(true);

                        let view_matrix = c.view_matrix();
                        let proj_matrix = c.proj_matrix();

                        if self.update_culling_camera.load(Ordering::SeqCst) {
                            if let Some(culling_pass) =
                                self.renderer.write().unwrap().pass_mut::<CullingPass>()
                            {
                                culling_pass
                                    .set_camera_data(c.position(), proj_matrix * view_matrix);
                            }
                        }

                        view.get_mut()
                            .update_view(view_matrix)
                            .update_proj(proj_matrix);
                    }
                    index += 1;
                });
        }
        self
    }

    fn handle_keyboard_event(&mut self) {
        self.listener.process_messages(|event: &KeyEvent| {
            if event.code == Key::F1 && event.state == InputState::Released {
                if let Some(info) = &mut self.info {
                    if info.is_active() {
                        info.set_active(false);
                    } else {
                        info.set_active(true);
                    }
                }
            }

            let mut movement = Vector3::default_zero();
            if event.code == Key::W {
                movement.z += CAMERA_SPEED;
            } else if event.code == Key::S {
                movement.z -= CAMERA_SPEED;
            } else if event.code == Key::A {
                movement.x -= CAMERA_SPEED;
            } else if event.code == Key::D {
                movement.x += CAMERA_SPEED;
            } else if event.code == Key::Q {
                movement.y += CAMERA_SPEED;
            } else if event.code == Key::E {
                movement.y -= CAMERA_SPEED;
            }
            movement *= self.context.global_timer().dt().as_secs_f32();
            if movement != Vector3::default_zero() {
                self.context
                    .shared_data()
                    .for_each_resource_mut(|_, c: &mut Camera| {
                        if c.is_active() {
                            let matrix = c.transform();
                            let translation = matrix.right() * movement.x
                                + matrix.up() * movement.y
                                + matrix.forward() * movement.z;
                            c.translate(translation);
                        }
                    });
            }
        });
    }

    fn handle_mouse_event(&mut self) {
        self.listener.process_messages(|event: &MouseEvent| {
            if let Some(view_3d) = &self.view_3d {
                self.is_on_view3d = view_3d.is_interacting();
            } else if let MouseState::Down = event.state {
                self.is_on_view3d = true;
            } else if let MouseState::Up = event.state {
                self.is_on_view3d = false;
            } else {
                self.context
                    .shared_data()
                    .for_each_resource(|_, w: &UIWidget| {
                        if w.is_interacting() {
                            self.is_on_view3d = false;
                        }
                    });
            }
            if self.is_on_view3d {
                let mut rotation_angle = Vector3::default_zero();

                rotation_angle.x = self.last_mouse_pos.y - event.normalized_y;
                rotation_angle.y = self.last_mouse_pos.x - event.normalized_x;
                rotation_angle *=
                    CAMERA_ROTATION_SPEED * self.context.global_timer().dt().as_secs_f32();
                if rotation_angle != Vector3::default_zero() {
                    self.context
                        .shared_data()
                        .for_each_resource_mut(|_, c: &mut Camera| {
                            if c.is_active() {
                                let d = c.transform().direction();
                                rotation_angle.x *= d.dot_product([0., 0., -1.].into()).signum();
                                let m = Matrix4::from_euler_angles(rotation_angle);
                                let v = m.transform(d);
                                c.look_toward(v);
                            }
                        });
                }
            }
            self.last_mouse_pos = Vector2::new(event.normalized_x as _, event.normalized_y as _);
        });
    }
}
