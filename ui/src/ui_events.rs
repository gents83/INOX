use std::path::PathBuf;

use nrg_messenger::implement_message;
use nrg_serialize::*;

#[derive(Clone, Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub enum DialogEvent {
    Confirmed(Uid, Uid, PathBuf), //my uid, requester uid, text
    Canceled(Uid),
}
implement_message!(DialogEvent);
