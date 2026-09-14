use inox_log::debug_log;
use inox_serialize::{Deserialize, Serialize};

use crate::{
    implement_node, implement_pin, LogicContext, Node, NodeExecutionType, NodeState, NodeTrait,
    PinId,
};
use inox_serialize::inox_serializable;

#[derive(Default, Serialize, Deserialize, Copy, Clone)]
#[serde(crate = "inox_serialize")]
pub enum LogicExecution {
    #[default]
    Type,
}
implement_pin!(LogicExecution);

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "inox_serialize")]
pub struct RustExampleNode {
    node: Node,
}
implement_node!(
    RustExampleNode,
    node,
    "Example",
    "Rust example node",
    NodeExecutionType::OnDemand
);
impl Default for RustExampleNode {
    fn default() -> Self {
        let mut node = Node::new(stringify!(RustExampleNode));
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
impl RustExampleNode {
    pub fn on_update(&mut self, pin: &PinId, _context: &LogicContext) -> NodeState {
        if *pin == PinId::new("in_execute") {
            debug_log!("Executing {}", self.name());
            debug_log!("in_int {}", self.node().get_input::<i32>("in_int").unwrap());
            debug_log!(
                "in_float {}",
                self.node().get_input::<f32>("in_float").unwrap()
            );
            debug_log!(
                "in_string {}",
                self.node().get_input::<String>("in_string").unwrap()
            );
            debug_log!(
                "in_bool {}",
                self.node().get_input::<bool>("in_bool").unwrap()
            );

            self.node_mut().pass_value::<i32>("in_int", "out_int");
            self.node_mut().pass_value::<f32>("in_float", "out_float");
            self.node_mut()
                .pass_value::<String>("in_string", "out_string");
            self.node_mut().pass_value::<bool>("in_bool", "out_bool");
            NodeState::Executed(Some(vec![PinId::new("out_execute")]))
        } else {
            panic!("Trying to execute through an unexpected pin {}", pin.name());
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "inox_serialize")]
pub struct ScriptInitNode {
    node: Node,
}
implement_node!(
    ScriptInitNode,
    node,
    "Init",
    "Script init node",
    NodeExecutionType::OneShot
);
impl Default for ScriptInitNode {
    fn default() -> Self {
        let mut node = Node::new(stringify!(ScriptInitNode));
        node.add_output("Execute", LogicExecution::default());
        Self { node }
    }
}
impl ScriptInitNode {
    pub fn on_update(&mut self, pin: &PinId, _context: &LogicContext) -> NodeState {
        debug_assert!(*pin == PinId::invalid());

        debug_log!("Executing {}", self.name());
        NodeState::Executed(Some(vec![PinId::new("Execute")]))
    }
}
