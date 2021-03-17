use nrg_platform::*;
use nrg_serialize::*;

pub enum WidgetEvent {
    Entering(UID),
    Exiting(UID),
    Pressed(UID),
    Released(UID),
}
impl Event for WidgetEvent {}
