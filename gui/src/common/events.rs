use nrg_math::Vector2;
use nrg_messenger::implement_message;
use nrg_serialize::Uid;

#[derive(Clone, Copy)]
pub enum WidgetEvent {
    Entering(Uid),
    Exiting(Uid),
    Pressed(Uid, Vector2),
    Released(Uid, Vector2),
    Dragging(Uid, Vector2),
    InvalidateLayout(Uid),
}

implement_message!(WidgetEvent);
