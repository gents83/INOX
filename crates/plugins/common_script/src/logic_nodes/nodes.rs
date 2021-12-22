use sabi_math::{VecBase, Vector3};
use sabi_nodes::{
    implement_node, LogicContext, LogicData, LogicExecution, LogicNodeRegistry, Node,
    NodeExecutionType, NodeState, NodeTrait, NodeTree, PinId, SerializableNodeTrait,
};
use sabi_resources::{Resource, SharedDataRc};
use sabi_scene::{Object, Script};
use sabi_serialize::*;

#[derive(Serializable, Clone)]
#[serializable(NodeTrait)]
pub struct RotateNode {
    node: Node,
    #[serializable(ignore)]
    is_running: bool,
}
implement_node!(
    RotateNode,
    node,
    "Object",
    "Rotate",
    NodeExecutionType::OnDemand
);
impl Default for RotateNode {
    fn default() -> Self {
        let mut node = Node::new(stringify!(RotateNode));
        node.add_input("OnImpulse", LogicExecution::default());
        node.add_input("Start", LogicExecution::default());
        node.add_input("Stop", LogicExecution::default());
        node.add_input::<f32>("X (in degrees)", 0.);
        node.add_input::<f32>("Y (in degrees)", 0.);
        node.add_input::<f32>("Z (in degrees)", 0.);
        Self {
            node,
            is_running: false,
        }
    }
}
impl RotateNode {
    pub fn on_update(&mut self, pin: &PinId, context: &LogicContext) -> NodeState {
        if *pin == PinId::new("OnImpulse") {
            self.rotate(context);
            return NodeState::Executed(None);
        } else if *pin == PinId::new("Start") {
            self.is_running = true;
        } else if *pin == PinId::new("Stop") {
            self.is_running = false;
            return NodeState::Executed(None);
        }
        if self.is_running {
            self.rotate(context);
        }
        NodeState::Running(None)
    }

    fn rotate(&self, context: &LogicContext) {
        let mut rotation = Vector3::default_zero();
        rotation.x = *self.node.get_input::<f32>("X (in degrees)").unwrap();
        rotation.y = *self.node.get_input::<f32>("Y (in degrees)").unwrap();
        rotation.z = *self.node.get_input::<f32>("Z (in degrees)").unwrap();
        println!("Rotating of [{:?}] degrees", rotation);
        rotation.x = rotation.x.to_radians();
        rotation.y = rotation.y.to_radians();
        rotation.z = rotation.z.to_radians();
        if let Some(object) = context.get_with_name::<Resource<Object>>(Script::LOGIC_OBJECT) {
            object.get_mut().rotate(rotation);
        } else {
            eprintln!("Unable to find {} in LogicContext", Script::LOGIC_OBJECT);
        }
    }
}

#[allow(dead_code)]
fn test_nodes() {
    let shared_data = SharedDataRc::default();
    let mut registry = LogicNodeRegistry::default();
    registry.on_create(&shared_data);
    registry.register_node::<RotateNode>(&shared_data);

    let data = r#"{"nodes": [{"node_type": "ScriptInitNode", "node": {"name": "ScriptInitNode", "inputs": {}, "outputs": {"Execute": {"pin_type": "LogicExecution", "Type": null}}}}, {"node_type": "RotateNode", "node": {"name": "RotateNode", "inputs": {"OnImpulse": {"pin_type": "LogicExecution", "Type": null}, "Stop": {"pin_type": "LogicExecution", "Type": null}, "Start": {"pin_type": "LogicExecution", "Type": null}, "X (in degrees)": {"pin_type": "f32", "value": 0.0}, "Y (in degrees)": {"pin_type": "f32", "value": 1.0}, "Z (in degrees)": {"pin_type": "f32", "value": 0.0}}, "outputs": {}}}], "links": [{"from_node": "ScriptInitNode", "to_node": "RotateNode", "from_pin": "Execute", "to_pin": "Start"}]}"#;
    let tree = deserialize::<NodeTree>(data, &shared_data.serializable_registry()).unwrap();
    assert_eq!(tree.get_nodes_count(), 2);
    assert_eq!(tree.get_links_count(), 1);

    let mut logic_data = LogicData::from(tree);
    logic_data.init();
    logic_data.execute();
    logic_data.execute();
    logic_data.execute();
    logic_data.execute();
    logic_data.execute();
}

#[test]
fn test_nodes_fn() {
    test_nodes()
}
