use sabi_resources::Singleton;
use sabi_serialize::{serialize, Serialize};

use super::logic_nodes::Node;

pub trait NodeType: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn base_type(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn create_serialized(&self) -> String;
}

struct SpecificNodeType<N> {
    marker: std::marker::PhantomData<N>,
}
impl<N> NodeType for SpecificNodeType<N>
where
    N: Node + Serialize + Default,
{
    fn create_serialized(&self) -> String {
        serialize(&N::default())
    }
    fn name(&self) -> &'static str {
        N::node_type()
    }
    fn base_type(&self) -> &'static str {
        N::base_type()
    }
    fn description(&self) -> &'static str {
        N::description()
    }
}
unsafe impl<N> Send for SpecificNodeType<N> where N: Node + Serialize + Default {}
unsafe impl<N> Sync for SpecificNodeType<N> where N: Node + Serialize + Default {}

#[derive(Default)]
pub struct LogicNodeRegistry {
    nodes: Vec<Box<dyn NodeType>>,
}
impl Singleton for LogicNodeRegistry {}

impl LogicNodeRegistry {
    pub fn register_node<N>(&mut self)
    where
        N: Node + Serialize + Default,
    {
        self.nodes.push(Box::new(SpecificNodeType::<N> {
            marker: std::marker::PhantomData::<N>::default(),
        }));
    }
    pub fn for_each_node<F>(&self, mut f: F)
    where
        F: FnMut(&dyn NodeType),
    {
        for node in &self.nodes {
            f(node.as_ref());
        }
    }
}
