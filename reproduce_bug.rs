
use std::ops::{Add, Sub, Mul, Div};

#[derive(Clone, Copy, Debug)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 { x, y, z }
    }
    fn abs(self) -> Self {
        Vec3::new(self.x.abs(), self.y.abs(), self.z.abs())
    }
}

impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl Mul<Vec3> for Vec3 {
    type Output = Vec3;
    fn mul(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x * other.x, self.y * other.y, self.z * other.z)
    }
}

impl Div<Vec3> for f32 {
    type Output = Vec3;
    fn div(self, other: Vec3) -> Vec3 {
        Vec3::new(self / other.x, self / other.y, self / other.z)
    }
}

// Emulate WGSL select
fn select(f: f32, t: f32, cond: bool) -> f32 {
    if cond { t } else { f }
}

// The suspicious implementation
fn intersect_aabb_suspicious(origin: Vec3, direction: Vec3, max_distance: f32, aabb_min: Vec3, aabb_max: Vec3) -> f32 {
    let size = aabb_max - aabb_min;
    let inverse_dir = 1.0 / direction;
    let n = origin * inverse_dir;
    let k = inverse_dir.abs() * size;
    let t_min = Vec3::new(-n.x - k.x, -n.y - k.y, -n.z - k.z);
    let t_max = Vec3::new(-n.x + k.x, -n.y + k.y, -n.z + k.z);

    let t_near = t_min.x.max(t_min.y).max(t_min.z);
    let t_far = t_max.x.min(t_max.y).min(t_max.z);

    select(t_far, t_near, t_near < max_distance && t_far > 0.0)
}

// The correct implementation
fn intersect_aabb_correct(origin: Vec3, direction: Vec3, max_distance: f32, aabb_min: Vec3, aabb_max: Vec3) -> f32 {
    let inverse_dir = 1.0 / direction;
    let t1 = (aabb_min - origin) * inverse_dir;
    let t2 = (aabb_max - origin) * inverse_dir;

    let tmin = Vec3::new(t1.x.min(t2.x), t1.y.min(t2.y), t1.z.min(t2.z));
    let tmax = Vec3::new(t1.x.max(t2.x), t1.y.max(t2.y), t1.z.max(t2.z));

    let t_near = tmin.x.max(tmin.y).max(tmin.z);
    let t_far = tmax.x.min(tmax.y).min(tmax.z);

    if t_near > t_far || t_far < 0.0 {
        return max_distance;
    }

    // Logic to match the return style
    // If t_near > 0, we hit from outside. Return t_near.
    // If t_near <= 0 and t_far > 0, we are inside. Return t_near (or 0).
    // The previous function returned t_near if valid.

    // If t_near is > max_distance, we missed (too far).
    if t_near > max_distance {
        return max_distance;
    }

    return t_near;
}

fn main() {
    let origin = Vec3::new(0.0, 0.0, -10.0);
    let direction = Vec3::new(0.0, 0.0, 1.0);
    let max_distance = 100.0;
    let aabb_min = Vec3::new(-1.0, -1.0, 5.0);
    let aabb_max = Vec3::new(1.0, 1.0, 7.0);

    let t_suspicious = intersect_aabb_suspicious(origin, direction, max_distance, aabb_min, aabb_max);
    let t_correct = intersect_aabb_correct(origin, direction, max_distance, aabb_min, aabb_max);

    println!("Origin: {:?}, Dir: {:?}", origin, direction);
    println!("AABB: {:?} to {:?}", aabb_min, aabb_max);
    println!("Suspicious t: {}", t_suspicious);
    println!("Correct t: {}", t_correct);

    if (t_correct - 15.0).abs() < 0.1 {
        println!("Correct version expects intersection at 15.0 (5 - (-10))");
    } else {
        println!("Correct version calc seems wrong? t={}", t_correct);
    }

    // Another test case: AABB off-center
    let aabb_min_offset = Vec3::new(10.0, 10.0, 5.0);
    let aabb_max_offset = Vec3::new(12.0, 12.0, 7.0);
    let t_suspicious_2 = intersect_aabb_suspicious(origin, direction, max_distance, aabb_min_offset, aabb_max_offset);
    let t_correct_2 = intersect_aabb_correct(origin, direction, max_distance, aabb_min_offset, aabb_max_offset);

    println!("\nTest 2 (Offset AABB):");
    println!("AABB: {:?} to {:?}", aabb_min_offset, aabb_max_offset);
    println!("Suspicious t: {}", t_suspicious_2);
    println!("Correct t: {}", t_correct_2);

    if t_correct_2 >= max_distance {
        println!("Correct version says MISS (correct)");
    }
}
