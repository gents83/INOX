#![warn(clippy::all)]

pub use self::triangle::*;

pub use cgmath::*;

//Modules

pub mod triangle;

pub trait VecBase {
    fn default_zero() -> Self;
    fn squared_distance(self, other: Self) -> f32;
    fn add(self, rhs: Self) -> Self;
    fn sub(self, rhs: Self) -> Self;
    fn mul(self, rhs: Self) -> Self;
    fn div(self, rhs: Self) -> Self;
}

pub trait MatBase {
    fn default_identity() -> Self;
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

macro_rules! implement_matrix_base {
    ($MatrixN:ident) => {
        impl MatBase for $MatrixN {
            fn default_identity() -> Self {
                Self::identity()
            }
        }
    };
}

pub type Vector2 = cgmath::Vector2<f32>;
pub type Vector3 = cgmath::Vector3<f32>;
pub type Vector4 = cgmath::Vector4<f32>;

pub type Matrix3 = cgmath::Matrix3<f32>;
pub type Matrix4 = cgmath::Matrix4<f32>;

pub type Quaternion = cgmath::Quaternion<f32>;

implement_vector_base!(Vector2);
implement_vector_base!(Vector3);
implement_vector_base!(Vector4);

implement_matrix_base!(Matrix3);
implement_matrix_base!(Matrix4);

#[inline]
pub fn lerp_v2(t: f32, p0: Vector2, p1: Vector2) -> Vector2 {
    Vector2::new(p0.x + t * (p1.x - p0.x), p0.y + t * (p1.y - p0.y))
}

pub fn get_translation_rotation_scale(mat: &Matrix4) -> (Vector3, Vector3, Vector3) {
    //Extract translation
    let translation: Vector3 = [mat.w[0], mat.w[1], mat.w[2]].into();
    //Extract scale
    let mut scale: Vector3 = [1., 1., 1.].into();
    let mut x_axis: Vector3 = [mat.x[0], mat.x[1], mat.x[2]].into();
    let mut y_axis: Vector3 = [mat.y[0], mat.y[1], mat.y[2]].into();
    let mut z_axis: Vector3 = [mat.z[0], mat.z[1], mat.z[2]].into();
    scale.x = mat.w[3] * (x_axis.x.powf(2.) + x_axis.y.powf(2.) + x_axis.z.powf(2.)).sqrt();
    if !scale.x.is_zero() {
        x_axis /= scale.x;
    }
    scale.y = mat.w[3] * (y_axis.x.powf(2.) + y_axis.y.powf(2.) + y_axis.z.powf(2.)).sqrt();
    if !scale.y.is_zero() {
        y_axis /= scale.y;
    }
    scale.z = mat.w[3] * (z_axis.x.powf(2.) + z_axis.y.powf(2.) + z_axis.z.powf(2.)).sqrt();
    if !scale.z.is_zero() {
        z_axis /= scale.z;
    }
    //Verify orientation and if necessary invert
    let z_cross = x_axis.cross(y_axis);
    if z_cross.dot(z_axis) < 0. {
        scale.x *= -1.;
        x_axis = -x_axis;
    }
    //Extract rotation
    let theta1 = y_axis.z.atan2(z_axis.z);
    let c2 = (x_axis.x.powf(2.) + x_axis.y.powf(2.)).sqrt();
    let theta2 = (-x_axis.z).atan2(c2);
    let s1 = theta1.sin();
    let c1 = theta1.cos();
    let theta3 = (s1 * z_axis.x - c1 * y_axis.x).atan2(c1 * y_axis.y - s1 * z_axis.y);
    let rotation: Vector3 = [-theta1, -theta2, -theta3].into();
    (translation, rotation, scale)
}

pub fn create_perspective<S: BaseFloat>(
    field_of_view: S,
    aspect_ratio: S,
    near_plane: S,
    far_plane: S,
) -> cgmath::Matrix4<S> {
    let half: S = num_traits::cast(0.5).unwrap();
    let f: S = S::one() / S::tan(half * field_of_view);

    cgmath::Matrix4::new(
        f / aspect_ratio,
        S::zero(),
        S::zero(),
        S::zero(),
        S::zero(),
        -f,
        S::zero(),
        S::zero(),
        S::zero(),
        S::zero(),
        far_plane / (near_plane - far_plane),
        -S::one(),
        S::zero(),
        S::zero(),
        (near_plane * far_plane) / (near_plane - far_plane),
        S::zero(),
    )
}

pub fn matrix4_to_array(mat: Matrix4) -> [[f32; 4]; 4] {
    mat.into()
}
