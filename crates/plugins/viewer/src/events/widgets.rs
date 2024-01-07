use inox_commands::CommandParser;
use inox_messenger::implement_message;
use inox_uid::Uid;

#[derive(PartialEq, Eq)]
pub enum WidgetType {
    Hierarchy,
    Gfx,
}
pub enum WidgetEvent {
    Selected(Uid),
    Create(WidgetType),
    Destroy(WidgetType),
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
                _ => false,
            },
            Self::Create(t) => match other {
                Self::Create(other_t) => t == other_t,
                _ => false,
            },
            Self::Destroy(t) => match other {
                Self::Destroy(other_t) => t == other_t,
                _ => false,
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
