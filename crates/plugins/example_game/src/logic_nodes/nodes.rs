/*
use sabi_nodes::{LogicExecution, NodeTrait, Pin, INPUT_PIN, OUTPUT_PIN};
use sabi_serialize::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
#[serde(crate = "sabi_serialize")]
struct InnerInnerData {
    pub last_value: u32,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(crate = "sabi_serialize")]
struct InnerData {
    pub mid_value: Pin<u32, INPUT_PIN>,
    pub inner_data: Pin<InnerInnerData, OUTPUT_PIN>,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(crate = "sabi_serialize")]
pub struct MoveNode {
    run: Pin<LogicExecution, INPUT_PIN>,
    x: Pin<f32, INPUT_PIN>,
    y: Pin<f32, INPUT_PIN>,
    z: Pin<f32, INPUT_PIN>,
    data: InnerData,
}
impl NodeTrait for MoveNode {
    fn category() -> &'static str {
        "Movement"
    }
    fn description() -> &'static str {
        "Node will move object in space"
    }
}

#[test]
fn test_node_tree() {
    use crate::logic_nodes::MoveNode;
    use sabi_nodes::{LogicNodeRegistry, NodeTree, RustExampleNode, ScriptInitNode};
    use sabi_serialize::serialize;

    let mut registry = LogicNodeRegistry::default();
    registry.register_node::<ScriptInitNode>();
    registry.register_node::<RustExampleNode>();
    registry.register_node::<MoveNode>();

    let mut tree = NodeTree::default();

    let init = ScriptInitNode::default();
    let serialized_data = serialize(&init);

    if let Some(n) = registry.deserialize(&serialized_data) {
        tree.add_node(n);
    }
    assert_eq!(tree.get_nodes().len(), 1);

    let mut node_a = RustExampleNode::default();
    if let Some(pin) = node_a.get_pin_mut("out_float") {
        pin.set(12.);
    }
    assert_eq!(node_a.out_float, 12.);
    let serialized_data = serialize(&node_a);
    println!("{}", serialized_data);

    if let Some(n) = registry.deserialize(&serialized_data) {
        tree.add_node(n);
    }
    assert_eq!(tree.get_nodes().len(), 2);

    let node_b = MoveNode::default();
    let serialized_data = serialize(&node_b);
    println!("{}", serialized_data);

    if let Some(n) = registry.deserialize(&serialized_data) {
        tree.add_node(n);
    }
    assert_eq!(tree.get_nodes().len(), 3);
}
*/
