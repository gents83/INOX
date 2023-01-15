use inox_commands::CommandParser;
use inox_messenger::implement_message;
use inox_uid::Uid;

pub enum WidgetEvent {
    Selected(Uid),
}

implement_message!(
    WidgetEvent,
    message_from_command_parser,
    compare_and_discard
);

impl WidgetEvent {
    fn compare_and_discard(&self, other: &Self) -> bool {
        match self {
            Self::Selected(id) => match other {
                Self::Selected(other_id) => id == other_id,
            },
        }
    }
    fn message_from_command_parser(command_parser: CommandParser) -> Option<Self> {
        if command_parser.has("select_object") {
            let values = command_parser.get_values_of::<String>("select_object");
            if let Ok(id) = Uid::parse_str(values[0].as_str()) {
                return Some(Self::Selected(id));
            }
        }
        None
    }
}
