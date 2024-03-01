use inox_commands::CommandParser;
use inox_math::{Vector2, Vector3, Vector4};
use inox_messenger::implement_message;

#[derive(Clone)]
#[allow(dead_code)]
pub enum DrawEvent {
    Line(Vector3, Vector3, Vector4),            // (start, end, color)
    BoundingBox(Vector3, Vector3, Vector4),     // (min, max, color)
    Quad(Vector2, Vector2, f32, Vector4, bool), // (min, max, z, color, is_wireframe)
    Arrow(Vector3, Vector3, Vector4, bool),     // (start, direction, color, is_wireframe)
    Sphere(Vector3, f32, Vector4, bool),        // (position, radius, color, is_wireframe)
    Circle(Vector3, f32, Vector4, bool),        // (position, radius, color, is_wireframe)
}
implement_message!(DrawEvent, message_from_command_parser, compare_and_discard);

impl DrawEvent {
    fn compare_and_discard(&self, _other: &Self) -> bool {
        false
    }
    fn message_from_command_parser(command_parser: CommandParser) -> Option<Self>
    where
        Self: Sized,
    {
        if command_parser.has("draw_line") {
            let values = command_parser.get_values_of("draw_line");
            return Some(DrawEvent::Line(
                Vector3::new(values[0], values[1], values[2]),
                Vector3::new(values[3], values[4], values[5]),
                Vector4::new(values[6], values[7], values[8], values[9]),
            ));
        } else if command_parser.has("draw_bounding_box") {
            let values = command_parser.get_values_of("draw_bounding_box");
            return Some(DrawEvent::BoundingBox(
                Vector3::new(values[0], values[1], values[2]),
                Vector3::new(values[3], values[4], values[5]),
                Vector4::new(values[6], values[7], values[8], values[9]),
            ));
        } else if command_parser.has("draw_quad") {
            let values = command_parser.get_values_of("draw_quad");
            return Some(DrawEvent::Quad(
                Vector2::new(values[0], values[1]),
                Vector2::new(values[2], values[3]),
                values[4],
                Vector4::new(values[5], values[6], values[7], values[8]),
                false,
            ));
        } else if command_parser.has("draw_quad_wireframe") {
            let values = command_parser.get_values_of("draw_quad_wireframe");
            return Some(DrawEvent::Quad(
                Vector2::new(values[0], values[1]),
                Vector2::new(values[2], values[3]),
                values[4],
                Vector4::new(values[5], values[6], values[7], values[8]),
                true,
            ));
        } else if command_parser.has("draw_arrow") {
            let values = command_parser.get_values_of("draw_arrow");
            return Some(DrawEvent::Arrow(
                Vector3::new(values[0], values[1], values[2]),
                Vector3::new(values[3], values[4], values[5]),
                Vector4::new(values[6], values[7], values[8], values[9]),
                false,
            ));
        } else if command_parser.has("draw_arrow_wireframe") {
            let values = command_parser.get_values_of("draw_arrow_wireframe");
            return Some(DrawEvent::Arrow(
                Vector3::new(values[0], values[1], values[2]),
                Vector3::new(values[3], values[4], values[5]),
                Vector4::new(values[6], values[7], values[8], values[9]),
                true,
            ));
        } else if command_parser.has("draw_sphere") {
            let values = command_parser.get_values_of("draw_sphere");
            return Some(DrawEvent::Sphere(
                Vector3::new(values[0], values[1], values[2]),
                values[3],
                Vector4::new(values[4], values[5], values[6], values[7]),
                false,
            ));
        } else if command_parser.has("draw_sphere_wireframe") {
            let values = command_parser.get_values_of("draw_sphere_wireframe");
            return Some(DrawEvent::Sphere(
                Vector3::new(values[0], values[1], values[2]),
                values[3],
                Vector4::new(values[4], values[5], values[6], values[7]),
                true,
            ));
        } else if command_parser.has("draw_circle_wireframe") {
            let values = command_parser.get_values_of("draw_circle_wireframe");
            return Some(DrawEvent::Circle(
                Vector3::new(values[0], values[1], values[2]),
                values[3],
                Vector4::new(values[4], values[5], values[6], values[7]),
                true,
            ));
        }
        None
    }
}
