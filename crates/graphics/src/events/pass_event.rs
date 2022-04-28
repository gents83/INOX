use inox_commands::CommandParser;
use inox_messenger::implement_message;

use crate::Pass;

pub enum PassEvent {
    Add(Box<dyn Pass>),
}
implement_message!(PassEvent, message_from_command_parser, compare_and_discard);

impl PassEvent {
    fn compare_and_discard(&self, _other: &Self) -> bool {
        false
    }
    fn message_from_command_parser(_command_parser: CommandParser) -> Option<Self>
    where
        Self: Sized,
    {
        None
    }
}
