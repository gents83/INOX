use sabi_blender::{implement_node, logic_nodes::ScriptExecution};
use sabi_serialize::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
#[serde(crate = "sabi_serialize")]
pub struct MoveNode {
    in_run: ScriptExecution,
    in_x: f32,
    in_y: f32,
    in_z: f32,
}
implement_node!(MoveNode, "LogicNodeBase", "Node will move object in space");
