use inox_serialize::{Deserialize, Serialize};

use crate::{LogicNodeRegistry, Node};

#[derive(Default, Serialize, Deserialize, Clone)]
#[serde(crate = "inox_serialize")]
pub struct NodeLink {
    from_node: String,
    to_node: String,
    from_pin: String,
    to_pin: String,
}

impl NodeLink {
    pub fn new(from_node: &str, to_node: &str, from_pin: &str, to_pin: &str) -> Self {
        Self {
            from_node: String::from(from_node),
            to_node: String::from(to_node),
            from_pin: String::from(from_pin),
            to_pin: String::from(to_pin),
        }
    }
    pub fn from_node(&self) -> &str {
        &self.from_node
    }
    pub fn to_node(&self) -> &str {
        &self.to_node
    }
    pub fn from_pin(&self) -> &str {
        &self.from_pin
    }
    pub fn to_pin(&self) -> &str {
        &self.to_pin
    }
    pub fn resolve(&self, registry: &LogicNodeRegistry, from_node: &Node, to_node: &mut Node) {
        registry.resolve_pin(from_node, &self.from_pin, to_node, &self.to_pin);
    }
}
