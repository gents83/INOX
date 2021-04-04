use nrg_serialize::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(crate = "nrg_serialize")]
pub enum HorizontalAlignment {
    None,
    Left,
    Right,
    Center,
    Stretch,
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(crate = "nrg_serialize")]
pub enum VerticalAlignment {
    None,
    Top,
    Bottom,
    Center,
    Stretch,
}
