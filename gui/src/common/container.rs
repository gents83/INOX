use nrg_serialize::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(crate = "nrg_serialize")]
pub enum ContainerFillType {
    None,
    Vertical,
    Horizontal,
}
