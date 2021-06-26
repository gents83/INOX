use crate::vector::Vector3;
use cgmath::*;

pub type Matrix3 = cgmath::Matrix3<f32>;
pub type Matrix4 = cgmath::Matrix4<f32>;

pub trait MatBase {
    fn default_identity() -> Self;
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

implement_matrix_base!(Matrix3);
implement_matrix_base!(Matrix4);

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

pub fn unproject(position: Vector3, view: Matrix4, projection: Matrix4) -> Vector3 {
    let view_inverse = view.inverse_transform().unwrap();
    let proj_inverse = projection.inverse_transform().unwrap();
    let unprojected_point =
        view_inverse * proj_inverse * Vector4::new(position.x, position.y, position.z, 1.0);
    unprojected_point.xyz() / unprojected_point.w
}
