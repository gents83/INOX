//! A portable dependency graph for pass scheduling.
//!
//! The graph deliberately stores names and resource keys only.  It has no
//! renderer, operating-system, or GPU dependencies, so it can be shared by
//! native and WebAssembly graph builders.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::Hash;

/// A pass declaration and the resources it reads and writes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PassNode<R> {
    pub name: String,
    pub enabled: bool,
    pub reads: Vec<R>,
    pub writes: Vec<R>,
}

impl<R> PassNode<R> {
    pub fn new(name: impl Into<String>, enabled: bool, reads: Vec<R>, writes: Vec<R>) -> Self {
        Self {
            name: name.into(),
            enabled,
            reads,
            writes,
        }
    }
}

/// A resource handoff from one pass to another.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dependency<R> {
    pub predecessor: String,
    pub successor: String,
    pub resource: R,
}

impl<R> Dependency<R> {
    pub fn new(predecessor: impl Into<String>, successor: impl Into<String>, resource: R) -> Self {
        Self {
            predecessor: predecessor.into(),
            successor: successor.into(),
            resource,
        }
    }
}

/// Errors found while validating a graph declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GraphError {
    DuplicateNodeName {
        name: String,
    },
    UnknownPredecessor {
        name: String,
    },
    UnknownSuccessor {
        name: String,
    },
    DuplicateDependency {
        predecessor: String,
        successor: String,
    },
    ResourceNotProduced {
        predecessor: String,
        successor: String,
    },
    ResourceNotConsumed {
        predecessor: String,
        successor: String,
    },
    DisabledPredecessor {
        predecessor: String,
        successor: String,
    },
    Cycle {
        nodes: Vec<String>,
    },
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateNodeName { name } => write!(f, "duplicate graph node name: {name}"),
            Self::UnknownPredecessor { name } => {
                write!(f, "unknown dependency predecessor: {name}")
            }
            Self::UnknownSuccessor { name } => write!(f, "unknown dependency successor: {name}"),
            Self::DuplicateDependency {
                predecessor,
                successor,
            } => {
                write!(f, "duplicate dependency: {predecessor} -> {successor}")
            }
            Self::ResourceNotProduced {
                predecessor,
                successor,
            } => write!(
                f,
                "dependency resource is not produced by predecessor {predecessor} for {successor}"
            ),
            Self::ResourceNotConsumed {
                predecessor,
                successor,
            } => write!(
                f,
                "dependency resource is not consumed by successor {successor} from {predecessor}"
            ),
            Self::DisabledPredecessor {
                predecessor,
                successor,
            } => write!(
                f,
                "enabled successor {successor} depends on disabled predecessor {predecessor}"
            ),
            Self::Cycle { nodes } => {
                write!(f, "graph contains a cycle involving: {}", nodes.join(", "))
            }
        }
    }
}

impl std::error::Error for GraphError {}

/// The validated active order of a graph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphValidation {
    /// Enabled node names in deterministic topological order.
    pub order: Vec<String>,
}

impl GraphValidation {
    pub fn topological_order(&self) -> &[String] {
        &self.order
    }
}

/// A declaration-order-stable pass/resource dependency graph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Graph<R> {
    pub nodes: Vec<PassNode<R>>,
    pub dependencies: Vec<Dependency<R>>,
}

impl<R> Graph<R> {
    pub fn new(nodes: Vec<PassNode<R>>, dependencies: Vec<Dependency<R>>) -> Self {
        Self {
            nodes,
            dependencies,
        }
    }

    pub fn from_parts(
        nodes: impl IntoIterator<Item = PassNode<R>>,
        dependencies: impl IntoIterator<Item = Dependency<R>>,
    ) -> Self {
        Self::new(
            nodes.into_iter().collect(),
            dependencies.into_iter().collect(),
        )
    }

    pub fn nodes(&self) -> &[PassNode<R>] {
        &self.nodes
    }

    pub fn dependencies(&self) -> &[Dependency<R>] {
        &self.dependencies
    }
}

impl<R: Clone + Eq + Hash> Graph<R> {
    /// Validates declarations and returns the active, stable topological order.
    ///
    /// Disabled nodes remain in the declaration for diagnostics and future
    /// configuration, but are not scheduled. Dependencies between disabled
    /// nodes are not scheduling edges; an enabled successor may not depend on
    /// a disabled predecessor because that resource cannot be produced.
    pub fn validate(&self) -> Result<GraphValidation, GraphError> {
        let mut indices = HashMap::with_capacity(self.nodes.len());
        for (index, node) in self.nodes.iter().enumerate() {
            if indices.insert(node.name.as_str(), index).is_some() {
                return Err(GraphError::DuplicateNodeName {
                    name: node.name.clone(),
                });
            }
        }

        let mut seen_dependencies = HashSet::with_capacity(self.dependencies.len());
        for dependency in &self.dependencies {
            let predecessor_index = indices.get(dependency.predecessor.as_str()).copied();
            if predecessor_index.is_none() {
                return Err(GraphError::UnknownPredecessor {
                    name: dependency.predecessor.clone(),
                });
            }
            let successor_index = indices.get(dependency.successor.as_str()).copied();
            if successor_index.is_none() {
                return Err(GraphError::UnknownSuccessor {
                    name: dependency.successor.clone(),
                });
            }

            let predecessor_index = predecessor_index.expect("checked above");
            let successor_index = successor_index.expect("checked above");
            if !self.nodes[predecessor_index].enabled && self.nodes[successor_index].enabled {
                return Err(GraphError::DisabledPredecessor {
                    predecessor: dependency.predecessor.clone(),
                    successor: dependency.successor.clone(),
                });
            }

            let key = (
                dependency.predecessor.as_str(),
                dependency.successor.as_str(),
                &dependency.resource,
            );
            if !seen_dependencies.insert(key) {
                return Err(GraphError::DuplicateDependency {
                    predecessor: dependency.predecessor.clone(),
                    successor: dependency.successor.clone(),
                });
            }

            let predecessor = &self.nodes[predecessor_index];
            if !predecessor
                .writes
                .iter()
                .any(|resource| resource == &dependency.resource)
            {
                return Err(GraphError::ResourceNotProduced {
                    predecessor: dependency.predecessor.clone(),
                    successor: dependency.successor.clone(),
                });
            }
            let successor = &self.nodes[successor_index];
            if !successor
                .reads
                .iter()
                .any(|resource| resource == &dependency.resource)
            {
                return Err(GraphError::ResourceNotConsumed {
                    predecessor: dependency.predecessor.clone(),
                    successor: dependency.successor.clone(),
                });
            }
        }

        let active_indices: Vec<usize> = self
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(index, node)| node.enabled.then_some(index))
            .collect();
        let mut indegree = vec![0usize; self.nodes.len()];
        let mut successors = vec![Vec::<usize>::new(); self.nodes.len()];
        for dependency in &self.dependencies {
            let predecessor = indices[dependency.predecessor.as_str()];
            let successor = indices[dependency.successor.as_str()];
            if self.nodes[predecessor].enabled && self.nodes[successor].enabled {
                indegree[successor] += 1;
                successors[predecessor].push(successor);
            }
        }

        let mut ready: Vec<usize> = active_indices
            .iter()
            .copied()
            .filter(|&index| indegree[index] == 0)
            .collect();
        let mut order = Vec::with_capacity(active_indices.len());
        while let Some(index) = ready.first().copied() {
            ready.remove(0);
            order.push(self.nodes[index].name.clone());
            for successor in successors[index].iter().copied() {
                indegree[successor] -= 1;
                if indegree[successor] == 0 {
                    // Declaration order is the tie-breaker, rather than edge
                    // insertion order or a hash-map iteration order.
                    let position = ready
                        .iter()
                        .position(|&candidate| candidate > successor)
                        .unwrap_or(ready.len());
                    ready.insert(position, successor);
                }
            }
        }

        if order.len() != active_indices.len() {
            let scheduled: HashSet<&str> = order.iter().map(String::as_str).collect();
            let mut candidates = vec![false; self.nodes.len()];
            for &index in &active_indices {
                candidates[index] = !scheduled.contains(self.nodes[index].name.as_str());
            }
            let cycle_indices = cycle_members(&successors, &candidates);
            let nodes = active_indices
                .iter()
                .filter_map(|&index| cycle_indices[index].then_some(self.nodes[index].name.clone()))
                .collect();
            return Err(GraphError::Cycle { nodes });
        }

        Ok(GraphValidation { order })
    }

    pub fn topological_order(&self) -> Result<Vec<String>, GraphError> {
        Ok(self.validate()?.order)
    }
}

/// Finds the nodes belonging to cyclic strongly connected components.
///
/// Kahn's algorithm leaves both cycle members and nodes downstream of a cycle
/// unscheduled. SCC classification keeps those downstream nodes out of the
/// diagnostic while retaining declaration order for the reported members.
fn cycle_members(successors: &[Vec<usize>], candidates: &[bool]) -> Vec<bool> {
    struct Tarjan<'a> {
        successors: &'a [Vec<usize>],
        candidates: &'a [bool],
        next_index: usize,
        indices: Vec<Option<usize>>,
        lowlinks: Vec<usize>,
        stack: Vec<usize>,
        on_stack: Vec<bool>,
        cyclic: Vec<bool>,
    }

    impl Tarjan<'_> {
        fn visit(&mut self, vertex: usize) {
            self.indices[vertex] = Some(self.next_index);
            self.lowlinks[vertex] = self.next_index;
            self.next_index += 1;
            self.stack.push(vertex);
            self.on_stack[vertex] = true;

            let outgoing = self.successors[vertex].clone();
            for successor in outgoing {
                if !self.candidates[successor] {
                    continue;
                }
                match self.indices[successor] {
                    None => {
                        self.visit(successor);
                        self.lowlinks[vertex] = self.lowlinks[vertex].min(self.lowlinks[successor]);
                    }
                    Some(successor_index) if self.on_stack[successor] => {
                        self.lowlinks[vertex] = self.lowlinks[vertex].min(successor_index);
                    }
                    Some(_) => {}
                }
            }

            if self.lowlinks[vertex] != self.indices[vertex].expect("vertex was indexed on entry") {
                return;
            }

            let mut component = Vec::new();
            while let Some(member) = self.stack.pop() {
                self.on_stack[member] = false;
                component.push(member);
                if member == vertex {
                    break;
                }
            }
            if component.len() > 1 || self.successors[vertex].contains(&vertex) {
                for member in component {
                    self.cyclic[member] = true;
                }
            }
        }
    }

    let mut tarjan = Tarjan {
        successors,
        candidates,
        next_index: 0,
        indices: vec![None; successors.len()],
        lowlinks: vec![0; successors.len()],
        stack: Vec::new(),
        on_stack: vec![false; successors.len()],
        cyclic: vec![false; successors.len()],
    };
    for (vertex, &candidate) in candidates.iter().enumerate() {
        if candidate && tarjan.indices[vertex].is_none() {
            tarjan.visit(vertex);
        }
    }
    tarjan.cyclic
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph_validates() {
        assert_eq!(
            Graph::<u32>::new(Vec::new(), Vec::new())
                .topological_order()
                .unwrap(),
            Vec::<String>::new()
        );
    }
}
