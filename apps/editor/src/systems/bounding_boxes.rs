use std::any::TypeId;

use nrg_graphics::{DrawEvent, Mesh};
use nrg_math::{Mat4Ops, Vector3, Zero};
use nrg_messenger::{read_messages, Message, MessageBox, MessageChannel, MessengerRw};
use nrg_resources::{SharedData, SharedDataRc};
use nrg_scene::{Hitbox, Object, ObjectId};

use crate::{EditMode, EditorEvent};

pub struct BoundingBoxDrawer {
    shared_data: SharedDataRc,
    message_channel: MessageChannel,
    global_dispatcher: MessageBox,
    objects_to_draw: Vec<ObjectId>,
}

impl BoundingBoxDrawer {
    pub fn new(shared_data: &SharedDataRc, global_messenger: &MessengerRw) -> Self {
        let message_channel = MessageChannel::default();

        global_messenger
            .write()
            .unwrap()
            .register_messagebox::<EditorEvent>(message_channel.get_messagebox());
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
            if let Some(object) = SharedData::get_resource::<Object>(&self.shared_data, object_id) {
                let mut min = Vector3::zero();
                let mut max = Vector3::zero();

                if let Some(hitbox) = object.get(|o| o.get_component::<Hitbox>()) {
                    min = hitbox.get(|h| h.min());
                    max = hitbox.get(|h| h.max());
                } else if let Some(mesh) = object.get(|o| o.get_component::<Mesh>()) {
                    let transform = mesh.get(|m| m.matrix());
                    let (mesh_min, mesh_max) = mesh.get(|m| m.mesh_data().compute_min_max());
                    min = transform.transform(mesh_min);
                    max = transform.transform(mesh_max);
                }
                self.global_dispatcher
                    .write()
                    .unwrap()
                    .send(DrawEvent::BoundingBox(min, max, [1., 1., 0., 1.].into()).as_boxed())
                    .ok();
            }
        }
    }
    fn update_events(&mut self) {
        nrg_profiler::scoped_profile!("update_events");

        read_messages(self.message_channel.get_listener(), |msg| {
            if msg.type_id() == TypeId::of::<EditorEvent>() {
                let event = msg.as_any().downcast_ref::<EditorEvent>().unwrap();
                match event {
                    EditorEvent::Selected(object_id) => {
                        if object_id.is_nil() {
                            self.objects_to_draw.clear();
                        } else {
                            self.objects_to_draw.push(*object_id);
                        }
                    }
                    EditorEvent::ChangeMode(mode) => {
                        if *mode == EditMode::View {
                            self.objects_to_draw.clear();
                        }
                    }
                    _ => {}
                }
            }
        });
    }
}
