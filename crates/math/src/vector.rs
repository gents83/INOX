use std::f32::consts::PI;

use cgmath::prelude::*;
use cgmath::InnerSpace;
use cgmath::{Deg, Rad};

pub type Vector2 = cgmath::Vector2<f32>;
pub type Vector3 = cgmath::Vector3<f32>;
pub type Vector4 = cgmath::Vector4<f32>;
pub type Vector2u = cgmath::Vector2<u32>;
pub type Vector3u = cgmath::Vector3<u32>;
pub type Vector4u = cgmath::Vector4<u32>;
pub type Vector2h = cgmath::Vector2<u16>;
pub type Vector3h = cgmath::Vector3<u16>;
pub type Vector4h = cgmath::Vector4<u16>;

pub trait VecBase<T> {
    fn default_zero() -> Self;
    fn default_one() -> Self;
    fn default_value(v: T) -> Self;
    fn squared_distance(self, other: Self) -> T;
    fn add(self, rhs: Self) -> Self;
    fn sub(self, rhs: Self) -> Self;
    fn mul(self, rhs: Self) -> Self;
    fn div(self, rhs: Self) -> Self;
    fn dot_product(self, rhs: Self) -> T;
}
pub trait VecBaseFloat<T> {
    fn length(self) -> T;
    fn normalized(self) -> Self;
    fn to_degrees(self) -> Self;
    fn to_radians(self) -> Self;
}

macro_rules! implement_vector_base {
    ($VectorN:ident, $Type:ty) => {
        impl VecBase<$Type> for $VectorN {
            fn default_zero() -> Self {
                Self::zero()
            }
            fn default_value(v: $Type) -> Self {
                Self::from_value(v)
            }
            fn default_one() -> Self {
                Self::from_value(1. as $Type)
            }
            fn squared_distance(self, other: Self) -> $Type {
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
            fn dot_product(self, rhs: Self) -> $Type {
                self.dot(rhs)
            }
        }
    };
}

macro_rules! implement_vector_base_float {
    ($VectorN:ident, $Type:ty) => {
        impl VecBaseFloat<$Type> for $VectorN {
            fn length(self) -> $Type {
                self.magnitude()
            }
            fn normalized(self) -> Self {
                self.normalize()
            }
            fn to_degrees(self) -> Self {
                self.map(|f| Deg::from(Rad(f)).0)
            }
            fn to_radians(self) -> Self {
                self.map(|f| Rad::from(Deg(f)).0)
            }
        }
    };
}

implement_vector_base!(Vector2, f32);
implement_vector_base!(Vector3, f32);
implement_vector_base!(Vector4, f32);
implement_vector_base_float!(Vector2, f32);
implement_vector_base_float!(Vector3, f32);
implement_vector_base_float!(Vector4, f32);

implement_vector_base!(Vector2u, u32);
implement_vector_base!(Vector3u, u32);
implement_vector_base!(Vector4u, u32);
implement_vector_base!(Vector2h, u16);
implement_vector_base!(Vector3h, u16);
implement_vector_base!(Vector4h, u16);

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
    let w: Vector3 = Vector3::new(-dir.y, dir.x, 0.);
    let up: Vector3 = w.cross(dir);
    let angle_h = dir.y.atan2(dir.x);
    let angle_p = dir.z.asin();
    let angle_b = (w.dot(up_world) / w.length()).atan2(up.dot(up_world) / up.length());
    Vector3::new(angle_b, angle_p, angle_h)
}
