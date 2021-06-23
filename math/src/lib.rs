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

pub type Degrees = cgmath::Deg<f32>;
pub type Radians = cgmath::Rad<f32>;

pub trait NewAngle {
    fn new(angle: f32) -> Self;
}

impl NewAngle for Degrees {
    fn new(angle_in_degrees: f32) -> Degrees {
        cgmath::Deg(angle_in_degrees)
    }
}
impl NewAngle for Radians {
    fn new(angle_in_radians: f32) -> Radians {
        cgmath::Rad(angle_in_radians)
    }
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

#[inline]
pub fn get_translation(mat: &Matrix4) -> Vector3 {
    [mat.w[0], mat.w[1], mat.w[2]].into()
}

#[inline]
pub fn get_scale(mat: &Matrix4) -> Vector3 {
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
    }
    scale
}

#[inline]
pub fn get_rotation(mat: &Matrix4) -> Vector3 {
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
    rotation
}

#[inline]
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

#[inline]
pub fn matrix4_to_array(mat: Matrix4) -> [[f32; 4]; 4] {
    mat.into()
}

pub fn create_look_at(eye: Vector3, target: Vector3, up: Vector3) -> Matrix4 {
    Matrix4::look_at_rh(
        [eye.x, eye.y, eye.z].into(),
        [target.x, target.y, target.z].into(),
        up,
    )
}
