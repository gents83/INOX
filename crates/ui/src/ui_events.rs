use inox_commands::CommandParser;
use inox_messenger::implement_message;

use crate::{UIInstance, UIVertex};

#[derive(Clone)]
pub enum UIEvent {
    Scale(f32),
    DrawData(Vec<UIVertex>, Vec<u32>, Vec<UIInstance>),
}
implement_message!(UIEvent, message_from_command_parser, compare_and_discard);

impl UIEvent {
    fn compare_and_discard(&self, _other: &Self) -> bool {
        true
    }
    fn message_from_command_parser(_command_parser: CommandParser) -> Option<Self>
    where
        Self: Sized,
    {
        None
    }
}
