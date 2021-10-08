use nrg_messenger::implement_message;
use nrg_scene::ObjectId;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EditMode {
    View,
    Select,
    Move,
    Rotate,
    Scale,
}

#[derive(Clone)]
pub enum EditorEvent {
    Selected(ObjectId),
    HoverMesh(u32),
    ChangeMode(EditMode),
}
implement_message!(EditorEvent);
