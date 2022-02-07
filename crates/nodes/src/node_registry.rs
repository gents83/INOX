use inox_resources::implement_singleton;
use inox_serialize::{inox_serializable::SerializableRegistryRc, Deserialize, Serialize};

use crate::{Node, NodeTrait, Pin, PinType};

pub trait NodeType: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn category(&self) -> &str;
    fn description(&self) -> &str;
    fn serialize_node(&self, registry: &SerializableRegistryRc) -> String;
    fn deserialize_node(
        &self,
        data: &str,
        registry: &SerializableRegistryRc,
    ) -> Option<Box<dyn NodeTrait + Send + Sync>>;
}

struct SpecificNodeType<N> {
    n: N,
    category: String,
    description: String,
    marker: std::marker::PhantomData<N>,
}
impl<N> NodeType for SpecificNodeType<N>
where
    N: NodeTrait + Serialize + Default + for<'de> Deserialize<'de> + 'static + Sized,
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
    fn serialize_node(&self, registry: &SerializableRegistryRc) -> String {
        self.n.serialize_node(registry)
    }
    fn deserialize_node(
        &self,
        data: &str,
        registry: &SerializableRegistryRc,
    ) -> Option<Box<dyn NodeTrait + Send + Sync>>
    where
        Self: Sized,
    {
        if let Some(n) = self.n.deserialize_node(data, registry) {
            if self.n.node().has_same_pins(n.node()) {
                return Some(Box::new(n) as Box<dyn NodeTrait + Send + Sync>);
            } else {
                return None;
            }
        }
        None
    }
}
unsafe impl<N> Send for SpecificNodeType<N> where N: NodeTrait + Serialize + Default {}
unsafe impl<N> Sync for SpecificNodeType<N> where N: NodeTrait + Serialize + Default {}

pub struct LogicNodeRegistry {
    serializable_registry: SerializableRegistryRc,
    pin_types: Vec<Box<dyn PinType>>,
    node_types: Vec<Box<dyn NodeType>>,
}
implement_singleton!(LogicNodeRegistry);

impl LogicNodeRegistry {
    pub fn new(serializable_registry: &SerializableRegistryRc) -> Self {
        Self {
            serializable_registry: serializable_registry.clone(),
            pin_types: Vec::new(),
            node_types: Vec::new(),
        }
    }
    pub fn register_pin_type<V>(&mut self)
    where
        V: Pin + PinType + Default + 'static,
    {
        let p = V::default();
        self.pin_types.push(Box::new(p));

        V::register_as_serializable(&self.serializable_registry);
    }
    pub fn unregister_pin_type<V>(&mut self)
    where
        V: Pin + PinType + Default + 'static,
    {
        let p = V::default();
        self.pin_types.retain(|v| v.name() != p.name());

        V::unregister_as_serializable(&self.serializable_registry);
    }
    pub fn register_node<N>(&mut self)
    where
        N: NodeTrait + Default + Serialize + for<'de> Deserialize<'de> + 'static,
    {
        let n = N::default();
        self.node_types.push(Box::new(SpecificNodeType::<N> {
            n,
            category: String::from(N::category()),
            description: String::from(N::description()),
            marker: std::marker::PhantomData::<N>::default(),
        }));

        N::register_as_serializable(&self.serializable_registry);
    }
    pub fn unregister_node<N>(&mut self)
    where
        N: NodeTrait + Default + Serialize + for<'de> Deserialize<'de> + 'static,
    {
        let n = N::default();
        self.node_types.retain(|v| v.name() != n.name());

        N::unregister_as_serializable(&self.serializable_registry);
    }
    pub fn for_each_node<F>(&self, mut f: F)
    where
        F: FnMut(&dyn NodeType, &SerializableRegistryRc),
    {
        for node in &self.node_types {
            f(node.as_ref(), &self.serializable_registry);
        }
    }
    pub fn deserialize_node(&self, data: &str) -> Option<Box<dyn NodeTrait + Send + Sync>> {
        for node in &self.node_types {
            if let Some(n) = node.deserialize_node(data, &self.serializable_registry) {
                println!("Deserializing as {}", n.serializable_name());
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
