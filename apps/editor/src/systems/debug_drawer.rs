use std::any::TypeId;

use nrg_graphics::{
    create_arrow, create_colored_quad, create_line, create_sphere, Material, Mesh, MeshData,
    Pipeline,
};
use nrg_math::{Vector2, Vector3, Vector4};
use nrg_messenger::{implement_message, read_messages, Message, MessageChannel, MessengerRw};
use nrg_resources::{DataTypeResource, Resource, SharedDataRw};

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
    BoundingBox(Vector3, Vector3, Vector4),     // (min, max, color)
    Quad(Vector2, Vector2, f32, Vector4, bool), // (min, max, z, color, is_wireframe)
    Arrow(Vector3, Vector3, Vector4, bool),     // (start, direction, color, is_wireframe)
    Sphere(Vector3, f32, Vector4, bool),        // (position, radius, color, is_wireframe)
}
implement_message!(DrawEvent);

const WIREFRAME_MESH_CATEGORY_IDENTIFIER: &str = "EditorWireframe";

pub struct DebugDrawer {
    mesh_instance: Resource<Mesh>,
    wireframe_mesh_instance: Resource<Mesh>,
    message_channel: MessageChannel,
}

impl DebugDrawer {
    pub fn new(
        shared_data: &SharedDataRw,
        global_messenger: &MessengerRw,
        pipeline_default: &Resource<Pipeline>,
        pipeline_wirefrane: &Resource<Pipeline>,
    ) -> Self {
        let mesh_instance = Mesh::create_from_data(
            shared_data,
            MeshData::new(WIREFRAME_MESH_CATEGORY_IDENTIFIER),
        );
        let material = Material::create_from_pipeline(shared_data, pipeline_default);
        mesh_instance.get_mut().set_material(material);

        let wireframe_mesh_instance = Mesh::create_from_data(
            shared_data,
            MeshData::new(WIREFRAME_MESH_CATEGORY_IDENTIFIER),
        );
        let material = Material::create_from_pipeline(shared_data, pipeline_wirefrane);
        wireframe_mesh_instance.get_mut().set_material(material);

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

    fn auto_send_event(&self, event: DrawEvent) {
        self.message_channel
            .get_messagebox()
            .write()
            .unwrap()
            .send(event.as_boxed())
            .ok();
    }

    fn update_events(&mut self) {
        nrg_profiler::scoped_profile!("update_events");

        let mut mesh_data = MeshData::new(WIREFRAME_MESH_CATEGORY_IDENTIFIER);
        let mut wireframe_mesh_data = MeshData::new(WIREFRAME_MESH_CATEGORY_IDENTIFIER);
        read_messages(self.message_channel.get_listener(), |msg| {
            if msg.type_id() == TypeId::of::<DrawEvent>() {
                let event = msg.as_any().downcast_ref::<DrawEvent>().unwrap();
                match *event {
                    DrawEvent::Line(start, end, color) => {
                        let (vertices, indices) = create_line(start, end, color);
                        wireframe_mesh_data.append_mesh(&vertices, &indices);
                    }
                    DrawEvent::BoundingBox(min, max, color) => {
                        self.auto_send_event(DrawEvent::Quad(
                            [min.x, min.y].into(),
                            [max.x, max.y].into(),
                            min.z,
                            color,
                            true,
                        ));
                        self.auto_send_event(DrawEvent::Quad(
                            [min.x, min.y].into(),
                            [max.x, max.y].into(),
                            max.z,
                            color,
                            true,
                        ));
                        self.auto_send_event(DrawEvent::Line(
                            [min.x, min.y, min.z].into(),
                            [min.x, min.y, max.z].into(),
                            color,
                        ));
                        self.auto_send_event(DrawEvent::Line(
                            [min.x, max.y, min.z].into(),
                            [min.x, max.y, max.z].into(),
                            color,
                        ));
                        self.auto_send_event(DrawEvent::Line(
                            [max.x, min.y, min.z].into(),
                            [max.x, min.y, max.z].into(),
                            color,
                        ));
                        self.auto_send_event(DrawEvent::Line(
                            [max.x, max.y, min.z].into(),
                            [max.x, max.y, max.z].into(),
                            color,
                        ));
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
        self.mesh_instance.get_mut().set_mesh_data(mesh_data);
        self.wireframe_mesh_instance
            .get_mut()
            .set_mesh_data(wireframe_mesh_data);
    }
}