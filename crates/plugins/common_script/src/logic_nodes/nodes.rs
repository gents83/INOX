use sabi_math::{VecBase, Vector3};
use sabi_nodes::{
    implement_node, LogicContext, LogicData, LogicExecution, LogicNodeRegistry, Node,
    NodeExecutionType, NodeState, NodeTrait, NodeTree, PinId, ScriptInitNode,
    SerializableNodeTrait,
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

    let mut tree = NodeTree::default();
    tree.add_default_node::<ScriptInitNode>("ScriptInitNode");
    tree.add_default_node::<RotateNode>("RotateNode");
    let node = tree.find_node_mut("RotateNode").unwrap().node_mut();
    if let Some(v) = node.get_input_mut::<f32>("Y (in degrees)") {
        *v = 1.;
    }
    //tree.add_link("ScriptInitNode", "RotateNode", "Execute", "Start");
    tree.add_link("ScriptInitNode", "RotateNode", "Execute", "OnImpulse");

    let data = serialize(&tree, &shared_data.serializable_registry());
    //println!("{}", data);
    let tree = deserialize::<NodeTree>(&data, &shared_data.serializable_registry()).unwrap();
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
