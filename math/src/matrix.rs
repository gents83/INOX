#![allow(dead_code)]
#![allow(unused_must_use)]

use super::vector::*;
use super::zero::*;
use super::one::*;

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
    ($MatrixN:ident { $($field:ident),+ }, $n:expr, $VecType:ty, $Type:ty) => {

        impl $MatrixN<$Type> {            
            #[inline]
            pub const fn new($($field: $VecType),+) -> $MatrixN<$Type> {
                $MatrixN { $($field: $field),+ }
            }
            
            pub const fn from_axis( &v: &[$VecType; $n] ) -> $MatrixN<$Type> {
                match v { [$($field),+] => $MatrixN { $($field),+ } }
            }
            
            pub fn identity() -> Self {
                type Vector = $VecType;
                type BaseType = $Type;
                let mut vec_array : [$VecType; $n] = [Vector::zero(); $n];
                for i in 0..$n {
                    vec_array[i].as_mut()[i] = BaseType::one();
                }
                $MatrixN::from_axis( &vec_array )
            }
        }
        
                
        impl ::std::fmt::Debug for $MatrixN<$Type> {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {                 
                $(self.$field.fmt(f)); +
            }
        }
    }
}

implement_matrix!(Matrix3 { axis_x, axis_y, axis_z }, 3, Vector3<f32>, f32);
implement_matrix!(Matrix4 { axis_x, axis_y, axis_z, axis_w }, 4, Vector4<f32>, f32);

/*
impl<T: std::ops::Mul<Output = T> + std::ops::Sub<Output = T> + std::ops::Deref> Matrix3<T> 
where T: num::Float {
    pub fn create_from_translation(self, v: Vector2<T>) -> Matrix3<T> {
        Self {  axis_x: Vector3 {x: 1.0, y: 0.0, z: 0.0},
                axis_y: [0.0, 1.0, 0.0].into(),
                axis_z: [v.x, v.y, 1.0].into() }
    }
}
*/