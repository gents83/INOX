use std::any::{Any, TypeId};

use sabi_serialize::{typetag, Deserialize, Serialize};

use crate::Node;

pub trait PinType: Send + Sync + 'static {
    fn type_id(&self) -> TypeId;
    fn is_pin_of_type(&self, type_id: std::any::TypeId) -> bool {
        self.type_id() == type_id
    }
    fn resolve_pin(&self, from_node: &Node, from_pin: &str, to_node: &mut Node, to_pin: &str);
    fn copy_from(&mut self, node: &Node, output_pin: &PinId);
}

#[typetag::serde(tag = "pin_type")]
pub trait Pin: Any + PinType + Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn get_type_id(&self) -> TypeId;
    fn get_type_name(&self) -> &'static str;
    fn duplicate(&self) -> Box<dyn Pin>;
}
impl Clone for Box<dyn Pin> {
    fn clone(&self) -> Box<dyn Pin> {
        self.duplicate()
    }
}

#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(crate = "sabi_serialize")]
pub struct PinId(String);
impl PinId {
    pub fn new(name: &str) -> Self {
        PinId(String::from(name))
    }
    pub fn name(&self) -> &str {
        &self.0
    }
    pub fn invalid() -> Self {
        PinId(String::new())
    }
}
