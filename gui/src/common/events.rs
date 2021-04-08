use nrg_math::Vector2u;
use nrg_platform::Event;
use nrg_serialize::UID;

pub enum WidgetEvent {
    Entering(UID),
    Exiting(UID),
    Pressed(UID),
    Released(UID),
    Dragging(UID, Vector2u),
}
impl Event for WidgetEvent {}
