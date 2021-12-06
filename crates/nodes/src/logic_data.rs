use sabi_serialize::{Deserialize, Serialize, SerializeFile};

use crate::{LogicExecution, NodeExecutionType, NodeState, NodeTree, PinId};

#[derive(Default, PartialEq, Eq, Hash, Clone)]
struct LinkInfo {
    node: usize,
    pin: PinId,
}
#[derive(Default, Clone)]
struct PinInfo {
    id: PinId,
    links: Vec<LinkInfo>,
}
#[derive(Default, Clone)]
struct NodeInfo {
    inputs: Vec<PinInfo>,
    outputs: Vec<PinInfo>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(crate = "sabi_serialize")]
pub struct LogicData {
    #[serde(flatten)]
    tree: NodeTree,
    #[serde(skip)]
    active_nodes: Vec<LinkInfo>,
    #[serde(skip)]
    nodes_info: Vec<NodeInfo>,
    #[serde(skip)]
    execution_state: Vec<NodeState>,
}

impl SerializeFile for LogicData {
    fn extension() -> &'static str {
        "logic"
    }
}

impl From<NodeTree> for LogicData {
    fn from(tree: NodeTree) -> Self {
        Self {
            tree,
            active_nodes: Vec::new(),
            nodes_info: Vec::new(),
            execution_state: Vec::new(),
        }
    }
}

impl LogicData {
    pub fn tree(&self) -> &NodeTree {
        &self.tree
    }
    pub fn init(&mut self) {
        let nodes = self.tree.nodes();
        nodes.iter().enumerate().for_each(|(node_index, n)| {
            if !n.node().has_input::<LogicExecution>() && n.node().has_output::<LogicExecution>() {
                self.active_nodes.push(LinkInfo {
                    node: node_index,
                    pin: PinId::invalid(),
                });
            }
            let mut node_info = NodeInfo::default();
            //Get for each pin of each input links info of linked node and its pin
            n.node().inputs().iter().for_each(|(id, _)| {
                let mut pin_info = PinInfo {
                    id: id.clone(),
                    ..Default::default()
                };
                let links = self.tree.get_links_to_pin(n.name(), id.name());
                links.iter().for_each(|l| {
                    if let Some(from_node_index) = self.tree.find_node_index(l.from_node()) {
                        if let Some((from_pin_id, _)) = nodes[from_node_index]
                            .node()
                            .outputs()
                            .iter()
                            .find(|(id, _)| id.name() == l.from_pin())
                        {
                            let link_info = LinkInfo {
                                node: from_node_index,
                                pin: from_pin_id.clone(),
                            };
                            pin_info.links.push(link_info);
                        } else {
                            eprintln!(
                                "Unable to find output pin {} of node {}",
                                l.from_pin(),
                                nodes[from_node_index].name()
                            );
                        }
                    }
                });
                node_info.inputs.push(pin_info);
            });
            //Get for each pin of each output links info of linked node and its pin
            n.node().outputs().iter().for_each(|(id, _)| {
                let mut pin_info = PinInfo {
                    id: id.clone(),
                    ..Default::default()
                };
                let links = self.tree.get_links_from_pin(n.name(), id.name());
                links.iter().for_each(|l| {
                    if let Some(to_node_index) = self.tree.find_node_index(l.to_node()) {
                        if let Some((to_pin_id, _)) = nodes[to_node_index]
                            .node()
                            .inputs()
                            .iter()
                            .find(|(id, _)| id.name() == l.to_pin())
                        {
                            let link_info = LinkInfo {
                                node: to_node_index,
                                pin: to_pin_id.clone(),
                            };
                            pin_info.links.push(link_info);
                        } else {
                            eprintln!(
                                "Unable to find input pin {} of node {}",
                                l.to_pin(),
                                nodes[to_node_index].name()
                            );
                        }
                    }
                });
                node_info.outputs.push(pin_info);
            });

            self.nodes_info.push(node_info);
        });
        self.execution_state.resize(nodes.len(), NodeState::Active);
    }

    pub fn execute(&mut self) {
        self.execute_active_nodes(self.active_nodes.clone());
    }

    fn execute_active_nodes(&mut self, mut nodes_to_execute: Vec<LinkInfo>) {
        if nodes_to_execute.is_empty() {
            return;
        }
        let mut new_nodes = Vec::new();
        nodes_to_execute.iter().for_each(|l| {
            let mut nodes = Self::execute_node(
                &mut self.tree,
                l,
                &self.nodes_info,
                &mut self.execution_state,
            );
            new_nodes.append(&mut nodes);
        });
        nodes_to_execute.retain(|link_info| {
            let node_state = &self.execution_state[link_info.node];
            match node_state {
                NodeState::Active => true,
                NodeState::Running(_) => {
                    if !self.active_nodes.contains(link_info) {
                        self.active_nodes.push(link_info.clone());
                    }
                    false
                }
                NodeState::Executed(_) => false,
            }
        });
        new_nodes.iter().for_each(|l| {
            if self.execution_state[l.node] == NodeState::Active && !nodes_to_execute.contains(l) {
                nodes_to_execute.push(l.clone());
            }
        });
        self.active_nodes.retain(|l| {
            self.tree.nodes()[l.node].execytion_type() != NodeExecutionType::OneShot
                || self.tree.nodes()[l.node].execytion_type() == NodeExecutionType::Continuous
        });
        self.execute_active_nodes(nodes_to_execute);
    }

    fn execute_node(
        tree: &mut NodeTree,
        link_info: &LinkInfo,
        nodes_info: &[NodeInfo],
        execution_state: &mut [NodeState],
    ) -> Vec<LinkInfo> {
        let mut new_nodes_to_execute = Vec::new();

        let info = &nodes_info[link_info.node];
        info.inputs.iter().for_each(|pin_info| {
            let node = tree.nodes_mut()[link_info.node].node();
            if node.is_input::<LogicExecution>(&pin_info.id) {
                return;
            }
            pin_info.links.iter().for_each(|l| {
                if execution_state[l.node] == NodeState::Active {
                    let mut nodes = Self::execute_node(tree, l, nodes_info, execution_state);
                    new_nodes_to_execute.append(&mut nodes);
                }

                let nodes = tree.nodes_mut();
                let (from_node, to_node) = if l.node < link_info.node {
                    let (start, end) = nodes.split_at_mut(link_info.node);
                    (start[l.node].node(), end[0].node_mut())
                } else {
                    let (start, end) = nodes.split_at_mut(l.node);
                    (end[0].node(), start[link_info.node].node_mut())
                };
                if let Some(input) = to_node.inputs_mut().get_mut(&pin_info.id) {
                    input.copy_from(from_node, &l.pin);
                }
            });
        });
        let node = &mut tree.nodes_mut()[link_info.node];
        execution_state[link_info.node] = node.execute(&link_info.pin);

        match &execution_state[link_info.node] {
            NodeState::Executed(output_pins) | NodeState::Running(output_pins) => {
                if let Some(pins) = output_pins {
                    pins.iter().for_each(|pin_id| {
                        info.outputs.iter().for_each(|o| {
                            if pin_id == &o.id {
                                o.links.iter().for_each(|link_info| {
                                    new_nodes_to_execute.push(link_info.clone());
                                });
                            }
                        });
                    });
                }
            }
            _ => {}
        }

        new_nodes_to_execute
    }
}
