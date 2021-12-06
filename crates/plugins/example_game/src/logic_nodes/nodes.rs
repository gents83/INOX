use sabi_nodes::{
    implement_node, LogicData, LogicExecution, LogicNodeRegistry, Node, NodeExecutionType,
    NodeState, NodeTrait, NodeTree, PinId, ScriptInitNode,
};
use sabi_serialize::{typetag, Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "sabi_serialize")]
pub struct RotateNode {
    node: Node,
    #[serde(skip)]
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
    pub fn on_update(&mut self, pin: &PinId) -> NodeState {
        if *pin == PinId::new("OnImpulse") {
            self.rotate();
            return NodeState::Executed(None);
        } else if *pin == PinId::new("Start") {
            self.is_running = true;
        } else if *pin == PinId::new("Stop") {
            self.is_running = false;
            return NodeState::Executed(None);
        }
        if self.is_running {
            self.rotate();
        }
        NodeState::Running(None)
    }

    fn rotate(&self) {
        println!(
            "Rotating of [{:?}, {}, {}] degrees",
            self.node.get_input::<f32>("X (in degrees)").unwrap(),
            self.node.get_input::<f32>("Y (in degrees)").unwrap(),
            self.node.get_input::<f32>("Z (in degrees)").unwrap()
        );
    }
}

#[allow(dead_code)]
fn test_nodes() {
    let mut registry = LogicNodeRegistry::default();
    registry.register_node::<ScriptInitNode>();
    registry.register_node::<RotateNode>();

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
    tree.add_link("ScriptInitNode", "RotateNode", "Execute", "Start");
    assert_eq!(tree.get_links_count(), 1);

    tree.add_default_node::<ScriptInitNode>("ScriptInitNode");
    assert_eq!(tree.get_nodes_count(), 1);

    let mut rotate_node = RotateNode::default();
    if let Some(v) = rotate_node
        .node_mut()
        .get_input_mut::<f32>("Y (in degrees)")
    {
        *v = 10.;
    }
    assert_eq!(
        *rotate_node
            .node()
            .get_input::<f32>("Y (in degrees)")
            .unwrap(),
        10.
    );
    tree.add_node(Box::new(rotate_node));
    assert_eq!(tree.get_nodes_count(), 2);

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
