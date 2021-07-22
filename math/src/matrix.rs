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

pub trait Mat4Ops {
    fn inverse(&self) -> Self;
    fn get_translation(&self) -> Vector3;
    fn get_scale(&self) -> Vector3;
    fn get_rotation(&self) -> Vector3;
    fn get_translation_rotation_scale(&self) -> (Vector3, Vector3, Vector3);
    fn transform(&self, vec: Vector3) -> Vector3;
}

macro_rules! implement_matrix4_operations {
    ($MatrixN:ident) => {
        impl Mat4Ops for $MatrixN {
            #[inline]
            fn inverse(&self) -> Self {
                self.inverse_transform().unwrap()
            }
            #[inline]
            fn get_translation(&self) -> Vector3 {
                [self.w[0], self.w[1], self.w[2]].into()
            }
            #[inline]
            fn get_scale(&self) -> Vector3 {
                //Extract scale
                let mut scale: Vector3 = [1., 1., 1.].into();
                let mut x_axis: Vector3 = [self.x[0], self.x[1], self.x[2]].into();
                let mut y_axis: Vector3 = [self.y[0], self.y[1], self.y[2]].into();
                let mut z_axis: Vector3 = [self.z[0], self.z[1], self.z[2]].into();
                scale.x =
                    self.w[3] * (x_axis.x.powf(2.) + x_axis.y.powf(2.) + x_axis.z.powf(2.)).sqrt();
                if !scale.x.is_zero() {
                    x_axis /= scale.x;
                }
                scale.y =
                    self.w[3] * (y_axis.x.powf(2.) + y_axis.y.powf(2.) + y_axis.z.powf(2.)).sqrt();
                if !scale.y.is_zero() {
                    y_axis /= scale.y;
                }
                scale.z =
                    self.w[3] * (z_axis.x.powf(2.) + z_axis.y.powf(2.) + z_axis.z.powf(2.)).sqrt();
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
            fn get_rotation(&self) -> Vector3 {
                //Extract scale
                let mut scale: Vector3 = [1., 1., 1.].into();
                let mut x_axis: Vector3 = [self.x[0], self.x[1], self.x[2]].into();
                let mut y_axis: Vector3 = [self.y[0], self.y[1], self.y[2]].into();
                let mut z_axis: Vector3 = [self.z[0], self.z[1], self.z[2]].into();
                scale.x =
                    self.w[3] * (x_axis.x.powf(2.) + x_axis.y.powf(2.) + x_axis.z.powf(2.)).sqrt();
                if !scale.x.is_zero() {
                    x_axis /= scale.x;
                }
                scale.y =
                    self.w[3] * (y_axis.x.powf(2.) + y_axis.y.powf(2.) + y_axis.z.powf(2.)).sqrt();
                if !scale.y.is_zero() {
                    y_axis /= scale.y;
                }
                scale.z =
                    self.w[3] * (z_axis.x.powf(2.) + z_axis.y.powf(2.) + z_axis.z.powf(2.)).sqrt();
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
            fn get_translation_rotation_scale(&self) -> (Vector3, Vector3, Vector3) {
                //Extract translation
                let translation: Vector3 = [self.w[0], self.w[1], self.w[2]].into();
                //Extract scale
                let mut scale: Vector3 = [1., 1., 1.].into();
                let mut x_axis: Vector3 = [self.x[0], self.x[1], self.x[2]].into();
                let mut y_axis: Vector3 = [self.y[0], self.y[1], self.y[2]].into();
                let mut z_axis: Vector3 = [self.z[0], self.z[1], self.z[2]].into();
                scale.x =
                    self.w[3] * (x_axis.x.powf(2.) + x_axis.y.powf(2.) + x_axis.z.powf(2.)).sqrt();
                if !scale.x.is_zero() {
                    x_axis /= scale.x;
                }
                scale.y =
                    self.w[3] * (y_axis.x.powf(2.) + y_axis.y.powf(2.) + y_axis.z.powf(2.)).sqrt();
                if !scale.y.is_zero() {
                    y_axis /= scale.y;
                }
                scale.z =
                    self.w[3] * (z_axis.x.powf(2.) + z_axis.y.powf(2.) + z_axis.z.powf(2.)).sqrt();
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
            fn transform(&self, vec: Vector3) -> Vector3 {
                let p: cgmath::Point3<f32> = self.transform_point([vec.x, vec.y, vec.x].into());
                [p.x, p.y, p.z].into()
            }
        }
    };
}

implement_matrix_base!(Matrix3);
implement_matrix_base!(Matrix4);
implement_matrix4_operations!(Matrix4);

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
    let view_inverse = view.inverse();
    let proj_inverse = projection.inverse();
    let unprojected_point =
        view_inverse * proj_inverse * Vector4::new(position.x, position.y, position.z, 1.0);
    unprojected_point.xyz() / unprojected_point.w
}
