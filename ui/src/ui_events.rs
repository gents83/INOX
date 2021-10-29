use std::path::PathBuf;

use nrg_messenger::implement_message;
use nrg_serialize::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
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
#[serde(crate = "nrg_serialize")]
pub enum DialogEvent {
    Request(DialogOp, PathBuf),
    Confirmed(DialogOp, PathBuf),
    Canceled(DialogOp),
}
implement_message!(DialogEvent);
