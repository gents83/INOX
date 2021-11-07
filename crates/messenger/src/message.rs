use std::any::{type_name, Any};

use nrg_commands::CommandParser;

use crate::MessageBox;

pub trait Message: Send + Sync + Any {
    fn as_any(&self) -> &dyn Any;
    fn redo(&self, events_rw: &MessageBox);
    fn undo(&self, events_rw: &MessageBox);

    #[inline]
    fn get_type_name(&self) -> String {
        let mut str = type_name::<Self>()
            .split(':')
            .collect::<Vec<&str>>()
            .last()
            .unwrap()
            .to_string();
        str.push_str(" - ");
        str.push_str(self.get_debug_info().as_str());
        str
    }
    fn get_debug_info(&self) -> String;
    fn as_boxed(&self) -> Box<dyn Message>;
}

pub trait MessageFromString: Message {
    fn from_command_parser(command_parser: CommandParser) -> Option<Box<dyn Message>>
    where
        Self: Sized;

    fn from_string(s: String) -> Option<Box<dyn Message>>
    where
        Self: Sized,
    {
        let command_parser = CommandParser::from_string(s);
        Self::from_command_parser(command_parser)
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
