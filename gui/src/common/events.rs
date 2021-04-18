use nrg_math::Vector2;
use nrg_platform::Event;
use nrg_serialize::Uid;

pub enum WidgetEvent {
    Entering(Uid),
    Exiting(Uid),
    Pressed(Uid),
    Released(Uid),
    Dragging(Uid, Vector2),
}
impl Event for WidgetEvent {}
