use std::any::TypeId;

use crate::{
    create_arrow, create_colored_quad, create_line, create_sphere, Material, Mesh, MeshData,
    Pipeline, PipelineType,
};
use sabi_commands::CommandParser;
use sabi_core::System;
use sabi_math::{Vector2, Vector3, Vector4};
use sabi_messenger::{
    implement_message, read_messages, Message, MessageChannel, MessageFromString, MessengerRw,
};
use sabi_profiler::debug_log;
use sabi_resources::{DataTypeResource, Resource, SharedDataRc};
use sabi_serialize::generate_random_uid;

/// A debug drawer
/// You can use this to draw things in the editor just sending events:
/// ```
/// use sabi_editor::systems::{DebugDrawer, DrawEvent};
/// use sabi_math::{Vector3, Zero};
/// use sabi_messenger::{MessengerRw, Message};
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

impl MessageFromString for DrawEvent {
    fn from_command_parser(command_parser: CommandParser) -> Option<Box<dyn Message>>
    where
        Self: Sized,
    {
        if command_parser.has("draw_line") {
            let values = command_parser.get_values_of("draw_line");
            return Some(
                DrawEvent::Line(
                    Vector3::new(values[0], values[1], values[2]),
                    Vector3::new(values[3], values[4], values[5]),
                    Vector4::new(values[6], values[7], values[8], values[9]),
                )
                .as_boxed(),
            );
        } else if command_parser.has("draw_bounding_box") {
            let values = command_parser.get_values_of("draw_bounding_box");
            return Some(
                DrawEvent::BoundingBox(
                    Vector3::new(values[0], values[1], values[2]),
                    Vector3::new(values[3], values[4], values[5]),
                    Vector4::new(values[6], values[7], values[8], values[9]),
                )
                .as_boxed(),
            );
        } else if command_parser.has("draw_quad") {
            let values = command_parser.get_values_of("draw_quad");
            return Some(
                DrawEvent::Quad(
                    Vector2::new(values[0], values[1]),
                    Vector2::new(values[2], values[3]),
                    values[4],
                    Vector4::new(values[5], values[6], values[7], values[8]),
                    false,
                )
                .as_boxed(),
            );
        } else if command_parser.has("draw_quad_wireframe") {
            let values = command_parser.get_values_of("draw_quad_wireframe");
            return Some(
                DrawEvent::Quad(
                    Vector2::new(values[0], values[1]),
                    Vector2::new(values[2], values[3]),
                    values[4],
                    Vector4::new(values[5], values[6], values[7], values[8]),
                    true,
                )
                .as_boxed(),
            );
        } else if command_parser.has("draw_arrow") {
            let values = command_parser.get_values_of("draw_arrow");
            return Some(
                DrawEvent::Arrow(
                    Vector3::new(values[0], values[1], values[2]),
                    Vector3::new(values[3], values[4], values[5]),
                    Vector4::new(values[6], values[7], values[8], values[9]),
                    false,
                )
                .as_boxed(),
            );
        } else if command_parser.has("draw_arrow_wireframe") {
            let values = command_parser.get_values_of("draw_arrow_wireframe");
            return Some(
                DrawEvent::Arrow(
                    Vector3::new(values[0], values[1], values[2]),
                    Vector3::new(values[3], values[4], values[5]),
                    Vector4::new(values[6], values[7], values[8], values[9]),
                    true,
                )
                .as_boxed(),
            );
        } else if command_parser.has("draw_sphere") {
            let values = command_parser.get_values_of("draw_sphere");
            return Some(
                DrawEvent::Sphere(
                    Vector3::new(values[0], values[1], values[2]),
                    values[3],
                    Vector4::new(values[4], values[5], values[6], values[7]),
                    false,
                )
                .as_boxed(),
            );
        } else if command_parser.has("draw_sphere_wireframe") {
            let values = command_parser.get_values_of("draw_sphere_wireframe");
            return Some(
                DrawEvent::Sphere(
                    Vector3::new(values[0], values[1], values[2]),
                    values[3],
                    Vector4::new(values[4], values[5], values[6], values[7]),
                    true,
                )
                .as_boxed(),
            );
        }
        None
    }
}

const WIREFRAME_MESH_CATEGORY_IDENTIFIER: &str = "EditorWireframe";

pub struct DebugDrawerSystem {
    mesh_instance: Resource<Mesh>,
    wireframe_mesh_instance: Resource<Mesh>,
    message_channel: MessageChannel,
}

impl DebugDrawerSystem {
    pub fn new(shared_data: &SharedDataRc, global_messenger: &MessengerRw) -> Self {
        let default_pipeline = shared_data
            .match_resource(|p: &Pipeline| p.data().pipeline_type == PipelineType::Default);
        let wireframe_pipeline = shared_data
            .match_resource(|p: &Pipeline| p.data().pipeline_type == PipelineType::Wireframe);

        if default_pipeline.is_none() {
            debug_log(
                "No pipeline with type Default found - did you forgot to read render.cfg file?",
            );
        }
        if wireframe_pipeline.is_none() {
            debug_log(
                "No pipeline with type Wireframe found - did you forgot to read render.cfg file?",
            );
        }

        let mesh_instance = Mesh::new_resource(
            shared_data,
            global_messenger,
            generate_random_uid(),
            MeshData::new(WIREFRAME_MESH_CATEGORY_IDENTIFIER),
        );
        if let Some(default_pipeline) = &default_pipeline {
            let material = Material::duplicate_from_pipeline(shared_data, default_pipeline);
            mesh_instance.get_mut().set_material(material);
        }

        let wireframe_mesh_instance = Mesh::new_resource(
            shared_data,
            global_messenger,
            generate_random_uid(),
            MeshData::new(WIREFRAME_MESH_CATEGORY_IDENTIFIER),
        );
        if let Some(wireframe_pipeline) = &wireframe_pipeline {
            let material = Material::duplicate_from_pipeline(shared_data, wireframe_pipeline);
            wireframe_mesh_instance.get_mut().set_material(material);
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

    fn auto_send_event(&self, event: DrawEvent) {
        self.message_channel
            .get_messagebox()
            .write()
            .unwrap()
            .send(event.as_boxed())
            .ok();
    }

    fn update_events(&mut self) {
        sabi_profiler::scoped_profile!("update_events");

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
                        let (mut vertices, indices) = create_arrow(position, direction);
                        vertices.iter_mut().for_each(|v| {
                            v.color = color.into();
                        });
                        if is_wireframe {
                            wireframe_mesh_data.append_mesh(&vertices, &indices);
                        } else {
                            mesh_data.append_mesh(&vertices, &indices);
                        }
                    }
                    DrawEvent::Sphere(position, radius, color, is_wireframe) => {
                        let (mut vertices, indices) = create_sphere(radius, 32, 16);
                        vertices.iter_mut().for_each(|v| {
                            v.pos[0] += position.x;
                            v.pos[1] += position.y;
                            v.pos[2] += position.z;
                            v.color = color.into();
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

unsafe impl Send for DebugDrawerSystem {}
unsafe impl Sync for DebugDrawerSystem {}

impl System for DebugDrawerSystem {
    fn read_config(&mut self, _plugin_name: &str) {}

    fn should_run_when_not_focused(&self) -> bool {
        false
    }
    fn init(&mut self) {}

    fn run(&mut self) -> bool {
        self.update_events();
        true
    }

    fn uninit(&mut self) {}
}
