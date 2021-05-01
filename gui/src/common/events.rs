use nrg_math::Vector2;
use nrg_events::{implement_event, Event, EventsRw};
use nrg_serialize::Uid;

#[derive(Clone, Copy)]
pub enum WidgetEvent {
    Entering(Uid),
    Exiting(Uid),
    Pressed(Uid, Vector2),
    Released(Uid, Vector2),
    Dragging(Uid, Vector2),
}

implement_event!(WidgetEvent);
