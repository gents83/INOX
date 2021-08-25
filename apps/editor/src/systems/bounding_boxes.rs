use std::any::TypeId;

use nrg_graphics::{
    create_cube_from_min_max, MaterialInstance, MeshData, MeshInstance, MeshRc, PipelineInstance,
};
use nrg_math::{Mat4Ops, Vector3, Zero};
use nrg_messenger::{read_messages, MessageChannel, MessengerRw};
use nrg_resources::{DataTypeResource, SharedData, SharedDataRw};
use nrg_scene::{Hitbox, Object, ObjectId};

use crate::widgets::ViewEvent;

pub struct BoundingBoxDrawer {
    mesh_instance: MeshRc,
    shared_data: SharedDataRw,
    message_channel: MessageChannel,
    objects_to_draw: Vec<ObjectId>,
}

impl BoundingBoxDrawer {
    pub fn new(
        shared_data: &SharedDataRw,
        global_messenger: &MessengerRw,
        wireframe_pipeline_name: &str,
    ) -> Self {
        let mesh_instance = MeshInstance::create_from_data(shared_data, MeshData::default());
        if let Some(pipeline) =
            PipelineInstance::find_from_name(shared_data, wireframe_pipeline_name)
        {
            let material = MaterialInstance::create_from_pipeline(shared_data, pipeline);
            mesh_instance.resource().get_mut().set_material(material);
        }

        let message_channel = MessageChannel::default();

        global_messenger
            .write()
            .unwrap()
            .register_messagebox::<ViewEvent>(message_channel.get_messagebox());
        Self {
            mesh_instance,
            shared_data: shared_data.clone(),
            message_channel,
            objects_to_draw: Vec::new(),
        }
    }
    pub fn update(&mut self) {
        self.update_events();

        let mut mesh_data = MeshData::default();
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
                let (vertices, indices) = create_cube_from_min_max(min, max);
                mesh_data.append_mesh(&vertices, &indices);
            }
        }
        self.mesh_instance
            .resource()
            .get_mut()
            .set_mesh_data(mesh_data);
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
