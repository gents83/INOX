use nrg_math::*;
use nrg_platform::*;
use nrg_serialize::*;

pub enum WidgetEvent {
    Entering(UID),
    Exiting(UID),
    Pressed(UID),
    Released(UID),
    Dragging(UID, Vector2f),
}
impl Event for WidgetEvent {}
