use nrg_camera::Camera;
use nrg_core::System;
use nrg_graphics::{RenderPass, View};
use nrg_math::{Vector2, Vector3, Zero};
use nrg_messenger::{read_messages, send_global_event, MessageChannel, MessengerRw};
use nrg_platform::{Key, KeyEvent, MouseButton, MouseEvent, WindowEvent};
use nrg_resources::{DataTypeResource, Resource, SerializableResource, SharedData, SharedDataRc};
use nrg_scene::Scene;
use nrg_serialize::generate_random_uid;
use std::{any::TypeId, env, path::PathBuf};

use crate::config::Config;

#[derive(PartialEq, Eq)]
enum Operation {
    None,
    LoadFile,
}

pub struct ViewerSystem {
    shared_data: SharedDataRc,
    global_messenger: MessengerRw,
    config: Config,
    message_channel: MessageChannel,
    render_passes: Vec<Resource<RenderPass>>,
    scene: Resource<Scene>,
    camera: Resource<Camera>,
    last_mouse_pos: Vector2,
    is_changing_camera: bool,
}

impl ViewerSystem {
    pub fn new(shared_data: SharedDataRc, global_messenger: MessengerRw, config: &Config) -> Self {
        let message_channel = MessageChannel::default();

        nrg_camera::register_resource_types(&shared_data);
        nrg_scene::register_resource_types(&shared_data);

        let scene = SharedData::add_resource::<Scene>(
            &shared_data,
            generate_random_uid(),
            Scene::default(),
        );

        let mut camera = Camera::new([0., 10., 10.].into(), [0., 0., 0.].into());
        camera.set_projection(45., config.width as _, config.height as _, 0.001, 1000.);
        let camera =
            SharedData::add_resource::<Camera>(&shared_data, generate_random_uid(), camera);
        camera.get_mut(|c| {
            c.set_active(false);
        });

        Self {
            shared_data,
            global_messenger,
            config: config.clone(),
            message_channel,
            render_passes: Vec::new(),
            scene,
            camera,
            last_mouse_pos: Vector2::zero(),
            is_changing_camera: false,
        }
    }

    fn window_init(&mut self) -> &mut Self {
        send_global_event(
            &self.global_messenger,
            WindowEvent::RequestChangeTitle(self.config.title.clone()),
        );
        send_global_event(
            &self.global_messenger,
            WindowEvent::RequestChangeSize(self.config.width, self.config.height),
        );
        send_global_event(
            &self.global_messenger,
            WindowEvent::RequestChangePos(self.config.pos_x, self.config.pos_y),
        );
        send_global_event(
            &self.global_messenger,
            WindowEvent::RequestChangeVisible(true),
        );
        self
    }
}

impl Drop for ViewerSystem {
    fn drop(&mut self) {
        nrg_scene::unregister_resource_types(&self.shared_data);
        nrg_camera::unregister_resource_types(&self.shared_data);
    }
}

impl System for ViewerSystem {
    fn should_run_when_not_focused(&self) -> bool {
        false
    }

    fn init(&mut self) {
        self.window_init()
            .load_pipelines()
            .check_command_line_arguments();

        self.global_messenger
            .write()
            .unwrap()
            .register_messagebox::<KeyEvent>(self.message_channel.get_messagebox())
            .register_messagebox::<MouseEvent>(self.message_channel.get_messagebox())
            .register_messagebox::<WindowEvent>(self.message_channel.get_messagebox());
    }

    fn run(&mut self) -> bool {
        self.update_events().update_view_from_camera();

        self.scene.get_mut(|s| {
            s.update_hierarchy(&self.shared_data);
        });

        true
    }
    fn uninit(&mut self) {
        self.global_messenger
            .write()
            .unwrap()
            .unregister_messagebox::<KeyEvent>(self.message_channel.get_messagebox())
            .unregister_messagebox::<MouseEvent>(self.message_channel.get_messagebox())
            .unregister_messagebox::<WindowEvent>(self.message_channel.get_messagebox());
    }
}

impl ViewerSystem {
    fn load_pipelines(&mut self) -> &mut Self {
        for render_pass_data in self.config.render_passes.iter() {
            self.render_passes.push(RenderPass::create_from_data(
                &self.shared_data,
                &self.global_messenger,
                generate_random_uid(),
                render_pass_data.clone(),
            ));
        }
        self
    }

    fn check_command_line_arguments(&mut self) -> &mut Self {
        let mut next_op = Operation::None;

        let args: Vec<String> = env::args().collect();
        if args.len() > 1 {
            (1..args.len()).for_each(|i| {
                println!("{:?}", args[i].as_str());
                if args[i].starts_with("-load_file") {
                    next_op = Operation::LoadFile;
                    if let Some(argument) = args[i].strip_prefix("-load_file ") {
                        self.load_scene(argument);
                        next_op = Operation::None;
                    }
                } else if next_op == Operation::LoadFile {
                    let argument = args[i].as_str();
                    self.load_scene(argument);
                    next_op = Operation::None;
                }
            });
        }
        self
    }

    fn load_scene(&mut self, filename: &str) {
        if filename.ends_with("scene_data") {
            self.scene.get_mut(|s| {
                s.clear();
            });
            self.scene = Scene::load_from_file(
                &self.shared_data,
                &self.global_messenger,
                PathBuf::from(filename).as_path(),
            );
        }
    }

    fn update_events(&mut self) -> &mut Self {
        nrg_profiler::scoped_profile!("update_events");

        read_messages(self.message_channel.get_listener(), |msg| {
            if msg.type_id() == TypeId::of::<KeyEvent>() {
                let event = msg.as_any().downcast_ref::<KeyEvent>().unwrap();
                self.handle_keyboard_event(event);
            } else if msg.type_id() == TypeId::of::<MouseEvent>() {
                let event = msg.as_any().downcast_ref::<MouseEvent>().unwrap();
                self.handle_mouse_event(event);
            } else if msg.type_id() == TypeId::of::<WindowEvent>() {
                let event = msg.as_any().downcast_ref::<WindowEvent>().unwrap();
                match event {
                    WindowEvent::SizeChanged(width, height) => {
                        self.shared_data.for_each_resource_mut(|_, c: &mut Camera| {
                            c.set_projection(
                                c.fov_in_degrees(),
                                *width as _,
                                *height as _,
                                c.near_plane(),
                                c.far_plane(),
                            );
                        });
                    }
                    _ => {}
                }
            }
        });
        self
    }

    fn update_view_from_camera(&mut self) -> &mut Self {
        if let Some(view) = self
            .shared_data
            .match_resource(|view: &View| view.view_index() == 0)
        {
            if self.shared_data.get_num_resources_of_type::<Camera>() == 1 {
                self.camera.get_mut(|c| {
                    c.set_active(true);
                });
            } else {
                self.camera.get_mut(|c| {
                    c.set_active(false);
                });
            }

            self.shared_data.for_each_resource(|_, c: &Camera| {
                if c.is_active() {
                    let view_matrix = c.view_matrix();
                    let proj_matrix = c.proj_matrix();

                    view.get_mut(|v| {
                        v.update_view(view_matrix).update_proj(proj_matrix);
                    });
                }
            })
        }
        self
    }

    fn handle_keyboard_event(&mut self, event: &KeyEvent) {
        let mut movement = Vector3::zero();
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
        self.shared_data.for_each_resource_mut(|_, c: &mut Camera| {
            if c.is_active() {
                c.translate(movement);
            }
        });
    }

    fn handle_mouse_event(&mut self, event: &MouseEvent) {
        if event.button == MouseButton::Left {
            self.is_changing_camera = !self.is_changing_camera;
        }
        if self.is_changing_camera {
            let mut rotation_angle = Vector3::zero();

            rotation_angle.x = event.normalized_y - self.last_mouse_pos.y;
            rotation_angle.y = event.normalized_x - self.last_mouse_pos.x;
            self.shared_data.for_each_resource_mut(|_, c: &mut Camera| {
                if c.is_active() {
                    c.rotate(rotation_angle * 5.);
                }
            });
        }
        self.last_mouse_pos = Vector2::new(event.normalized_x as _, event.normalized_y as _);
    }
}
