use sabi_resources::Singleton;
use sabi_serialize::{deserialize, serialize, Deserialize, Serialize};

use crate::{Node, NodeTrait, PinType};

pub trait NodeType: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn category(&self) -> &str;
    fn description(&self) -> &str;
    fn serialize(&self) -> String;
    fn deserialize(&self, data: &str) -> Option<Box<dyn NodeTrait>>;
}

struct SpecificNodeType<N> {
    n: N,
    marker: std::marker::PhantomData<N>,
}
impl<N> NodeType for SpecificNodeType<N>
where
    N: NodeTrait + Serialize + Default + for<'de> Deserialize<'de> + 'static,
{
    fn name(&self) -> &str {
        self.n.name()
    }
    fn category(&self) -> &str {
        self.n.category()
    }
    fn description(&self) -> &str {
        self.n.description()
    }
    fn serialize(&self) -> String {
        serialize(&self.n)
    }
    fn deserialize(&self, data: &str) -> Option<Box<dyn NodeTrait>>
    where
        Self: Sized,
    {
        if let Ok(n) = deserialize::<N>(data) {
            Some(Box::new(n))
        } else {
            None
        }
    }
}
unsafe impl<N> Send for SpecificNodeType<N> where N: NodeTrait + Serialize + Default {}
unsafe impl<N> Sync for SpecificNodeType<N> where N: NodeTrait + Serialize + Default {}

#[derive(Default)]
pub struct LogicNodeRegistry {
    pin_types: Vec<Box<dyn PinType>>,
    node_types: Vec<Box<dyn NodeType>>,
}
impl Singleton for LogicNodeRegistry {}

impl LogicNodeRegistry {
    pub fn register_pin_type<V>(&mut self)
    where
        V: PinType + Default + 'static,
    {
        self.pin_types.push(Box::new(V::default()));
    }
    pub fn register_node<N>(&mut self)
    where
        N: NodeTrait + Default + Serialize + for<'de> Deserialize<'de> + 'static,
    {
        let n = N::default();
        self.node_types.push(Box::new(SpecificNodeType::<N> {
            n,
            marker: std::marker::PhantomData::<N>::default(),
        }));
    }
    pub fn for_each_node<F>(&self, mut f: F)
    where
        F: FnMut(&dyn NodeType),
    {
        for node in &self.node_types {
            f(node.as_ref());
        }
    }
    pub fn deserialize(&self, data: &str) -> Option<Box<dyn NodeTrait>> {
        for node in &self.node_types {
            if let Some(n) = node.deserialize(data) {
                return Some(n);
            }
        }
        None
    }
    pub fn resolve_pin(&self, from_node: &Node, from_pin: &str, to_node: &mut Node, to_pin: &str) {
        let from_type = from_node.get_pin_type_id(from_pin);
        let to_type = to_node.get_pin_type_id(to_pin);
        if from_type != to_type {
            eprintln!(
                "Trying to convert pin type {} to {} from Node {} pin {} to Node {} pin {}",
                from_node.get_pin_type_name(from_pin),
                to_node.get_pin_type_name(to_pin),
                from_node.name(),
                from_pin,
                to_node.name(),
                to_pin
            );
            return;
        }
        for pin_type in &self.pin_types {
            let type_id = pin_type.type_id();
            if type_id == from_type && type_id == to_type {
                return pin_type.resolve_pin(from_node, from_pin, to_node, to_pin);
            }
        }
        eprintln!(
            "Impossible to resolve Pin type {} to {} from Node {} pin {} to Node {} pin {}",
            from_node.get_pin_type_name(from_pin),
            to_node.get_pin_type_name(to_pin),
            from_node.name(),
            from_pin,
            to_node.name(),
            to_pin
        );
    }
}
