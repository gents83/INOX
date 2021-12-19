use crate::{LogicNodeRegistry, NodeLink, NodeTrait};
use sabi_serialize::*;

#[derive(Default, Serializable, Clone)]
pub struct NodeTree {
    nodes: Vec<Box<dyn NodeTrait>>,
    links: Vec<NodeLink>,
}

impl SerializeFile for NodeTree {
    fn extension() -> &'static str {
        "node_tree"
    }
}

impl NodeTree {
    pub fn add_node(&mut self, node: Box<dyn NodeTrait>) {
        if self.find_node(node.name()).is_none() {
            self.nodes.push(node);
        }
    }
    pub fn add_default_node<T>(&mut self, name: &str)
    where
        T: NodeTrait + Default + Serializable + 'static,
    {
        if self.find_node(name).is_none() {
            let mut node = Box::new(T::default());
            node.set_name(name);
            self.nodes.push(node);
        }
    }
    pub fn find_node(&self, name: &str) -> Option<&dyn NodeTrait> {
        let uid = generate_uid_from_string(name);
        self.nodes
            .iter()
            .find(|n| n.id() == uid)
            .map(|n| n.as_ref())
    }
    pub fn find_node_mut(&mut self, name: &str) -> Option<&mut dyn NodeTrait> {
        let uid = generate_uid_from_string(name);
        self.nodes
            .iter_mut()
            .find(|n| n.id() == uid)
            .map(|n| n.as_mut())
    }
    pub fn find_node_as<T>(&self, name: &str) -> Option<&T>
    where
        T: NodeTrait + Default + Serializable + 'static,
    {
        let uid = generate_uid_from_string(name);
        self.nodes
            .iter()
            .find(|n| n.id() == uid)
            .map(|n| n.as_any().downcast_ref::<T>().unwrap())
    }
    pub fn find_node_mut_as<T>(&mut self, name: &str) -> Option<&mut T>
    where
        T: NodeTrait + Default + Serializable + 'static,
    {
        let uid = generate_uid_from_string(name);
        self.nodes
            .iter_mut()
            .find(|n| n.id() == uid)
            .map(|n| n.as_any_mut().downcast_mut::<T>().unwrap())
    }
    pub fn get_nodes_count(&self) -> usize {
        self.nodes.len()
    }
    pub fn add_link(&mut self, from_node: &str, to_node: &str, from_pin: &str, to_pin: &str) {
        self.links
            .push(NodeLink::new(from_node, to_node, from_pin, to_pin));
    }
    pub fn get_links_from(&self, from_node: &str) -> Vec<&NodeLink> {
        self.links
            .iter()
            .filter(|link| link.from_node() == from_node)
            .collect()
    }
    pub fn get_links_from_pin(&self, from_node: &str, from_pin: &str) -> Vec<&NodeLink> {
        self.links
            .iter()
            .filter(|link| link.from_node() == from_node && link.from_pin() == from_pin)
            .collect()
    }
    pub fn get_links_to(&self, to_node: &str) -> Vec<&NodeLink> {
        self.links
            .iter()
            .filter(|link| link.to_node() == to_node)
            .collect()
    }
    pub fn get_links_to_pin(&self, to_node: &str, to_pin: &str) -> Vec<&NodeLink> {
        self.links
            .iter()
            .filter(|link| link.to_node() == to_node && link.to_pin() == to_pin)
            .collect()
    }
    pub fn get_links_count(&self) -> usize {
        self.links.len()
    }
    pub fn resolve_links(&mut self, registry: &LogicNodeRegistry) {
        let links = &self.links;
        let nodes = &mut self.nodes;
        links.iter().for_each(|l| {
            let from = nodes.iter().position(|n| n.name() == l.from_node());
            let to = nodes.iter().position(|n| n.name() == l.to_node());
            if let (Some(from), Some(to)) = (from, to) {
                let (from_node, to_node) = if from < to {
                    let (start, end) = nodes.split_at_mut(to);
                    (start[from].node(), end[0].node_mut())
                } else {
                    let (start, end) = nodes.split_at_mut(from);
                    (end[0].node(), start[to].node_mut())
                };
                l.resolve(registry, from_node, to_node);
            }
        });
    }
    pub fn nodes(&self) -> &Vec<Box<dyn NodeTrait>> {
        &self.nodes
    }
    pub fn nodes_mut(&mut self) -> &mut Vec<Box<dyn NodeTrait>> {
        &mut self.nodes
    }
    pub fn links(&self) -> &Vec<NodeLink> {
        &self.links
    }
    pub fn find_node_index(&self, name: &str) -> Option<usize> {
        let uid = generate_uid_from_string(name);
        self.nodes.iter().position(|n| n.id() == uid)
    }
}
