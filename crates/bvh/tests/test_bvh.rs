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
