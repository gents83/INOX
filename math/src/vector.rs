use cgmath::*;

pub type Vector2 = cgmath::Vector2<f32>;
pub type Vector3 = cgmath::Vector3<f32>;
pub type Vector4 = cgmath::Vector4<f32>;

pub trait VecBase {
    fn default_zero() -> Self;
    fn squared_distance(self, other: Self) -> f32;
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
