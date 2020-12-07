#![allow(dead_code)]
#![allow(unused_must_use)]

use super::float::*;
use super::number::*;
use super::vector::*;

#[derive(PartialEq, Eq, Copy, Clone, Hash)]
pub struct Matrix3<T> {
    pub axis_x: Vector3<T>,
    pub axis_y: Vector3<T>,
    pub axis_z: Vector3<T>,
}

#[derive(PartialEq, Eq, Copy, Clone, Hash)]
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
                $MatrixN { $($field: $field),+ }
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
                unsafe { ::std::mem::transmute(self) }
            }
        }

        impl<$T> AsMut<[[$T; $n]; $n]> for $MatrixN<$T> {
            #[inline]
            fn as_mut(&mut self) -> &mut [[$T; $n]; $n] {
                unsafe { ::std::mem::transmute(self) }
            }
        }

        impl<'a, $T> From<&'a [[$T; $n]; $n]> for &'a $MatrixN<$T> {
            #[inline]
            fn from(m: &'a [[$T; $n]; $n]) -> &'a $MatrixN<$T> {
                unsafe { ::std::mem::transmute(m) }
            }
        }

        impl<'a, $T> From<&'a mut [[$T; $n]; $n]> for &'a mut $MatrixN<$T> {
            #[inline]
            fn from(m: &'a mut [[$T; $n]; $n]) -> &'a mut $MatrixN<$T> {
                unsafe { ::std::mem::transmute(m) }
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
                unsafe { ::std::mem::transmute(self) }
            }
        }

        impl<$T> AsMut<[$T; ($n * $n)]> for $MatrixN<$T> {
            #[inline]
            fn as_mut(&mut self) -> &mut [$T; ($n * $n)] {
                unsafe { ::std::mem::transmute(self) }
            }
        }

        impl<'a, $T> From<&'a [$T; ($n * $n)]> for &'a $MatrixN<$T> {
            #[inline]
            fn from(m: &'a [$T; ($n * $n)]) -> &'a $MatrixN<$T> {
                unsafe { ::std::mem::transmute(m) }
            }
        }

        impl<'a, $T> From<&'a mut [$T; ($n * $n)]> for &'a mut $MatrixN<$T> {
            #[inline]
            fn from(m: &'a mut [$T; ($n * $n)]) -> &'a mut $MatrixN<$T> {
                unsafe { ::std::mem::transmute(m) }
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

implement_matrix!(Matrix3<T> { axis_x: 0, axis_y: 1, axis_z: 2 }, 3, Vector3);
implement_matrix!(Matrix4<T> { axis_x: 0, axis_y: 1, axis_z: 2, axis_w: 3 }, 4, Vector4);

pub type Matrix3f = Matrix3<f32>;
pub type Matrix4f = Matrix4<f32>;


impl<T> Matrix3<T> 
where T: Float {
    #[inline]
    pub fn new(
        c0r0:T, c0r1:T, c0r2:T,
        c1r0:T, c1r1:T, c1r2:T,
        c2r0:T, c2r1:T, c2r2:T,
    ) -> Matrix3<T> {
        Matrix3::from_columns(
            Vector3::new(c0r0, c0r1, c0r2),
            Vector3::new(c1r0, c1r1, c1r2),
            Vector3::new(c2r0, c2r1, c2r2),
        )
    }
    #[inline]
    pub fn from_translation(v: Vector2<T>) -> Matrix3<T> {
        Self {  axis_x: Vector3 {x: T::one(), y: T::zero(), z: T::zero()},
                axis_y: Vector3 {x: T::zero(), y: T::one(), z: T::zero()},
                axis_z: Vector3 {x: v.x, y: v.y, z: T::one()} }
    }
    #[inline]
    pub fn from_scale_vector(v: Vector3<T>) -> Matrix3<T> {
        Self {  axis_x: Vector3 {x: v.x, y: T::zero(), z: T::zero()},
                axis_y: Vector3 {x: T::zero(), y: v.y, z: T::zero()},
                axis_z: Vector3 {x: T::zero(), y: T::zero(), z: T::one()} }
    }
    #[inline]
    pub fn from_scale(x: T, y:T ) -> Matrix3<T> {
        Self {  axis_x: Vector3 {x: x, y: T::zero(), z: T::zero()},
                axis_y: Vector3 {x: T::zero(), y: y, z: T::zero()},
                axis_z: Vector3 {x: T::zero(), y: T::zero(), z: T::one()} }
    }    
    #[inline]
    fn get_transpose(&self) -> Matrix3<T> {
        Matrix3::new(
            self[0][0], self[1][0], self[2][0],
            self[0][1], self[1][1], self[2][1],
            self[0][2], self[1][2], self[2][2],
        )
    }
}

impl<T> Matrix4<T> 
where T: Float {
    #[inline]
    pub fn new(
        c0r0: T, c0r1: T, c0r2: T, c0r3: T,
        c1r0: T, c1r1: T, c1r2: T, c1r3: T,
        c2r0: T, c2r1: T, c2r2: T, c2r3: T,
        c3r0: T, c3r1: T, c3r2: T, c3r3: T,
    ) -> Matrix4<T>  {
        Matrix4::from_columns(
            Vector4::new(c0r0, c0r1, c0r2, c0r3),
            Vector4::new(c1r0, c1r1, c1r2, c1r3),
            Vector4::new(c2r0, c2r1, c2r2, c2r3),
            Vector4::new(c3r0, c3r1, c3r2, c3r3),
        )
    }
    #[inline]
    pub fn from_translation(v: Vector3<T>) -> Matrix4<T> {
        Self {  axis_x: Vector4 {x: T::one(), y: T::zero(), z: T::zero(), w: T::zero()},
                axis_y: Vector4 {x: T::zero(), y: T::one(), z: T::zero(), w: T::zero()},
                axis_z: Vector4 {x: T::zero(), y: T::zero(), z: T::one(), w: T::zero()},
                axis_w: Vector4 {x: v.x, y: v.y, z: v.z, w: T::one()}  }
    }
    #[inline]
    pub fn from_scale_vector(v: Vector4<T>) -> Matrix4<T> {
        Self {  axis_x: Vector4 {x: v.x, y: T::zero(), z: T::zero(), w: T::zero()},
                axis_y: Vector4 {x: T::zero(), y: v.y, z: T::zero(), w: T::zero()},
                axis_z: Vector4 {x: T::zero(), y: T::zero(), z: v.z, w: T::zero()},
                axis_w: Vector4 {x: T::zero(), y: T::zero(), z: T::zero(), w: T::one()}  }
    }
    #[inline]
    pub fn from_scale(x: T, y:T, z:T) -> Matrix4<T> {
        Self {  axis_x: Vector4 {x: x, y: T::zero(), z: T::zero(), w: T::zero()},
                axis_y: Vector4 {x: T::zero(), y: y, z: T::zero(), w: T::zero()},
                axis_z: Vector4 {x: T::zero(), y: T::zero(), z: z, w: T::zero()},
                axis_w: Vector4 {x: T::zero(), y: T::zero(), z: T::zero(), w: T::one()}  }
    }
}

