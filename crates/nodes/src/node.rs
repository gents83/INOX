use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::{LogicContext, Pin, PinId};
use sabi_serialize::*;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum NodeExecutionType {
    OnDemand, //default
    OneShot,
    Continuous,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum NodeState {
    Active,
    Running(Option<Vec<PinId>>),
    Executed(Option<Vec<PinId>>),
}
impl Default for NodeState {
    fn default() -> Self {
        NodeState::Active
    }
}

pub type NodeId = Uid;

#[serializable_trait]
pub trait NodeTrait: Serializable + Any + Send + Sync + 'static {
    fn get_type() -> &'static str
    where
        Self: Sized;
    fn category() -> &'static str
    where
        Self: Sized;
    fn description() -> &'static str
    where
        Self: Sized;
    fn node(&self) -> &Node;
    fn node_mut(&mut self) -> &mut Node;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn id(&self) -> NodeId {
        self.node().id()
    }
    fn name(&self) -> &str {
        self.node().name()
    }
    fn set_name(&mut self, name: &str) {
        self.node_mut().set_name(name)
    }
    fn execytion_type(&self) -> NodeExecutionType;
    fn execute(&mut self, pin: &PinId, context: &LogicContext) -> NodeState;
    fn duplicate_node(&self) -> Box<dyn NodeTrait>;
    fn serialize_node(&self, serializable_registry: &SerializableRegistry) -> String;
    fn deserialize_node(&self, s: &str) -> Option<Self>
    where
        Self: Sized;
}
impl Clone for Box<dyn NodeTrait> {
    fn clone(&self) -> Box<dyn NodeTrait> {
        self.duplicate_node()
    }
}

#[derive(Serializable, Clone)]
pub struct Node {
    name: String,
    inputs: HashMap<PinId, Box<dyn Pin>>,
    outputs: HashMap<PinId, Box<dyn Pin>>,
}
impl Node {
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
        }
    }
    pub fn inputs(&self) -> &HashMap<PinId, Box<dyn Pin>> {
        &self.inputs
    }
    pub fn outputs(&self) -> &HashMap<PinId, Box<dyn Pin>> {
        &self.outputs
    }
    pub fn inputs_mut(&mut self) -> &mut HashMap<PinId, Box<dyn Pin>> {
        &mut self.inputs
    }
    pub fn outputs_mut(&mut self) -> &mut HashMap<PinId, Box<dyn Pin>> {
        &mut self.outputs
    }
    pub fn id(&self) -> NodeId {
        generate_uid_from_string(&self.name)
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn set_name(&mut self, name: &str) {
        self.name = String::from(name);
    }
    pub fn get_pin_type_name(&self, name: &str) -> &str {
        let uid = PinId::new(name);
        if let Some(input) = self.inputs.get(&uid) {
            input.get_type_name()
        } else if let Some(output) = self.outputs.get(&uid) {
            output.get_type_name()
        } else {
            eprintln!(
                "Trying to get a pin {} that doesn't exist in node {}",
                name,
                self.name()
            );
            ""
        }
    }
    pub fn get_pin_type_id(&self, name: &str) -> TypeId {
        let uid = PinId::new(name);
        if let Some(input) = self.inputs.get(&uid) {
            input.get_type_id()
        } else if let Some(output) = self.outputs.get(&uid) {
            output.get_type_id()
        } else {
            eprintln!(
                "Trying to get a pin {} that doesn't exist in node {}",
                name,
                self.name()
            );
            TypeId::of::<()>()
        }
    }
    pub fn add_input<V>(&mut self, name: &str, value: V)
    where
        V: Pin,
    {
        let uid = PinId::new(name);
        self.inputs.insert(uid, Box::new(value));
    }
    pub fn add_output<V>(&mut self, name: &str, value: V)
    where
        V: Pin,
    {
        let uid = PinId::new(name);
        self.outputs.insert(uid, Box::new(value));
    }
    pub fn input<V>(&self, pin_id: &PinId) -> Option<&V>
    where
        V: Pin,
    {
        self.inputs
            .get(pin_id)
            .and_then(|i| i.as_any().downcast_ref::<V>())
    }
    pub fn output<V>(&self, pin_id: &PinId) -> Option<&V>
    where
        V: Pin,
    {
        self.outputs
            .get(pin_id)
            .and_then(|o| o.as_any().downcast_ref::<V>())
    }
    pub fn input_mut<V>(&mut self, pin_id: &PinId) -> Option<&mut V>
    where
        V: 'static,
    {
        self.inputs
            .get_mut(pin_id)
            .and_then(|i| i.as_any_mut().downcast_mut::<V>())
    }
    pub fn output_mut<V>(&mut self, pin_id: &PinId) -> Option<&mut V>
    where
        V: 'static,
    {
        self.outputs
            .get_mut(pin_id)
            .and_then(|o| o.as_any_mut().downcast_mut::<V>())
    }
    pub fn get_input<V>(&self, name: &str) -> Option<&V>
    where
        V: Pin,
    {
        let uid = PinId::new(name);
        self.input::<V>(&uid)
    }
    pub fn get_output<V>(&self, name: &str) -> Option<&V>
    where
        V: Pin,
    {
        let uid = PinId::new(name);
        self.output::<V>(&uid)
    }
    pub fn get_input_mut<V>(&mut self, name: &str) -> Option<&mut V>
    where
        V: 'static,
    {
        let uid = PinId::new(name);
        self.input_mut::<V>(&uid)
    }
    pub fn get_output_mut<V>(&mut self, name: &str) -> Option<&mut V>
    where
        V: 'static,
    {
        let uid = PinId::new(name);
        self.output_mut::<V>(&uid)
    }
    pub fn has_same_pins(&self, node: &Node) -> bool {
        let mut same_inputs = true;
        let mut same_outputs = true;
        for (uid, input) in &self.inputs {
            if let Some(other_input) = node.inputs.get(uid) {
                if input.get_type_id() != other_input.get_type_id() {
                    same_inputs = false;
                }
            } else {
                same_inputs = false;
            }
        }
        for (uid, output) in &self.outputs {
            if let Some(other_output) = node.outputs.get(uid) {
                if output.get_type_id() != other_output.get_type_id() {
                    same_outputs = false;
                }
            } else {
                same_outputs = false;
            }
        }
        same_inputs && same_outputs
    }
    pub fn has_input<V>(&self) -> bool
    where
        V: Pin,
    {
        self.inputs
            .iter()
            .any(|(_, i)| i.as_any().downcast_ref::<V>().is_some())
    }
    pub fn has_output<V>(&self) -> bool
    where
        V: Pin,
    {
        self.outputs
            .iter()
            .any(|(_, i)| i.as_any().downcast_ref::<V>().is_some())
    }
    pub fn is_input<V>(&self, pin_id: &PinId) -> bool
    where
        V: Pin,
    {
        self.inputs
            .get(pin_id)
            .map_or(false, |i| i.as_any().downcast_ref::<V>().is_some())
    }
    pub fn is_output<V>(&self, pin_id: &PinId) -> bool
    where
        V: Pin,
    {
        self.outputs
            .get(pin_id)
            .map_or(false, |o| o.as_any().downcast_ref::<V>().is_some())
    }
    pub fn pass_value<V>(&mut self, input_name: &str, output_name: &str)
    where
        V: Pin + Clone,
    {
        let input_uid = PinId::new(input_name);
        let output_uid = PinId::new(output_name);
        let input = self.inputs.get(&input_uid);
        let output = self.outputs.get_mut(&output_uid);
        if let (Some(output), Some(input)) = (output, input) {
            let i = input.as_any().downcast_ref::<V>().unwrap();
            let o = output.as_any_mut().downcast_mut::<V>().unwrap();
            *o = i.clone();
        }
    }
    pub fn resolve(&mut self, input: &PinId, from_node: &Node, from_pin: &PinId) {
        if let Some(i) = self.inputs.get_mut(input) {
            i.copy_from(from_node, from_pin);
        }
    }
}
