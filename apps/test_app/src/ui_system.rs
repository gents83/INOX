use std::any::TypeId;

use nrg_core::{JobHandlerRw, System, SystemId};
use nrg_graphics::{MaterialInstance, MaterialRc, MeshData, MeshInstance, PipelineInstance};
use nrg_math::{get_random_f32, Mat4Ops, MatBase, Matrix4, Vector2};

use nrg_messenger::{read_messages, MessageChannel, MessengerRw};
use nrg_platform::{MouseButton, MouseEvent, MouseState};
use nrg_resources::{DataTypeResource, SharedData, SharedDataRw};
use nrg_scene::{Hitbox, Object, Scene, SceneRc, Transform};

pub struct UISystem {
    id: SystemId,
    shared_data: SharedDataRw,
    job_handler: JobHandlerRw,
    global_messenger: MessengerRw,
    message_channel: MessageChannel,
    ui_scene: SceneRc,
    ui_default_material: MaterialRc,
}

impl UISystem {
    pub fn new(
        shared_data: SharedDataRw,
        global_messenger: MessengerRw,
        job_handler: JobHandlerRw,
    ) -> Self {
        let message_channel = MessageChannel::default();
        Self {
            id: SystemId::new(),
            shared_data,
            job_handler,
            global_messenger,
            message_channel,
            ui_scene: SceneRc::default(),
            ui_default_material: MaterialRc::default(),
        }
    }

    fn create_scene(&mut self) -> &mut Self {
        self.ui_scene = SharedData::add_resource::<Scene>(&self.shared_data, Scene::default());
        self
    }

    fn create_default_material(&mut self) -> &mut Self {
        if let Some(pipeline) =
            SharedData::match_resource(&self.shared_data, |p: &PipelineInstance| {
                p.data().name == "UI"
            })
        {
            self.ui_default_material =
                MaterialInstance::create_from_pipeline(&self.shared_data, pipeline);
        } else {
            panic!("No pipeline with name UI has been loaded");
        }
        self
    }

    fn add_2d_quad(&mut self, x: f32, y: f32, size_x: f32, size_y: f32) -> &mut Self {
        let object = Object::generate_empty(&self.shared_data);

        let transform = object
            .resource()
            .get_mut()
            .add_default_component::<Transform>(&self.shared_data);
        let mat = Matrix4::from_translation([x, y, 0.].into())
            * Matrix4::from_nonuniform_scale(size_x, size_y, 1.);
        transform.resource().get_mut().set_matrix(mat);

        let hitbox = object
            .resource()
            .get_mut()
            .add_default_component::<Hitbox>(&self.shared_data);
        hitbox
            .resource()
            .get_mut()
            .set_dimensions([0., 0., 0.].into(), [size_x, size_y, 0.].into());

        {
            let mut mesh_data = MeshData::default();
            mesh_data.add_quad_default([0., 0., 1., 1.].into(), 0.);
            mesh_data.set_vertex_color(
                [
                    get_random_f32(0., 1.),
                    get_random_f32(0., 1.),
                    get_random_f32(0., 1.),
                    get_random_f32(0.5, 1.),
                ]
                .into(),
            );
            let mesh = MeshInstance::create_from_data(&self.shared_data, mesh_data);
            self.ui_default_material
                .resource()
                .get_mut()
                .add_mesh(mesh.clone());

            object
                .resource()
                .get_mut()
                .add_component(mesh)
                .add_component(self.ui_default_material.clone());
        }

        self.ui_scene.resource().get_mut().add_object(object);

        self
    }

    fn update_scene(&mut self) -> &mut Self {
        let objects = self.ui_scene.resource().get().get_objects();
        for (i, obj) in objects.iter().enumerate() {
            let job_name = format!("Object[{}]", i);
            let obj = obj.clone();
            let shared_data = self.shared_data.clone();
            self.job_handler
                .write()
                .unwrap()
                .add_job(job_name.as_str(), move || {
                    obj.resource().get_mut().update_from_parent(
                        &shared_data,
                        Matrix4::default_identity(),
                        |object, object_matrix| {
                            if let Some(mesh) = object.get_component::<MeshInstance>() {
                                mesh.resource().get_mut().set_transform(object_matrix);
                            }
                            if let Some(hitbox) = object.get_component::<Hitbox>() {
                                let offset = object_matrix.get_translation();
                                hitbox
                                    .resource()
                                    .get_mut()
                                    .set_transform(Matrix4::from_translation(offset));
                            }
                        },
                    );
                });
        }

        self
    }

    fn check_interactions(&mut self, mouse_pos: Vector2) -> &mut Self {
        let hitboxes = SharedData::get_resources_of_type::<Hitbox>(&self.shared_data);
        for (i, hitbox) in hitboxes.iter().enumerate() {
            let min = hitbox.resource().get().min();
            let max = hitbox.resource().get().max();
            if mouse_pos.x >= min.x
                && mouse_pos.x <= max.x
                && mouse_pos.y >= min.y
                && mouse_pos.y <= max.y
            {
                println!(
                    "{:?} is in Hitbox[{}] = |{:?} - {:?}|",
                    mouse_pos, i, min, max
                );
            }
        }
        self
    }

    fn update_events(&mut self) -> &mut Self {
        read_messages(self.message_channel.get_listener(), |msg| {
            if msg.type_id() == TypeId::of::<MouseEvent>() {
                let event = msg.as_any().downcast_ref::<MouseEvent>().unwrap();
                if event.state == MouseState::Up && event.button == MouseButton::Left {
                    let mouse_pos = [event.x as f32, event.y as f32].into();
                    self.check_interactions(mouse_pos);
                }
            }
        });
        self
    }
}

impl System for UISystem {
    fn id(&self) -> nrg_core::SystemId {
        self.id
    }

    fn should_run_when_not_focused(&self) -> bool {
        false
    }
    fn init(&mut self) {
        self.global_messenger
            .write()
            .unwrap()
            .register_messagebox::<MouseEvent>(self.message_channel.get_messagebox());

        self.create_scene().create_default_material();

        for i in 0..15 {
            for j in 0..10 {
                self.add_2d_quad(i as f32 * 150., j as f32 * 150., 100., 100.);
            }
        }
    }

    fn run(&mut self) -> bool {
        self.update_scene().update_events();
        true
    }

    fn uninit(&mut self) {
        self.global_messenger
            .write()
            .unwrap()
            .unregister_messagebox::<MouseEvent>(self.message_channel.get_messagebox());
    }
}
