use std::any::{type_name, Any, TypeId};

use sabi_serialize::*;

use crate::Node;

pub trait PinType: Send + Sync + 'static {
    fn name(&self) -> &'static str {
        type_name::<Self>()
            .split(':')
            .collect::<Vec<&str>>()
            .last()
            .unwrap()
    }
    fn type_id(&self) -> TypeId;
    fn is_pin_of_type(&self, type_id: std::any::TypeId) -> bool {
        self.type_id() == type_id
    }
    fn resolve_pin(&self, from_node: &Node, from_pin: &str, to_node: &mut Node, to_pin: &str);
    fn copy_from(&mut self, node: &Node, output_pin: &PinId);
}

#[serializable_trait]
pub trait Pin: Serializable + Any + PinType + Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn get_type_id(&self) -> TypeId;
    fn get_type_name(&self) -> &'static str;
    fn duplicate_pin(&self) -> Box<dyn Pin>;
}
impl Clone for Box<dyn Pin> {
    fn clone(&self) -> Box<dyn Pin> {
        self.duplicate_pin()
    }
}

#[derive(Default, Serializable, PartialEq, Eq, Hash, Clone)]
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
