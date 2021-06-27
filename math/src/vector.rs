use std::f32::consts::PI;

use cgmath::*;

pub type Vector2 = cgmath::Vector2<f32>;
pub type Vector3 = cgmath::Vector3<f32>;
pub type Vector4 = cgmath::Vector4<f32>;

pub trait VecBase {
    fn default_zero() -> Self;
    fn squared_distance(self, other: Self) -> f32;
    fn length(self) -> f32;
    fn add(self, rhs: Self) -> Self;
    fn sub(self, rhs: Self) -> Self;
    fn mul(self, rhs: Self) -> Self;
    fn div(self, rhs: Self) -> Self;
}

macro_rules! implement_vector_base {
    ($VectorN:ident) => {
        impl VecBase for $VectorN {
            fn default_zero() -> Self {
                Self::zero()
            }
            fn length(self) -> f32 {
                self.magnitude()
            }
            fn squared_distance(self, other: Self) -> f32 {
                self.distance2(other)
            }
            fn add(self, rhs: Self) -> Self {
                self.add_element_wise(rhs)
            }
            fn sub(self, rhs: Self) -> Self {
                self.sub_element_wise(rhs)
            }
            fn mul(self, rhs: Self) -> Self {
                self.mul_element_wise(rhs)
            }
            fn div(self, rhs: Self) -> Self {
                self.div_element_wise(rhs)
            }
        }
    };
}

implement_vector_base!(Vector2);
implement_vector_base!(Vector3);
implement_vector_base!(Vector4);

#[inline]
pub fn lerp_v2(t: f32, p0: Vector2, p1: Vector2) -> Vector2 {
    Vector2::new(p0.x + t * (p1.x - p0.x), p0.y + t * (p1.y - p0.y))
}

pub fn cartesian_to_spherical(cartesian: Vector3) -> Vector3 {
    let r: f32 = cartesian.length();
    if cartesian.x == 0. && cartesian.y == 0. {
        return Vector3::new(r, 0., 0.);
    }
    let mut theta: f32 = (cartesian.y / cartesian.x).atan();
    let phi: f32 = (Vector2::new(cartesian.x, cartesian.y).length() / cartesian.z).atan();
    if cartesian.x < 0. && cartesian.y >= 0. && theta == 0. {
        theta = PI;
    } else if cartesian.x < 0. && cartesian.y < 0. && theta.signum() > 0. {
        theta -= PI;
    } else if cartesian.x < 0. && cartesian.y > 0. && theta.signum() < 0. {
        theta += PI;
    }
    Vector3::new(r, theta, phi)
}

pub fn spherical_to_cartesian(spherical: Vector3) -> Vector3 {
    let (r, theta, phi) = (spherical.x, spherical.y, spherical.z);
    let x = r * phi.sin() * theta.cos();
    let y = r * phi.sin() * theta.sin();
    let z = r * phi.cos();
    Vector3::new(x, y, z)
}

pub fn direction_to_euler_angles(direction: Vector3) -> Vector3 {
    let dir = direction.normalize();
    let up_world: Vector3 = [0., 1., 0.].into();
    let right: Vector3 = dir.cross(up_world).normalize();
    let up: Vector3 = right.cross(dir).normalize();
    let angle_h = -dir.y.atan2(dir.x);
    let angle_p = dir.z.asin();
    let angle_b = (right.dot(up_world)).atan2(-up.dot(up_world));
    Vector3::new(angle_p, angle_h, angle_b)
}
