use inox_graph::{Dependency, Graph, GraphError, PassNode};

fn node(name: &str, enabled: bool, reads: &[u32], writes: &[u32]) -> PassNode<u32> {
    PassNode::new(name, enabled, reads.to_vec(), writes.to_vec())
}

fn dependency(predecessor: &str, successor: &str, resource: u32) -> Dependency<u32> {
    Dependency::new(predecessor, successor, resource)
}

#[test]
fn stable_order_uses_declaration_order_for_ready_nodes() {
    let graph = Graph::new(
        vec![
            node("join", true, &[1, 2], &[3]),
            node("first", true, &[], &[1]),
            node("second", true, &[], &[2]),
        ],
        vec![
            dependency("first", "join", 1),
            dependency("second", "join", 2),
        ],
    );

    assert_eq!(
        graph.topological_order().unwrap(),
        ["first", "second", "join"]
    );
}

#[test]
fn duplicate_node_names_are_rejected() {
    let graph = Graph::new(
        vec![node("same", true, &[], &[]), node("same", false, &[], &[])],
        Vec::new(),
    );

    assert!(matches!(
        graph.validate(),
        Err(GraphError::DuplicateNodeName { name }) if name == "same"
    ));
}

#[test]
fn unknown_endpoints_are_rejected() {
    let predecessor_unknown = Graph::new(
        vec![node("consumer", true, &[1], &[])],
        vec![dependency("producer", "consumer", 1)],
    );
    assert!(matches!(
        predecessor_unknown.validate(),
        Err(GraphError::UnknownPredecessor { name }) if name == "producer"
    ));

    let successor_unknown = Graph::new(
        vec![node("producer", true, &[], &[1])],
        vec![dependency("producer", "consumer", 1)],
    );
    assert!(matches!(
        successor_unknown.validate(),
        Err(GraphError::UnknownSuccessor { name }) if name == "consumer"
    ));
}

#[test]
fn cycles_are_rejected_with_deterministic_members() {
    let graph = Graph::new(
        vec![
            node("a", true, &[2], &[1]),
            node("b", true, &[1], &[2]),
            node("independent", true, &[], &[]),
        ],
        vec![dependency("a", "b", 1), dependency("b", "a", 2)],
    );

    assert!(matches!(
        graph.validate(),
        Err(GraphError::Cycle { nodes }) if nodes == ["a", "b"]
    ));
}

#[test]
fn cycle_diagnostics_exclude_nodes_downstream_of_cycle() {
    let graph = Graph::new(
        vec![
            node("a", true, &[2], &[1]),
            node("b", true, &[1], &[2, 3]),
            node("downstream", true, &[3], &[]),
        ],
        vec![
            dependency("a", "b", 1),
            dependency("b", "a", 2),
            dependency("b", "downstream", 3),
        ],
    );

    assert!(matches!(
        graph.validate(),
        Err(GraphError::Cycle { nodes }) if nodes == ["a", "b"]
    ));
}

#[test]
fn duplicate_dependencies_are_rejected() {
    let graph = Graph::new(
        vec![
            node("producer", true, &[], &[1]),
            node("consumer", true, &[1], &[]),
        ],
        vec![
            dependency("producer", "consumer", 1),
            dependency("producer", "consumer", 1),
        ],
    );

    assert!(matches!(
        graph.validate(),
        Err(GraphError::DuplicateDependency { predecessor, successor })
            if predecessor == "producer" && successor == "consumer"
    ));
}

#[test]
fn disabled_nodes_are_retained_but_not_scheduled() {
    let graph = Graph::new(
        vec![
            node("disabled", false, &[], &[1]),
            node("active", true, &[], &[]),
        ],
        Vec::new(),
    );

    let validation = graph.validate().unwrap();
    assert_eq!(validation.order, ["active"]);
    assert_eq!(graph.nodes()[0].name, "disabled");
}

#[test]
fn resource_handoff_requires_producer_and_consumer_metadata() {
    let not_produced = Graph::new(
        vec![
            node("producer", true, &[], &[]),
            node("consumer", true, &[1], &[]),
        ],
        vec![dependency("producer", "consumer", 1)],
    );
    assert!(matches!(
        not_produced.validate(),
        Err(GraphError::ResourceNotProduced { .. })
    ));

    let not_consumed = Graph::new(
        vec![
            node("producer", true, &[], &[1]),
            node("consumer", true, &[], &[]),
        ],
        vec![dependency("producer", "consumer", 1)],
    );
    assert!(matches!(
        not_consumed.validate(),
        Err(GraphError::ResourceNotConsumed { .. })
    ));
}

#[test]
fn disabled_dependency_to_enabled_consumer_is_rejected() {
    let graph = Graph::new(
        vec![
            node("disabled", false, &[], &[1]),
            node("active", true, &[1], &[]),
        ],
        vec![dependency("disabled", "active", 1)],
    );

    assert!(matches!(
        graph.validate(),
        Err(GraphError::DisabledPredecessor { predecessor, successor })
            if predecessor == "disabled" && successor == "active"
    ));
}
