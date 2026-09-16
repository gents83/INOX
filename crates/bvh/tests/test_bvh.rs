use inox_bvh::{BVHTree, AABB};
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
