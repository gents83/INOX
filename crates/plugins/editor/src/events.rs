use sabi_commands::CommandParser;
use sabi_messenger::{implement_message, Message, MessageFromString};
use sabi_scene::ObjectId;

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

impl MessageFromString for EditorEvent {
    fn from_command_parser(command_parser: CommandParser) -> Option<Box<dyn Message>>
    where
        Self: Sized,
    {
        if command_parser.has("select_object") {
            let values = command_parser.get_values_of::<ObjectId>("select_object");
            return Some(EditorEvent::Selected(values[0]).as_boxed());
        }
        //TODO support other messages
        None
    }
}
