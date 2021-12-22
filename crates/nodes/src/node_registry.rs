use sabi_resources::{implement_singleton, SharedDataRc};
use sabi_serialize::*;

use crate::{
    LogicData, LogicExecution, Node, NodeLink, NodeTrait, NodeTree, Pin, PinId, PinType,
    RustExampleNode, ScriptInitNode,
};

pub trait NodeType: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn category(&self) -> &str;
    fn description(&self) -> &str;
    fn serialize_node(&self, serializable_registry: &SerializableRegistry) -> String;
    fn deserialize_node(&self, data: &str) -> Option<Box<dyn NodeTrait>>;
}

struct SpecificNodeType<N> {
    n: N,
    category: String,
    description: String,
    marker: std::marker::PhantomData<N>,
}
impl<N> NodeType for SpecificNodeType<N>
where
    N: NodeTrait + Serializable + Default + 'static + Sized,
{
    fn name(&self) -> &str {
        self.n.name()
    }
    fn category(&self) -> &str {
        &self.category
    }
    fn description(&self) -> &str {
        &self.description
    }
    fn serialize_node(&self, serializable_registry: &SerializableRegistry) -> String {
        self.n.serialize_node(serializable_registry)
    }
    fn deserialize_node(&self, data: &str) -> Option<Box<dyn NodeTrait>>
    where
        Self: Sized,
    {
        if let Some(n) = self.n.deserialize_node(data) {
            if self.n.node().has_same_pins(n.node()) {
                return Some(Box::new(n) as Box<dyn NodeTrait>);
            } else {
                return None;
            }
        }
        None
    }
}
unsafe impl<N> Send for SpecificNodeType<N> where N: NodeTrait + Serializable + Default {}
unsafe impl<N> Sync for SpecificNodeType<N> where N: NodeTrait + Serializable + Default {}

#[derive(Default)]
pub struct LogicNodeRegistry {
    pin_types: Vec<Box<dyn PinType>>,
    node_types: Vec<Box<dyn NodeType>>,
}
implement_singleton!(LogicNodeRegistry, on_create);

impl LogicNodeRegistry {
    pub fn on_create(&mut self, shared_data: &SharedDataRc) {
        shared_data.register_serializable_trait::<dyn Pin>();
        shared_data.register_serializable_trait::<dyn NodeTrait>();
        shared_data.register_serializable_type::<PinId>();
        shared_data.register_serializable_type::<NodeLink>();
        shared_data.register_serializable_type::<Node>();
        shared_data.register_serializable_type::<NodeTree>();
        shared_data.register_serializable_type::<LogicData>();

        //Registering basic types
        self.register_pin_type::<f32>(shared_data);
        self.register_pin_type::<f64>(shared_data);
        self.register_pin_type::<u8>(shared_data);
        self.register_pin_type::<i8>(shared_data);
        self.register_pin_type::<u16>(shared_data);
        self.register_pin_type::<i16>(shared_data);
        self.register_pin_type::<u32>(shared_data);
        self.register_pin_type::<i32>(shared_data);
        self.register_pin_type::<bool>(shared_data);
        self.register_pin_type::<String>(shared_data);
        self.register_pin_type::<LogicExecution>(shared_data);

        //Registering default nodes
        self.register_node::<RustExampleNode>(shared_data);
        self.register_node::<ScriptInitNode>(shared_data);
    }

    pub fn register_pin_type<V>(&mut self, shared_data: &SharedDataRc)
    where
        V: Pin
            + Default
            + 'static
            + Serializable
            + TypeInfo
            + FromSerializable
            + AsSerializable<dyn Pin>,
    {
        shared_data.register_serializable_type_with_trait::<dyn Pin, V>();

        let p = V::default();
        println!("Registering pin type: {}", p.name());
        self.pin_types.push(Box::new(p));
    }
    pub fn register_node<N>(&mut self, shared_data: &SharedDataRc)
    where
        N: NodeTrait
            + Default
            + 'static
            + Serializable
            + TypeInfo
            + FromSerializable
            + AsSerializable<dyn NodeTrait>,
    {
        shared_data.register_serializable_type_with_trait::<dyn NodeTrait, N>();

        let n = N::default();
        println!("Registering node: {}", n.name());
        self.node_types.push(Box::new(SpecificNodeType::<N> {
            n,
            category: String::from(N::category()),
            description: String::from(N::description()),
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
    pub fn deserialize_node(&self, data: &str) -> Option<Box<dyn NodeTrait>> {
        for node in &self.node_types {
            if let Some(n) = node.deserialize_node(data) {
                println!("Deserializing as {}", n.name());
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
