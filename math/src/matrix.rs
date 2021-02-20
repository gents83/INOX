#![allow(dead_code)]
#![allow(unused_must_use)]
#![allow(clippy::clippy::too_many_arguments)]

use super::angle::*;
use super::float::*;
use super::number::*;
use super::vector::*;
use super::zero::*;
use nrg_serialize::*;

#[derive(Serialize, Deserialize, PartialOrd, PartialEq, Eq, Clone, Copy)]
#[serde(crate = "nrg_serialize")]
pub struct Matrix3<T> {
    pub axis_x: Vector3<T>,
    pub axis_y: Vector3<T>,
    pub axis_z: Vector3<T>,
}

#[derive(Serialize, Deserialize, PartialOrd, PartialEq, Eq, Clone, Copy)]
#[serde(crate = "nrg_serialize")]
pub struct Matrix4<T> {
    pub axis_x: Vector4<T>,
    pub axis_y: Vector4<T>,
    pub axis_z: Vector4<T>,
    pub axis_w: Vector4<T>,
}

macro_rules! implement_matrix {
    ($MatrixN:ident<$T:ident> { $($field:ident : $index:expr),+ }, $n:expr, $VecType:ident) => {
        impl<$T> $MatrixN<T>
        where $T: Float {
            #[inline]
            pub fn from_columns($($field: $VecType<$T>),+) -> $MatrixN<$T> {
                $MatrixN { $($field),+ }
            }
            #[inline]
            pub fn from_axis( &v: &[$VecType<$T>; $n] ) -> $MatrixN<$T> {
                match v { [$($field),+] => $MatrixN { $($field),+ } }
            }
            #[inline]
            pub fn from_uniform_scale(s: T ) -> $MatrixN<$T> {
                $MatrixN::from_scale_vector($VecType::from(s))
            }
            #[inline]
            pub fn identity() -> $MatrixN<$T> {
                $MatrixN::from_uniform_scale(T::one())
            }
        }

        impl<$T> Into<[[$T; $n]; $n]> for $MatrixN<$T>
        where T: Number {
            #[inline]
            fn into(self) -> [[$T; $n]; $n] {
                match self { $MatrixN { $($field),+ } => [$($field.into()),+] }
            }
        }

        impl<$T> AsRef<[[$T; $n]; $n]> for $MatrixN<$T> {
            #[inline]
            fn as_ref(&self) -> &[[$T; $n]; $n] {
                unsafe { &*(self as *const $MatrixN<$T> as *const [[$T; $n]; $n]) }
            }
        }

        impl<$T> AsMut<[[$T; $n]; $n]> for $MatrixN<$T> {
            #[inline]
            fn as_mut(&mut self) -> &mut [[$T; $n]; $n] {
                unsafe { &mut *(self as *mut $MatrixN<$T> as *mut [[$T; $n]; $n]) }
            }
        }

        impl<'a, $T> From<&'a [[$T; $n]; $n]> for &'a $MatrixN<$T> {
            #[inline]
            fn from(m: &'a [[$T; $n]; $n]) -> &'a $MatrixN<$T> {
                unsafe { &*(m as *const [[$T; $n]; $n] as *const $MatrixN<$T>) }
            }
        }

        impl<'a, $T> From<&'a mut [[$T; $n]; $n]> for &'a mut $MatrixN<$T> {
            #[inline]
            fn from(m: &'a mut [[$T; $n]; $n]) -> &'a mut $MatrixN<$T> {
                unsafe { &mut *(m as *mut [[$T; $n]; $n] as *mut $MatrixN<$T>) }
            }
        }

        impl<$T> From<[[$T; $n]; $n]> for $MatrixN<$T>
        where T: Number{
            #[inline]
            fn from(m: [[$T; $n]; $n]) -> $MatrixN<$T> {
                $MatrixN { $($field: From::from(m[$index])),+ }
            }
        }

        impl<$T> AsRef<[$T; ($n * $n)]> for $MatrixN<$T> {
            #[inline]
            fn as_ref(&self) -> &[$T; ($n * $n)] {
                unsafe { &*(self as *const $MatrixN<$T> as *const [$T; ($n * $n)]) }
            }
        }

        impl<$T> AsMut<[$T; ($n * $n)]> for $MatrixN<$T> {
            #[inline]
            fn as_mut(&mut self) -> &mut [$T; ($n * $n)] {
                unsafe { &mut *(self as *mut $MatrixN<$T> as *mut [$T; ($n * $n)]) }
            }
        }

        impl<'a, $T> From<&'a [$T; ($n * $n)]> for &'a $MatrixN<$T> {
            #[inline]
            fn from(m: &'a [$T; ($n * $n)]) -> &'a $MatrixN<$T> {
                unsafe { &*(m as *const [$T; ($n * $n)] as *const $MatrixN<$T>) }
            }
        }

        impl<'a, $T> From<&'a mut [$T; ($n * $n)]> for &'a mut $MatrixN<$T> {
            #[inline]
            fn from(m: &'a mut [$T; ($n * $n)]) -> &'a mut $MatrixN<$T> {
                unsafe { &mut *(m as *mut [$T; ($n * $n)] as *mut $MatrixN<$T>) }
            }
        }

        impl<T, Idx> ::std::ops::Index<Idx> for $MatrixN<T>
        where T: Number, Idx: std::slice::SliceIndex<[[T;$n]]> + std::slice::SliceIndex<[[T;$n]], Output = [T;$n]> {
            type Output = [T;$n];

            #[inline]
            fn index<'a>(&'a self, i: Idx) -> &'a [T;$n] {
                let v: &[[T;$n]; $n] = self.as_ref();
                &v[i]
            }
        }

        impl<T, Idx> ::std::ops::IndexMut<Idx> for $MatrixN<T>
        where T: Number, Idx: std::slice::SliceIndex<[[T;$n]]> + std::slice::SliceIndex<[[T;$n]], Output = [T;$n]> {
            #[inline]
            fn index_mut<'a>(&'a mut self, i: Idx) -> &'a mut [T;$n] {
                let v: &mut [[T;$n]; $n] = self.as_mut();
                &mut v[i]
            }
        }

        impl<$T> ::std::fmt::Debug for $MatrixN<$T>
        where $T: Float {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                $(self.$field.fmt(f)); +
            }
        }
    }
}

impl<T> ::std::ops::Mul for Matrix3<T>
where
    T: Float,
{
    type Output = Matrix3<T>;
    #[inline]
    fn mul(self, rhs: Matrix3<T>) -> Matrix3<T> {
        Matrix3::new(
            self.row(0).dot(rhs[0].into()),
            self.row(1).dot(rhs[0].into()),
            self.row(2).dot(rhs[0].into()),
            self.row(0).dot(rhs[1].into()),
            self.row(1).dot(rhs[1].into()),
            self.row(2).dot(rhs[1].into()),
            self.row(0).dot(rhs[2].into()),
            self.row(1).dot(rhs[2].into()),
            self.row(2).dot(rhs[2].into()),
        )
    }
}

impl<T> ::std::ops::Mul for Matrix4<T>
where
    T: Float,
{
    type Output = Matrix4<T>;
    #[inline]
    fn mul(self, rhs: Matrix4<T>) -> Matrix4<T> {
        let a: Vector4<T> = self[0].into();
        let b: Vector4<T> = self[1].into();
        let c: Vector4<T> = self[2].into();
        let d: Vector4<T> = self[3].into();

        Matrix4::from_columns(
            a * rhs[0][0] + b * rhs[0][1] + c * rhs[0][2] + d * rhs[0][3],
            a * rhs[1][0] + b * rhs[1][1] + c * rhs[1][2] + d * rhs[1][3],
            a * rhs[2][0] + b * rhs[2][1] + c * rhs[2][2] + d * rhs[2][3],
            a * rhs[3][0] + b * rhs[3][1] + c * rhs[3][2] + d * rhs[3][3],
        )
    }
}

implement_matrix!(Matrix3<T> { axis_x: 0, axis_y: 1, axis_z: 2 }, 3, Vector3);
implement_matrix!(Matrix4<T> { axis_x: 0, axis_y: 1, axis_z: 2, axis_w: 3 }, 4, Vector4);

pub type Matrix3f = Matrix3<f32>;
pub type Matrix4f = Matrix4<f32>;

impl<T> Matrix3<T>
where
    T: Float,
{
    #[inline]
    pub fn new(
        c0r0: T,
        c0r1: T,
        c0r2: T,
        c1r0: T,
        c1r1: T,
        c1r2: T,
        c2r0: T,
        c2r1: T,
        c2r2: T,
    ) -> Matrix3<T> {
        Matrix3::from_columns(
            Vector3::new(c0r0, c0r1, c0r2),
            Vector3::new(c1r0, c1r1, c1r2),
            Vector3::new(c2r0, c2r1, c2r2),
        )
    }
    #[inline]
    pub fn row(&self, r: usize) -> Vector3<T> {
        Vector3::new(self[0][r], self[1][r], self[2][r])
    }
    #[inline]
    pub fn from_translation(v: Vector2<T>) -> Matrix3<T> {
        Self {
            axis_x: Vector3 {
                x: T::one(),
                y: T::zero(),
                z: T::zero(),
            },
            axis_y: Vector3 {
                x: T::zero(),
                y: T::one(),
                z: T::zero(),
            },
            axis_z: Vector3 {
                x: v.x,
                y: v.y,
                z: T::one(),
            },
        }
    }
    #[inline]
    pub fn set_translation(&mut self, pos: Vector2<T>) {
        self[2][0] = pos.x;
        self[2][1] = pos.y;
    }
    #[inline]
    pub fn from_scale_vector(v: Vector3<T>) -> Matrix3<T> {
        Self {
            axis_x: Vector3 {
                x: v.x,
                y: T::zero(),
                z: T::zero(),
            },
            axis_y: Vector3 {
                x: T::zero(),
                y: v.y,
                z: T::zero(),
            },
            axis_z: Vector3 {
                x: T::zero(),
                y: T::zero(),
                z: T::one(),
            },
        }
    }
    #[inline]
    pub fn from_scale(x: T, y: T) -> Matrix3<T> {
        Self {
            axis_x: Vector3 {
                x,
                y: T::zero(),
                z: T::zero(),
            },
            axis_y: Vector3 {
                x: T::zero(),
                y,
                z: T::zero(),
            },
            axis_z: Vector3 {
                x: T::zero(),
                y: T::zero(),
                z: T::one(),
            },
        }
    }
    #[inline]
    pub fn get_transpose(&self) -> Matrix3<T> {
        Matrix3::new(
            self[0][0], self[1][0], self[2][0], self[0][1], self[1][1], self[2][1], self[0][2],
            self[1][2], self[2][2],
        )
    }
    pub fn from_look_at(direction: Vector3<T>, up: Vector3<T>) -> Matrix3<T> {
        let dir = direction.get_normalized();
        let side = dir.cross(up).get_normalized();
        let up = side.cross(dir).get_normalized();
        Matrix3::from_columns(side, up, dir)
    }
}

impl<T> Matrix4<T>
where
    T: Float,
{
    #[inline]
    pub fn new(
        c0r0: T,
        c0r1: T,
        c0r2: T,
        c0r3: T,
        c1r0: T,
        c1r1: T,
        c1r2: T,
        c1r3: T,
        c2r0: T,
        c2r1: T,
        c2r2: T,
        c2r3: T,
        c3r0: T,
        c3r1: T,
        c3r2: T,
        c3r3: T,
    ) -> Matrix4<T> {
        Matrix4::from_columns(
            Vector4::new(c0r0, c0r1, c0r2, c0r3),
            Vector4::new(c1r0, c1r1, c1r2, c1r3),
            Vector4::new(c2r0, c2r1, c2r2, c2r3),
            Vector4::new(c3r0, c3r1, c3r2, c3r3),
        )
    }
    #[inline]
    pub fn row(&self, r: usize) -> Vector4<T> {
        Vector4::new(self[0][r], self[1][r], self[2][r], self[3][r])
    }
    #[inline]
    pub fn from_translation(v: Vector3<T>) -> Matrix4<T> {
        Self {
            axis_x: Vector4 {
                x: T::one(),
                y: T::zero(),
                z: T::zero(),
                w: T::zero(),
            },
            axis_y: Vector4 {
                x: T::zero(),
                y: T::one(),
                z: T::zero(),
                w: T::zero(),
            },
            axis_z: Vector4 {
                x: T::zero(),
                y: T::zero(),
                z: T::one(),
                w: T::zero(),
            },
            axis_w: Vector4 {
                x: v.x,
                y: v.y,
                z: v.z,
                w: T::one(),
            },
        }
    }
    #[inline]
    pub fn set_translation(&mut self, pos: Vector3<T>) {
        self[3][0] = pos.x;
        self[3][1] = pos.y;
        self[3][2] = pos.z;
    }
    #[inline]
    pub fn from_scale_vector(v: Vector4<T>) -> Matrix4<T> {
        Self {
            axis_x: Vector4 {
                x: v.x,
                y: T::zero(),
                z: T::zero(),
                w: T::zero(),
            },
            axis_y: Vector4 {
                x: T::zero(),
                y: v.y,
                z: T::zero(),
                w: T::zero(),
            },
            axis_z: Vector4 {
                x: T::zero(),
                y: T::zero(),
                z: v.z,
                w: T::zero(),
            },
            axis_w: Vector4 {
                x: T::zero(),
                y: T::zero(),
                z: T::zero(),
                w: T::one(),
            },
        }
    }
    #[inline]
    pub fn from_scale(x: T, y: T, z: T) -> Matrix4<T> {
        Self {
            axis_x: Vector4 {
                x,
                y: T::zero(),
                z: T::zero(),
                w: T::zero(),
            },
            axis_y: Vector4 {
                x: T::zero(),
                y,
                z: T::zero(),
                w: T::zero(),
            },
            axis_z: Vector4 {
                x: T::zero(),
                y: T::zero(),
                z,
                w: T::zero(),
            },
            axis_w: Vector4 {
                x: T::zero(),
                y: T::zero(),
                z: T::zero(),
                w: T::one(),
            },
        }
    }
    pub fn from_look_at(pos: Vector3<T>, target: Vector3<T>, up: Vector3<T>) -> Matrix4<T> {
        Matrix4::from_look_at_dir(pos, target - pos, up)
    }
    pub fn from_look_at_dir(pos: Vector3<T>, direction: Vector3<T>, up: Vector3<T>) -> Matrix4<T> {
        let dir = direction.get_normalized();
        let side = dir.cross(up).get_normalized();
        let up = side.cross(dir).get_normalized();
        Matrix4::new(
            side.x,
            up.x,
            -dir.x,
            T::zero(),
            side.y,
            up.y,
            -dir.y,
            T::zero(),
            side.z,
            up.z,
            -dir.z,
            T::zero(),
            -pos.dot(side),
            -pos.dot(up),
            pos.dot(dir),
            T::one(),
        )
    }
    pub fn from_axis_angle(axis: Vector3<T>, angle_in_radians: Radians<T>) -> Matrix4<T> {
        let sin = angle_in_radians.0.sin();
        let cos = angle_in_radians.0.cos();
        let _1subc = T::one() - cos;

        Matrix4::new(
            _1subc * axis.x * axis.x + cos,
            _1subc * axis.x * axis.y + sin * axis.z,
            _1subc * axis.x * axis.z - sin * axis.y,
            T::zero(),
            _1subc * axis.x * axis.y - sin * axis.z,
            _1subc * axis.y * axis.y + cos,
            _1subc * axis.y * axis.z + sin * axis.x,
            T::zero(),
            _1subc * axis.x * axis.z + sin * axis.y,
            _1subc * axis.y * axis.z - sin * axis.x,
            _1subc * axis.z * axis.z + cos,
            T::zero(),
            T::zero(),
            T::zero(),
            T::zero(),
            T::one(),
        )
    }
    pub fn create_perspective(
        fovy_in_radians: Radians<T>,
        aspect_ratio: T,
        near_plane: T,
        far_plane: T,
    ) -> Matrix4<T> {
        assert!(
            fovy_in_radians > Radians::zero(),
            "The vertical field of view cannot be below zero, FoV: {:?}",
            Degree(fovy_in_radians).0
        );
        assert!(
            fovy_in_radians < Radians::half_pi(),
            "The vertical field of view cannot be greater than 180, FoV: {:?}",
            Degree(fovy_in_radians).0
        );
        assert!(
            aspect_ratio.abs() != T::zero(),
            "The absolute aspect ratio cannot be zero, aspect_ratio: {:?}",
            aspect_ratio.abs()
        );
        assert!(
            near_plane > T::zero(),
            "The near plane distance cannot be below zero, near_plane: {:?}",
            near_plane
        );
        assert!(
            far_plane > T::zero(),
            "The far plane distance cannot be below zero, far_plane: {:?}",
            far_plane
        );
        assert!(
            far_plane != near_plane,
            "The far plane cannot be equal to near plane, far_plane: {:?}, near_plane: {:?} ",
            far_plane,
            near_plane
        );

        let two: T = T::from(2).unwrap();
        let f = (fovy_in_radians / two).0.tan().recip(); //cotangent

        let c0r0 = f / aspect_ratio;
        let c0r1 = T::zero();
        let c0r2 = T::zero();
        let c0r3 = T::zero();

        let c1r0 = T::zero();
        let c1r1 = f;
        let c1r2 = T::zero();
        let c1r3 = T::zero();

        let c2r0 = T::zero();
        let c2r1 = T::zero();
        let c2r2 = (far_plane + near_plane) / (near_plane - far_plane);
        let c2r3 = -T::one();

        let c3r0 = T::zero();
        let c3r1 = T::zero();
        let c3r2 = (two * far_plane * near_plane) / (near_plane - far_plane);
        let c3r3 = T::zero();

        Matrix4::new(
            c0r0, c0r1, c0r2, c0r3, c1r0, c1r1, c1r2, c1r3, c2r0, c2r1, c2r2, c2r3, c3r0, c3r1,
            c3r2, c3r3,
        )
    }
    pub fn create_frustum(
        left: T,
        right: T,
        bottom: T,
        top: T,
        near_plane: T,
        far_plane: T,
    ) -> Matrix4<T> {
        assert!(
            left <= right,
            "left cannot be greater than right, left: {:?} right: {:?}",
            left,
            right
        );
        assert!(
            bottom <= top,
            "bottom cannot be greater than top, bottom: {:?} top: {:?}",
            bottom,
            top
        );
        assert!(
            near_plane <= far_plane,
            "near cannot be greater than far, near: {:?} far: {:?}",
            near_plane,
            far_plane
        );

        let two: T = T::from(2).unwrap();

        let c0r0 = (two * near_plane) / (right - left);
        let c0r1 = T::zero();
        let c0r2 = T::zero();
        let c0r3 = T::zero();

        let c1r0 = T::zero();
        let c1r1 = (two * near_plane) / (top - bottom);
        let c1r2 = T::zero();
        let c1r3 = T::zero();

        let c2r0 = (right + left) / (right - left);
        let c2r1 = (top + bottom) / (top - bottom);
        let c2r2 = -(far_plane + near_plane) / (far_plane - near_plane);
        let c2r3 = -T::one();

        let c3r0 = T::zero();
        let c3r1 = T::zero();
        let c3r2 = -(two * far_plane * near_plane) / (far_plane - near_plane);
        let c3r3 = T::zero();

        Matrix4::new(
            c0r0, c0r1, c0r2, c0r3, c1r0, c1r1, c1r2, c1r3, c2r0, c2r1, c2r2, c2r3, c3r0, c3r1,
            c3r2, c3r3,
        )
    }
    pub fn create_orthographic(
        left: T,
        right: T,
        bottom: T,
        top: T,
        near_plane: T,
        far_plane: T,
    ) -> Matrix4<T> {
        assert!(
            left <= right,
            "left cannot be greater than right, left: {:?} right: {:?}",
            left,
            right
        );
        assert!(
            bottom <= top,
            "bottom cannot be greater than top, bottom: {:?} top: {:?}",
            bottom,
            top
        );
        assert!(
            near_plane <= far_plane,
            "near cannot be greater than far, near: {:?} far: {:?}",
            near_plane,
            far_plane
        );
        let two: T = T::from(2).unwrap();

        let c0r0 = two / (right - left);
        let c0r1 = T::zero();
        let c0r2 = T::zero();
        let c0r3 = T::zero();

        let c1r0 = T::zero();
        let c1r1 = two / (top - bottom);
        let c1r2 = T::zero();
        let c1r3 = T::zero();

        let c2r0 = T::zero();
        let c2r1 = T::zero();
        let c2r2 = -two / (far_plane - near_plane);
        let c2r3 = T::zero();

        let c3r0 = -(right + left) / (right - left);
        let c3r1 = -(top + bottom) / (top - bottom);
        let c3r2 = -(far_plane + near_plane) / (far_plane - near_plane);
        let c3r3 = T::one();

        Matrix4::new(
            c0r0, c0r1, c0r2, c0r3, c1r0, c1r1, c1r2, c1r3, c2r0, c2r1, c2r2, c2r3, c3r0, c3r1,
            c3r2, c3r3,
        )
    }
}
