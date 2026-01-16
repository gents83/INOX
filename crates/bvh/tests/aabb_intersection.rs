use inox_math::{Vector3, VecBase};

fn select(f: f32, t: f32, cond: bool) -> f32 {
    if cond { t } else { f }
}

fn intersect_aabb(origin: Vector3, direction: Vector3, max_distance: f32, aabb_min: Vector3, aabb_max: Vector3) -> f32 {
    let inverse_dir = Vector3::new(1.0 / direction.x, 1.0 / direction.y, 1.0 / direction.z);

    // using VecBase::mul for element-wise multiplication
    let t1 = (aabb_min - origin).mul(inverse_dir);
    let t2 = (aabb_max - origin).mul(inverse_dir);

    // VecBase also has min/max
    let t_min = t1.min(t2);
    let t_max = t1.max(t2);

    let t_near = t_min.x.max(t_min.y).max(t_min.z);
    let t_far = t_max.x.min(t_max.y).min(t_max.z);

    let is_hit = t_near <= t_far && t_far >= 0.0 && t_near < max_distance;

    select(max_distance, t_near, is_hit)
}

#[test]
fn test_aabb_intersection_basic() {
    let origin = Vector3::new(0.0, 0.0, -10.0);
    let direction = Vector3::new(0.0, 0.0, 1.0);
    let max_distance = 100.0;
    let aabb_min = Vector3::new(-1.0, -1.0, 5.0);
    let aabb_max = Vector3::new(1.0, 1.0, 7.0);

    let t = intersect_aabb(origin, direction, max_distance, aabb_min, aabb_max);

    assert!((t - 15.0).abs() < 0.001, "Expected intersection at 15.0, got {}", t);
}

#[test]
fn test_aabb_intersection_miss() {
    let origin = Vector3::new(0.0, 0.0, -10.0);
    let direction = Vector3::new(0.0, 0.0, 1.0);
    let max_distance = 100.0;
    let aabb_min = Vector3::new(10.0, 10.0, 5.0);
    let aabb_max = Vector3::new(12.0, 12.0, 7.0);

    let t = intersect_aabb(origin, direction, max_distance, aabb_min, aabb_max);

    assert_eq!(t, max_distance, "Expected miss (max_distance), got {}", t);
}

#[test]
fn test_aabb_intersection_inside() {
    let origin = Vector3::new(0.0, 0.0, 6.0);
    let direction = Vector3::new(0.0, 0.0, 1.0);
    let max_distance = 100.0;
    let aabb_min = Vector3::new(-1.0, -1.0, 5.0);
    let aabb_max = Vector3::new(1.0, 1.0, 7.0);

    let t = intersect_aabb(origin, direction, max_distance, aabb_min, aabb_max);

    assert!((t - -1.0).abs() < 0.001, "Expected intersection at -1.0 (inside), got {}", t);
}
