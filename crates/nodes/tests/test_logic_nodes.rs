use inox_nodes::{
    LogicData, LogicExecution, LogicNodeRegistry, NodeTrait, NodeTree, RustExampleNode,
    ScriptInitNode,
};
use inox_serialize::{deserialize, serialize};

#[test]
fn registers_serializes_and_executes_example_nodes() {
    let mut registry = LogicNodeRegistry::default();
    registry.register_node::<ScriptInitNode>();
    registry.register_node::<RustExampleNode>();
    registry.register_pin_type::<f32>();
    registry.register_pin_type::<i32>();
    registry.register_pin_type::<bool>();
    registry.register_pin_type::<String>();
    registry.register_pin_type::<LogicExecution>();

    let mut tree = NodeTree::default();
    tree.add_link("ScriptInitNode", "NodeA", "Execute", "in_execute");
    tree.add_link("NodeA", "NodeB", "out_int", "in_int");
    tree.add_link("NodeA", "NodeB", "out_string", "in_string");
    tree.add_link("NodeA", "NodeB", "out_execute", "in_execute");
    assert_eq!(tree.get_links_count(), 4);

    let init = ScriptInitNode::default();
    let serialized_init = init.serialize_node();
    tree.add_node(
        registry
            .deserialize_node(&serialized_init)
            .expect("registered init node should deserialize"),
    );

    let mut node = RustExampleNode::default();
    node.set_name("NodeA");
    *node.node_mut().get_input_mut::<i32>("in_int").unwrap() = 19;
    *node.node_mut().get_input_mut::<f32>("in_float").unwrap() = 22.0;
    *node
        .node_mut()
        .get_input_mut::<String>("in_string")
        .unwrap() = "Ciao".to_owned();
    *node.node_mut().get_input_mut::<bool>("in_bool").unwrap() = true;
    assert_eq!(*node.node().get_input::<i32>("in_int").unwrap(), 19);
    assert_eq!(*node.node().get_input::<f32>("in_float").unwrap(), 22.0);
    assert_eq!(
        node.node().get_input::<String>("in_string").unwrap(),
        "Ciao"
    );
    assert!(*node.node().get_input::<bool>("in_bool").unwrap());

    tree.add_node(
        registry
            .deserialize_node(&node.serialize_node())
            .expect("registered example node should deserialize"),
    );
    tree.add_default_node::<RustExampleNode>("NodeB");
    assert_eq!(tree.get_nodes_count(), 3);

    let bytes = serialize(&tree);
    let restored = deserialize::<NodeTree>(&bytes).expect("node tree should deserialize");
    assert_eq!(restored.get_nodes_count(), 3);
    assert_eq!(restored.get_links_count(), 4);

    let mut data = LogicData::from(restored);
    data.init();
    data.execute(&std::time::Duration::from_millis(30));
}
