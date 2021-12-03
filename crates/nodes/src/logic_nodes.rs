use sabi_serialize::{Deserialize, Serialize};

use crate::{implement_node, implement_pin, Node, NodeTrait};
use sabi_serialize::typetag;

#[derive(Serialize, Deserialize, Copy, Clone)]
#[serde(crate = "sabi_serialize")]
pub enum LogicExecution {
    Type,
}
impl Default for LogicExecution {
    fn default() -> Self {
        LogicExecution::Type
    }
}
implement_pin!(LogicExecution);

#[derive(Serialize, Deserialize)]
#[serde(crate = "sabi_serialize")]
pub struct RustExampleNode {
    node: Node,
}
implement_node!(RustExampleNode, node);
impl Default for RustExampleNode {
    fn default() -> Self {
        let mut node = Node::new("RustExampleNode", "Example", "Rust example node");
        node.add_input("in_int", 0_i32);
        node.add_input("in_float", 0_f32);
        node.add_input("in_string", String::new());
        node.add_input("in_bool", false);
        node.add_input("in_execute", LogicExecution::default());

        node.add_output("out_execute", LogicExecution::default());
        node.add_output("out_int", 0_i32);
        node.add_output("out_float", 0_f32);
        node.add_output("out_string", String::new());
        node.add_output("out_bool", false);
        Self { node }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "sabi_serialize")]
pub struct ScriptInitNode {
    node: Node,
}
implement_node!(ScriptInitNode, node);
impl Default for ScriptInitNode {
    fn default() -> Self {
        let mut node = Node::new("ScriptInitNode", "Init", "Script init node");
        node.add_output("out_execute", LogicExecution::default());
        Self { node }
    }
}

#[allow(dead_code)]
fn test_node() {
    use crate::{LogicNodeRegistry, NodeTree};
    use sabi_serialize::serialize;

    let mut registry = LogicNodeRegistry::default();
    registry.register_node::<RustExampleNode>();
    registry.register_node::<ScriptInitNode>();

    registry.register_pin_type::<f32>();
    registry.register_pin_type::<f64>();
    registry.register_pin_type::<u8>();
    registry.register_pin_type::<i8>();
    registry.register_pin_type::<u16>();
    registry.register_pin_type::<i16>();
    registry.register_pin_type::<u32>();
    registry.register_pin_type::<i32>();
    registry.register_pin_type::<bool>();
    registry.register_pin_type::<String>();
    registry.register_pin_type::<LogicExecution>();

    let mut tree = NodeTree::default();
    tree.add_link("NodeA", "NodeB", "out_int", "in_int");
    tree.add_link("NodeA", "NodeB", "out_execute", "in_execute");
    assert_eq!(tree.get_links_count(), 2);

    let init = ScriptInitNode::default();
    let serialized_data = serialize(&init);
    println!("ScriptInitNode");
    println!("{}", serialized_data);

    if let Some(n) = registry.deserialize(&serialized_data) {
        tree.add_node(n);
    }
    assert_eq!(tree.get_nodes_count(), 1);

    let mut node_a = RustExampleNode::default();
    node_a.set_name("NodeA");
    if let Some(v) = node_a.node_mut().get_output_mut::<i32>("out_int") {
        *v = 19;
    }
    assert_eq!(*node_a.node().get_output::<i32>("out_int").unwrap(), 19);
    let serialized_data = serialize(&node_a);
    println!("RustExampleNode - A");
    println!("{}", serialized_data);

    if let Some(n) = registry.deserialize(&serialized_data) {
        tree.add_node(n);
    }
    assert_eq!(tree.get_nodes_count(), 2);

    tree.add_default_node::<RustExampleNode>("NodeB");
    assert_eq!(tree.get_nodes_count(), 3);

    tree.resolve_links(&registry);

    if let Some(node_b) = tree.find_node_as::<RustExampleNode>("NodeB") {
        assert_eq!(*node_b.node().get_input::<i32>("in_int").unwrap(), 19);
        let serialized_data = serialize(node_b);
        println!("RustExampleNode - B");
        println!("{}", serialized_data);
    }
}

#[test]
fn test_node_fn() {
    test_node()
}
