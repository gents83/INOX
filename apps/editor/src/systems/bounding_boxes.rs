use std::any::TypeId;

use nrg_graphics::MeshInstance;
use nrg_math::{Mat4Ops, Vector3, Vector4, Zero};
use nrg_messenger::{read_messages, Message, MessageBox, MessageChannel, MessengerRw};
use nrg_resources::{SharedData, SharedDataRw};
use nrg_scene::{Hitbox, Object, ObjectId};

use crate::widgets::ViewEvent;

use super::DrawEvent;

pub struct BoundingBoxDrawer {
    shared_data: SharedDataRw,
    message_channel: MessageChannel,
    global_dispatcher: MessageBox,
    objects_to_draw: Vec<ObjectId>,
}

impl BoundingBoxDrawer {
    pub fn new(shared_data: &SharedDataRw, global_messenger: &MessengerRw) -> Self {
        let message_channel = MessageChannel::default();

        global_messenger
            .write()
            .unwrap()
            .register_messagebox::<ViewEvent>(message_channel.get_messagebox());
        Self {
            shared_data: shared_data.clone(),
            message_channel,
            global_dispatcher: global_messenger.write().unwrap().get_dispatcher(),
            objects_to_draw: Vec::new(),
        }
    }
    pub fn update(&mut self) {
        self.update_events();

        for object_id in self.objects_to_draw.iter() {
            if SharedData::has_resource::<Object>(&self.shared_data, *object_id) {
                let mut min = Vector3::zero();
                let mut max = Vector3::zero();
                let object = SharedData::get_resource::<Object>(&self.shared_data, *object_id);
                if let Some(hitbox) = object.resource().get().get_component::<Hitbox>() {
                    min = hitbox.resource().get().min();
                    max = hitbox.resource().get().max();
                } else if let Some(mesh) = object.resource().get().get_component::<MeshInstance>() {
                    let transform = mesh.resource().get().matrix();
                    let (mesh_min, mesh_max) = mesh.resource().get().mesh_data().compute_min_max();
                    min = transform.transform(mesh_min);
                    max = transform.transform(mesh_max);
                }
                self.draw_bounding_box(min, max);
            }
        }
    }

    fn draw_bounding_box(&self, min: Vector3, max: Vector3) {
        let color: Vector4 = [1., 1., 0., 1.].into();
        self.global_dispatcher
            .write()
            .unwrap()
            .send(
                DrawEvent::Quad(
                    [min.x, min.y].into(),
                    [max.x, max.y].into(),
                    min.z,
                    color,
                    true,
                )
                .as_boxed(),
            )
            .ok();
        self.global_dispatcher
            .write()
            .unwrap()
            .send(
                DrawEvent::Quad(
                    [min.x, min.y].into(),
                    [max.x, max.y].into(),
                    max.z,
                    color,
                    true,
                )
                .as_boxed(),
            )
            .ok();
        self.global_dispatcher
            .write()
            .unwrap()
            .send(
                DrawEvent::Line(
                    [min.x, min.y, min.z].into(),
                    [min.x, min.y, max.z].into(),
                    color,
                )
                .as_boxed(),
            )
            .ok();
        self.global_dispatcher
            .write()
            .unwrap()
            .send(
                DrawEvent::Line(
                    [min.x, max.y, min.z].into(),
                    [min.x, max.y, max.z].into(),
                    color,
                )
                .as_boxed(),
            )
            .ok();
        self.global_dispatcher
            .write()
            .unwrap()
            .send(
                DrawEvent::Line(
                    [max.x, min.y, min.z].into(),
                    [max.x, min.y, max.z].into(),
                    color,
                )
                .as_boxed(),
            )
            .ok();
        self.global_dispatcher
            .write()
            .unwrap()
            .send(
                DrawEvent::Line(
                    [max.x, max.y, min.z].into(),
                    [max.x, max.y, max.z].into(),
                    color,
                )
                .as_boxed(),
            )
            .ok();
    }

    fn update_events(&mut self) {
        nrg_profiler::scoped_profile!("update_events");

        read_messages(self.message_channel.get_listener(), |msg| {
            if msg.type_id() == TypeId::of::<ViewEvent>() {
                let event = msg.as_any().downcast_ref::<ViewEvent>().unwrap();
                let ViewEvent::Selected(object_id) = event;
                if object_id.is_nil() {
                    self.objects_to_draw.clear();
                } else {
                    self.objects_to_draw.push(*object_id);
                }
            }
        });
    }
}
