use crate::angle::NewAngle;
use crate::vector::{VecBaseFloat, Vector3, Vector4};
use crate::Degrees;
use crate::{Quat, Quaternion};
use cgmath::{Deg, InnerSpace, SquareMatrix, Transform};

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
    fn from_euler_angles(roll_yaw_pitch: Vector3) -> Self;
    fn inverse(&self) -> Self;
    fn transpose(&self) -> Self;
    fn set_translation(&mut self, translation: Vector3) -> &mut Self;
    fn add_translation(&mut self, translation: Vector3) -> &mut Self;
    fn add_scale(&mut self, scale: Vector3) -> &mut Self;
    fn add_rotation(&mut self, roll_yaw_pitch: Vector3) -> &mut Self;
    fn translation(&self) -> Vector3;
    fn scale(&self) -> Vector3;
    fn rotation(&self) -> Vector3;
    fn get_translation_rotation_scale(&self) -> (Vector3, Vector3, Vector3);
    fn from_translation_rotation_scale(
        translation: Vector3,
        roll_yaw_pitch: Vector3,
        scale: Vector3,
    ) -> Self
    where
        Self: Sized;
    fn look_at(&mut self, position: Vector3);
    fn look_towards(&mut self, direction: Vector3);
    fn get_direction(&self) -> Vector3;
    fn transform(&self, vec: Vector3) -> Vector3;
}

macro_rules! implement_matrix4_operations {
    ($MatrixN:ident) => {
        impl Mat4Ops for $MatrixN {
            #[inline]
            fn from_euler_angles(rotation: Vector3) -> Self {
                Matrix4::from(Quaternion::from_euler_angles(rotation))
            }
            #[inline]
            fn inverse(&self) -> Self {
                self.inverse_transform().unwrap()
            }
            #[inline]
            fn transpose(&self) -> Self {
                <Self as cgmath::Matrix>::transpose(self)
            }
            #[inline]
            fn set_translation(&mut self, translation: Vector3) -> &mut Self {
                self.w[0] = translation.x;
                self.w[1] = translation.y;
                self.w[2] = translation.z;
                self
            }
            #[inline]
            fn add_translation(&mut self, translation: Vector3) -> &mut Self {
                let p = self.translation();
                self.set_translation(p + translation)
            }
            #[inline]
            fn add_scale(&mut self, scale: Vector3) -> &mut Self {
                *self = *self * Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z);
                self
            }
            #[inline]
            fn add_rotation(&mut self, rotation: Vector3) -> &mut Self {
                *self = *self * Matrix4::from_euler_angles(rotation);
                self
            }
            #[inline]
            fn translation(&self) -> Vector3 {
                [self.w[0], self.w[1], self.w[2]].into()
            }
            #[inline]
            fn scale(&self) -> Vector3 {
                let s = Matrix3::from_cols(self.x.xyz(), self.y.xyz(), self.z.xyz());
                let sx = s.x.length();
                let sy = s.y.length();
                let sz = s.determinant().signum() * s.z.length();
                [sx, sy, sz].into()
            }
            #[inline]
            fn rotation(&self) -> Vector3 {
                let mut s = Matrix3::from_cols(self.x.xyz(), self.y.xyz(), self.z.xyz());
                let sx = s.x.length();
                let sy = s.y.length();
                let sz = s.determinant().signum() * s.z.length();
                s.x /= sx;
                s.y /= sy;
                s.z /= sz;
                let r = Quaternion::from(s);
                r.to_euler_angles()
            }
            #[inline]
            fn get_translation_rotation_scale(&self) -> (Vector3, Vector3, Vector3) {
                (self.translation(), self.rotation(), self.scale())
            }
            #[inline]
            fn from_translation_rotation_scale(
                translation: Vector3,
                rotation: Vector3, //in radians
                scale: Vector3,
            ) -> Self
            where
                Self: Sized,
            {
                let t = Matrix4::from_translation(translation);
                let r = Matrix4::from(Quaternion::from_euler_angles(rotation));
                let s = Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z);

                t * r * s
            }

            #[inline]
            fn transform(&self, vec: Vector3) -> Vector3 {
                let point = self.transform_point([vec.x, vec.y, vec.z].into());
                [point.x, point.y, point.z].into()
            }

            fn look_at(&mut self, target: Vector3) {
                let p = self.translation();
                let forward = (target - p).normalized();
                let mut up = Vector3::unit_y();
                if forward.dot(up) >= 1. - f32::EPSILON && forward.dot(up) <= 1. + f32::EPSILON {
                    up = Matrix4::from_angle_x(Degrees::new(90.)).transform(forward);
                };
                let right = up.cross(forward).normalized();
                up = forward.cross(right).normalize();
                let mut l: Matrix4 = Matrix3::from_cols(right, up, forward).into();
                l.set_translation(p);
                *self = l;
            }
            #[inline]
            fn look_towards(&mut self, direction: Vector3) {
                let position = self.translation();
                let target = position + direction.normalize();
                self.look_at(target)
            }

            #[inline]
            fn get_direction(&self) -> Vector3 {
                let rotation = self.rotation();
                let q = Quaternion::from_euler_angles(rotation);
                q.v.normalize()
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
#[inline]
pub fn matrix3_to_array(mat: Matrix3) -> [[f32; 3]; 3] {
    mat.into()
}

pub fn unproject(position: Vector3, view: Matrix4, projection: Matrix4) -> Vector3 {
    let view_inverse = view.inverse();
    let proj_inverse = projection.inverse();
    let unprojected_point =
        view_inverse * proj_inverse * Vector4::new(position.x, position.y, position.z, 1.0);
    unprojected_point.xyz() / unprojected_point.w
}

pub fn perspective(fovy: Deg<f32>, aspect: f32, near: f32, far: f32) -> Matrix4 {
    cgmath::perspective(fovy, aspect, near, far)
}
