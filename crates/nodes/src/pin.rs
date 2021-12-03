use std::any::{Any, TypeId};

use sabi_serialize::{typetag, Deserialize, Serialize};

use crate::Node;

enum PinPort {
    Input = 0,
    Output = 1,
}
pub const INPUT_PIN: usize = PinPort::Input as usize;
pub const OUTPUT_PIN: usize = PinPort::Output as usize;

pub trait PinType: Send + Sync + 'static {
    fn type_id(&self) -> TypeId;
    fn is_pin_of_type(&self, type_id: std::any::TypeId) -> bool {
        self.type_id() == type_id
    }
    fn resolve_pin(&self, from_node: &Node, from_pin: &str, to_node: &mut Node, to_pin: &str);
}

pub trait Pin<const T: usize>: Any + Send + Sync + 'static {
    fn is_input(&self) -> bool {
        T == PinPort::Input as usize
    }
    fn is_output(&self) -> bool {
        T == PinPort::Output as usize
    }
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn get_type_id(&self) -> TypeId;
    fn get_type_name(&self) -> &'static str;
}

#[typetag::serde(tag = "InputPin")]
pub trait InputPin: Pin<INPUT_PIN> {}

#[typetag::serde(tag = "OutputPin")]
pub trait OutputPin: Pin<OUTPUT_PIN> {}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(crate = "sabi_serialize")]
pub struct PinId(String);
impl PinId {
    pub fn new(name: &str) -> Self {
        PinId(String::from(name))
    }
}
