use std::any::TypeId;

use nrg_graphics::{
    create_arrow, create_colored_quad, create_line, create_sphere, MaterialInstance, MeshData,
    MeshInstance, MeshRc, PipelineInstance,
};
use nrg_math::{Vector2, Vector3, Vector4};
use nrg_messenger::{implement_message, read_messages, MessageChannel, MessengerRw};
use nrg_resources::{DataTypeResource, SharedDataRw};

/// A debug drawer
/// You can use this to draw things in the editor just sending events:
/// ```
/// use nrg_editor::systems::{DebugDrawer, DrawEvent};
/// use nrg_math::{Vector3, Zero};
/// use nrg_messenger::{MessengerRw, Message};
///
/// let global_messenger = MessengerRw::default();
/// let global_dispatcher = global_messenger.read().unwrap().get_dispatcher().clone();
///     global_dispatcher
///     .write()
///     .unwrap()
///     .send(
///         DrawEvent::Sphere([2., 2., 2.].into(), 2., [1., 0., 0., 1.].into(), true)
///             .as_boxed(),
///     )
///     .ok();
///
///     global_dispatcher
///     .write()
///     .unwrap()
///     .send(
///         DrawEvent::Arrow(
///             Vector3::zero(),
///             [2., 2., 0.].into(),
///             [1., 0., 0., 1.].into(),
///             false,
///         )
///         .as_boxed(),
///     )
///     .ok();
/// ```

#[derive(Clone)]
#[allow(dead_code)]
pub enum DrawEvent {
    Line(Vector3, Vector3, Vector4),            // (start, end, color)
    Quad(Vector2, Vector2, f32, Vector4, bool), // (min, max, z, color, is_wireframe)
    Arrow(Vector3, Vector3, Vector4, bool),     // (start, direction, color, is_wireframe)
    Sphere(Vector3, f32, Vector4, bool),        // (position, radius, color, is_wireframe)
}
implement_message!(DrawEvent);

pub struct DebugDrawer {
    mesh_instance: MeshRc,
    wireframe_mesh_instance: MeshRc,
    message_channel: MessageChannel,
}

impl DebugDrawer {
    pub fn new(
        shared_data: &SharedDataRw,
        global_messenger: &MessengerRw,
        pipeline_name_default: &str,
        pipeline_name_wirefrane: &str,
    ) -> Self {
        let mesh_instance = MeshInstance::create_from_data(shared_data, MeshData::default());
        if let Some(pipeline) = PipelineInstance::find_from_name(shared_data, pipeline_name_default)
        {
            let material = MaterialInstance::create_from_pipeline(shared_data, pipeline);
            mesh_instance.resource().get_mut().set_material(material);
        }
        let wireframe_mesh_instance =
            MeshInstance::create_from_data(shared_data, MeshData::default());
        if let Some(pipeline) =
            PipelineInstance::find_from_name(shared_data, pipeline_name_wirefrane)
        {
            let material = MaterialInstance::create_from_pipeline(shared_data, pipeline);
            wireframe_mesh_instance
                .resource()
                .get_mut()
                .set_material(material);
        }

        let message_channel = MessageChannel::default();

        global_messenger
            .write()
            .unwrap()
            .register_messagebox::<DrawEvent>(message_channel.get_messagebox());
        Self {
            mesh_instance,
            wireframe_mesh_instance,
            message_channel,
        }
    }
    pub fn update(&mut self) {
        self.update_events();
    }

    fn update_events(&mut self) {
        nrg_profiler::scoped_profile!("update_events");

        let mut mesh_data = MeshData::default();
        let mut wireframe_mesh_data = MeshData::default();
        read_messages(self.message_channel.get_listener(), |msg| {
            if msg.type_id() == TypeId::of::<DrawEvent>() {
                let event = msg.as_any().downcast_ref::<DrawEvent>().unwrap();
                match *event {
                    DrawEvent::Line(start, end, color) => {
                        let (vertices, indices) = create_line(start, end, color);
                        wireframe_mesh_data.append_mesh(&vertices, &indices);
                    }
                    DrawEvent::Quad(min, max, z, color, is_wireframe) => {
                        if is_wireframe {
                            let (vertices, indices) = create_line(
                                [min.x, min.y, z].into(),
                                [min.x, max.y, z].into(),
                                color,
                            );
                            wireframe_mesh_data.append_mesh(&vertices, &indices);
                            let (vertices, indices) = create_line(
                                [min.x, max.y, z].into(),
                                [max.x, max.y, z].into(),
                                color,
                            );
                            wireframe_mesh_data.append_mesh(&vertices, &indices);
                            let (vertices, indices) = create_line(
                                [max.x, max.y, z].into(),
                                [max.x, min.y, z].into(),
                                color,
                            );
                            wireframe_mesh_data.append_mesh(&vertices, &indices);
                            let (vertices, indices) = create_line(
                                [max.x, min.y, z].into(),
                                [min.x, min.y, z].into(),
                                color,
                            );
                            wireframe_mesh_data.append_mesh(&vertices, &indices);
                        } else {
                            let (vertices, indices) =
                                create_colored_quad([min.x, min.y, max.x, max.y].into(), z, color);
                            mesh_data.append_mesh(&vertices, &indices);
                        }
                    }
                    DrawEvent::Arrow(position, direction, color, is_wireframe) => {
                        let (vertices, indices) = create_arrow(position, direction, color);
                        if is_wireframe {
                            wireframe_mesh_data.append_mesh(&vertices, &indices);
                        } else {
                            mesh_data.append_mesh(&vertices, &indices);
                        }
                    }
                    DrawEvent::Sphere(position, radius, color, is_wireframe) => {
                        let (mut vertices, indices) = create_sphere(radius, 32, 16);
                        vertices.iter_mut().for_each(|v| {
                            v.pos += position;
                            v.color = color;
                        });
                        if is_wireframe {
                            wireframe_mesh_data.append_mesh(&vertices, &indices);
                        } else {
                            mesh_data.append_mesh(&vertices, &indices);
                        }
                    }
                }
            }
        });
        self.mesh_instance
            .resource()
            .get_mut()
            .set_mesh_data(mesh_data);
        self.wireframe_mesh_instance
            .resource()
            .get_mut()
            .set_mesh_data(wireframe_mesh_data);
    }
}
