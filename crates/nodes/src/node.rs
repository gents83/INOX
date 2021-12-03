use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::{InputPin, OutputPin, PinId};
use sabi_serialize::{generate_uid_from_string, Deserialize, Serialize, Uid};

pub type NodeId = Uid;

pub trait NodeTrait: Any + 'static {
    fn node(&self) -> &Node;
    fn node_mut(&mut self) -> &mut Node;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn id(&self) -> &NodeId {
        self.node().id()
    }
    fn name(&self) -> &str {
        self.node().name()
    }
    fn set_name(&mut self, name: &str) {
        self.node_mut().set_name(name)
    }
    fn category(&self) -> &str {
        self.node().category()
    }
    fn description(&self) -> &str {
        self.node().description()
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "sabi_serialize")]
pub struct Node {
    id: NodeId,
    name: String,
    category: String,
    description: String,
    inputs: HashMap<PinId, Box<dyn InputPin>>,
    outputs: HashMap<PinId, Box<dyn OutputPin>>,
}
impl Node {
    pub fn new(name: &str, category: &str, description: &str) -> Self {
        Self {
            id: generate_uid_from_string(name),
            name: String::from(name),
            category: String::from(category),
            description: String::from(description),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
        }
    }
    pub fn id(&self) -> &NodeId {
        &self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn set_name(&mut self, name: &str) {
        self.name = String::from(name);
        self.id = generate_uid_from_string(name);
    }
    pub fn category(&self) -> &str {
        &self.category
    }
    pub fn description(&self) -> &str {
        &self.description
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
        V: InputPin,
    {
        let uid = PinId::new(name);
        self.inputs.insert(uid, Box::new(value));
    }
    pub fn add_output<V>(&mut self, name: &str, value: V)
    where
        V: OutputPin,
    {
        let uid = PinId::new(name);
        self.outputs.insert(uid, Box::new(value));
    }
    pub fn get_input<V>(&self, name: &str) -> Option<&V>
    where
        V: InputPin,
    {
        let uid = PinId::new(name);
        if let Some(i) = self.inputs.get(&uid) {
            return i.as_any().downcast_ref::<V>();
        }
        None
    }
    pub fn get_output<V>(&self, name: &str) -> Option<&V>
    where
        V: OutputPin,
    {
        let uid = PinId::new(name);
        if let Some(o) = self.outputs.get(&uid) {
            return o.as_any().downcast_ref::<V>();
        }
        None
    }
    pub fn get_input_mut<V>(&mut self, name: &str) -> Option<&mut V>
    where
        V: 'static,
    {
        let uid = PinId::new(name);
        if let Some(i) = self.inputs.get_mut(&uid) {
            return i.as_any_mut().downcast_mut::<V>();
        }
        None
    }
    pub fn get_output_mut<V>(&mut self, name: &str) -> Option<&mut V>
    where
        V: 'static,
    {
        let uid = PinId::new(name);
        if let Some(o) = self.outputs.get_mut(&uid) {
            return o.as_any_mut().downcast_mut::<V>();
        }
        None
    }
}
