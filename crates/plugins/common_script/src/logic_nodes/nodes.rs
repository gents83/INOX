use inox_math::{VecBase, Vector3};
use inox_nodes::{
    implement_node, LogicContext, LogicExecution, Node, NodeExecutionType, NodeState, PinId,
};
use inox_resources::Resource;
use inox_scene::{Object, Script};
use inox_serialize::{inox_serializable, Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "inox_serialize")]
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
        //println!("Rotating of [{:?}] degrees", rotation);
        rotation.x = rotation.x.to_radians();
        rotation.y = rotation.y.to_radians();
        rotation.z = rotation.z.to_radians();
        if let Some(object) = context.get_with_name::<Resource<Object>>(Script::LOGIC_OBJECT) {
            object.get_mut().rotate(rotation * context.dt.as_secs_f32());
        } else {
            eprintln!("Unable to find {} in LogicContext", Script::LOGIC_OBJECT);
        }
    }
}
