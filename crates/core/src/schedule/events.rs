use inox_commands::CommandParser;
use inox_messenger::implement_message;

use crate::{Phases, SystemId};

pub enum SystemEvent {
    Added(SystemId, Phases),
    Removed(SystemId, Phases),
}

implement_message!(
    SystemEvent,
    message_from_command_parser,
    compare_and_discard
);

impl SystemEvent {
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
