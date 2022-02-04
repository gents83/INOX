use std::any::type_name;

use inox_commands::CommandParser;

use crate::{MessageHub, MessageHubRc};

pub trait Message: Send + Sync + 'static {
    fn send(self: Box<Self>, messenger: &mut MessageHub);
    #[inline]
    fn send_to(self, messenger: &MessageHubRc)
    where
        Self: Sized,
    {
        messenger.send_event(self);
    }
    fn from_string(_s: &str) -> Option<Self>
    where
        Self: Sized,
    {
        None
    }

    #[inline]
    fn get_type_name(&self) -> String {
        type_name::<Self>()
            .split(':')
            .collect::<Vec<&str>>()
            .last()
            .unwrap()
            .to_string()
    }
}
pub trait MessageFromString: Message {
    fn from_command_parser(command_parser: CommandParser) -> Option<Self>
    where
        Self: Sized;
    #[inline]
    fn from_string(s: &str) -> Option<Self>
    where
        Self: Sized,
    {
        let command_parser = CommandParser::from_string(s);
        <Self as MessageFromString>::from_command_parser(command_parser)
    }
}

fn read_event(string: String) -> (bool, String, String) {
    if let Some(pos) = string.find("[[[") {
        let (_, string) = string.split_at(pos + 3);
        if let Some(pos) = string.find("]]]") {
            let (serialized_event, string) = string.split_at(pos);
            let (_, string) = string.split_at(3);
            return (true, serialized_event.to_string(), string.to_string());
        }
    }
    (false, String::default(), String::default())
}

pub fn get_events_from_string(string: String) -> Vec<String> {
    let mut result = Vec::new();
    let (is_event, serialized_event, string) = read_event(string);
    if is_event {
        result.push(serialized_event);
        for e in get_events_from_string(string) {
            result.push(e);
        }
    }
    result
}
