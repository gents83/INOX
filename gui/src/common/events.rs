use nrg_math::Vector2;
use nrg_platform::Event;
use nrg_serialize::Uid;

pub enum WidgetEvent {
    Entering(Uid),
    Exiting(Uid),
    Pressed(Uid, Vector2),
    Released(Uid, Vector2),
    Dragging(Uid, Vector2),
}
impl Event for WidgetEvent {}
