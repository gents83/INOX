use sabi_nodes::{implement_node, ScriptExecution};
use sabi_serialize::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
#[serde(crate = "sabi_serialize")]
struct InnerInnerData {
    pub last_value: u32,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(crate = "sabi_serialize")]
struct InnerData {
    pub in_mid_value: u32,
    pub out_inner_data: InnerInnerData,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(crate = "sabi_serialize")]
pub struct MoveNode {
    in_run: ScriptExecution,
    in_x: f32,
    in_y: f32,
    in_z: f32,
    data: InnerData,
}
implement_node!(MoveNode, "LogicNodeBase", "Node will move object in space");
