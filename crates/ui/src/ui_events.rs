use std::path::PathBuf;

use inox_commands::CommandParser;
use inox_messenger::implement_message;
use inox_serialize::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(crate = "inox_serialize")]
pub enum DialogOp {
    New,
    Open,
    Save,
}
impl From<&str> for DialogOp {
    fn from(string: &str) -> Self {
        match string {
            "New" => DialogOp::New,
            "Open" => DialogOp::Open,
            "Save" => DialogOp::Save,
            _ => panic!("Unknown DialogOp: {}", string),
        }
    }
}

impl From<DialogOp> for &str {
    fn from(op: DialogOp) -> Self {
        match op {
            DialogOp::New => "New",
            DialogOp::Open => "Open",
            DialogOp::Save => "Save",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(crate = "inox_serialize")]
pub enum DialogEvent {
    Request(DialogOp, PathBuf),
    Confirmed(DialogOp, PathBuf),
    Canceled(DialogOp),
}
implement_message!(DialogEvent, message_from_command_parser);

impl DialogEvent {
    fn message_from_command_parser(command_parser: CommandParser) -> Option<Self>
    where
        Self: Sized,
    {
        if command_parser.has("new_dialog") {
            let values = command_parser.get_values_of::<String>("new_dialog");
            return Some(DialogEvent::Request(
                DialogOp::New,
                PathBuf::from(values[0].as_str()),
            ));
        } else if command_parser.has("open_dialog") {
            let values = command_parser.get_values_of::<String>("open_dialog");
            return Some(DialogEvent::Request(
                DialogOp::Open,
                PathBuf::from(values[0].as_str()),
            ));
        } else if command_parser.has("save_dialog") {
            let values = command_parser.get_values_of::<String>("save_dialog");
            return Some(DialogEvent::Request(
                DialogOp::Save,
                PathBuf::from(values[0].as_str()),
            ));
        } else if command_parser.has("confirm_new_dialog") {
            let values = command_parser.get_values_of::<String>("confirm_new_dialog");
            return Some(DialogEvent::Confirmed(
                DialogOp::New,
                PathBuf::from(values[0].as_str()),
            ));
        } else if command_parser.has("confirm_open_dialog") {
            let values = command_parser.get_values_of::<String>("confirm_open_dialog");
            return Some(DialogEvent::Confirmed(
                DialogOp::Open,
                PathBuf::from(values[0].as_str()),
            ));
        } else if command_parser.has("confirm_save_dialog") {
            let values = command_parser.get_values_of::<String>("confirm_save_dialog");
            return Some(DialogEvent::Confirmed(
                DialogOp::Save,
                PathBuf::from(values[0].as_str()),
            ));
        } else if command_parser.has("cancel_new_dialog") {
            return Some(DialogEvent::Canceled(DialogOp::New));
        } else if command_parser.has("cancel_open_dialog") {
            return Some(DialogEvent::Canceled(DialogOp::Open));
        } else if command_parser.has("cancel_save_dialog") {
            return Some(DialogEvent::Canceled(DialogOp::Save));
        }
        None
    }
}
