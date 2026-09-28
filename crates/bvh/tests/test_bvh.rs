use inox_bvh::{
    create_linearized_bvh, try_create_linearized_bvh, validate_linearized_bvh, BVHTree,
    LinearizedBVHError, AABB,
};
use inox_math::Vector3;

#[test]
fn builds_root_and_leaf_nodes_for_two_bounds() {
    let first = AABB::create(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0), 0);
    let second = AABB::create(Vector3::new(2.0, 2.0, 2.0), Vector3::new(3.0, 3.0, 3.0), 1);
    let tree = BVHTree::new(&[first, second]);

    assert_eq!(tree.nodes().len(), 3);
    assert_eq!(tree.nodes()[0].parent(), -1);
    assert_eq!(tree.nodes()[0].left(), 1);
    assert_eq!(tree.nodes()[0].right(), 2);
    assert_eq!(tree.nodes()[1].aabb_index(), 0);
    assert_eq!(tree.nodes()[2].aabb_index(), 1);
    assert!(tree.nodes()[1].is_leaf());
    assert!(tree.nodes()[2].is_leaf());
}

#[test]
fn computes_bounds_and_surface_area() {
    let bounds = AABB::compute_aabb(&[
        AABB::create(Vector3::new(-1.0, 0.0, 2.0), Vector3::new(1.0, 2.0, 3.0), 0),
        AABB::create(Vector3::new(0.0, -2.0, 1.0), Vector3::new(4.0, 1.0, 5.0), 1),
    ]);

    assert_eq!(bounds.min(), Vector3::new(-1.0, -2.0, 1.0));
    assert_eq!(bounds.max(), Vector3::new(4.0, 2.0, 5.0));
    assert_eq!(bounds.center(), Vector3::new(1.5, 0.0, 3.0));
    assert_eq!(
        bounds.surface_area(),
        2.0 * (5.0 * 4.0 + 4.0 * 4.0 + 4.0 * 5.0)
    );
}

#[test]
fn empty_and_one_node_bvhs_have_checked_linearization() {
    let empty = BVHTree::try_new(&[]).unwrap();
    assert!(empty.nodes().is_empty());
    assert!(try_create_linearized_bvh(&empty).unwrap().is_empty());

    let one = AABB::create(Vector3::new(1.0, 2.0, 3.0), Vector3::new(1.0, 2.0, 3.0), 7);
    let tree = BVHTree::try_new(&[one]).unwrap();
    assert_eq!(tree.nodes().len(), 1);
    assert!(tree.validate().is_ok());
    let linearized = create_linearized_bvh(&tree);
    assert_eq!(linearized.len(), 1);
    assert_eq!(linearized[0].miss, -1);
    assert_eq!(linearized[0].reference, 7);
}

#[test]
fn checked_builder_rejects_non_finite_and_degenerate_ordering_errors() {
    assert!(BVHTree::try_new(&[AABB::create(
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 1.0),
        0,
    )])
    .is_err());
    assert!(BVHTree::try_new(&[AABB::create(
        Vector3::new(f32::NAN, 0.0, 0.0),
        Vector3::new(1.0, 1.0, 1.0),
        0,
    )])
    .is_err());
    let degenerate = BVHTree::try_new(&[AABB::create(
        Vector3::new(2.0, 2.0, 2.0),
        Vector3::new(2.0, 2.0, 2.0),
        0,
    )])
    .unwrap();
    assert!(degenerate.validate().is_ok());
}

#[test]
fn invalid_escape_links_are_rejected() {
    let node = inox_bvh::GPUBVHNode {
        min: [0.0; 3],
        max: [1.0; 3],
        miss: 3,
        reference: 0,
    };
    assert_eq!(
        validate_linearized_bvh(&[node]),
        Err(LinearizedBVHError::InvalidEscape {
            index: 0,
            escape: 3
        })
    );
}

#[test]
fn sah_partition_is_not_forced_to_a_midpoint() {
    let bounds = [
        AABB::create(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0), 0),
        AABB::create(
            Vector3::new(100.0, 0.0, 0.0),
            Vector3::new(101.0, 1.0, 1.0),
            1,
        ),
        AABB::create(
            Vector3::new(101.0, 0.0, 0.0),
            Vector3::new(102.0, 1.0, 1.0),
            2,
        ),
        AABB::create(
            Vector3::new(102.0, 0.0, 0.0),
            Vector3::new(103.0, 1.0, 1.0),
            3,
        ),
    ];
    let tree = BVHTree::new(&bounds);
    let root = &tree.nodes()[0];

    // Isolating the distant first leaf has lower SAH cost than a midpoint
    // split, so the root's left child must be a leaf and its right child an
    // internal node containing the remaining three leaves.
    assert!(tree.nodes()[root.left() as usize].is_leaf());
    assert!(!tree.nodes()[root.right() as usize].is_leaf());
    assert!(tree.validate().is_ok());
}

#[test]
fn traversal_links_form_a_single_rooted_tree_with_one_leaf_per_input() {
    let bounds = [
        AABB::create(
            Vector3::new(-5.0, 0.0, 0.0),
            Vector3::new(-4.0, 1.0, 1.0),
            0,
        ),
        AABB::create(
            Vector3::new(-2.0, 1.0, 0.0),
            Vector3::new(-1.0, 2.0, 1.0),
            1,
        ),
        AABB::create(Vector3::new(0.0, 2.0, 0.0), Vector3::new(1.0, 3.0, 1.0), 2),
        AABB::create(Vector3::new(2.0, 3.0, 0.0), Vector3::new(3.0, 4.0, 1.0), 3),
        AABB::create(Vector3::new(4.0, 4.0, 0.0), Vector3::new(5.0, 5.0, 1.0), 4),
    ];
    let tree = BVHTree::new(&bounds);
    let nodes = tree.nodes();
    let mut visited = vec![false; nodes.len()];
    let mut leaves = vec![false; bounds.len()];
    let mut stack = vec![0usize];

    assert!(!nodes.is_empty());
    assert_eq!(nodes[0].parent(), -1);
    while let Some(index) = stack.pop() {
        assert!(index < nodes.len());
        assert!(!visited[index], "BVH traversal visited a node twice");
        visited[index] = true;
        let node = &nodes[index];
        if node.is_leaf() {
            let aabb_index = usize::try_from(node.aabb_index()).unwrap();
            assert!(aabb_index < bounds.len());
            assert!(!leaves[aabb_index], "BVH duplicated an input leaf");
            leaves[aabb_index] = true;
        } else {
            let left = usize::try_from(node.left()).unwrap();
            let right = usize::try_from(node.right()).unwrap();
            assert!(left < nodes.len());
            assert!(right < nodes.len());
            assert_ne!(left, right);
            assert_eq!(nodes[left].parent(), index as i32);
            assert_eq!(nodes[right].parent(), index as i32);
            stack.push(left);
            stack.push(right);
        }
    }

    assert!(visited.into_iter().all(|was_visited| was_visited));
    assert!(leaves.into_iter().all(|was_visited| was_visited));
}
