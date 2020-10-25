use super::vector::Vector3;
use super::vector::Vector4;
use super::zero::*;

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
            
            pub fn print(&self) {
                $(self.$field.print()); +
            }
            
            pub fn identity() -> Self {
                type Vector = $VecType;
                let mut vec_array : [$VecType; $n] = [Vector::zero(); $n];
                for i in 0..$n {
                    vec_array[i].as_mut()[i] = 1.0;
                }
                $MatrixN::from_axis( &vec_array )
            }
        }
    }
}

implement_matrix!(Matrix3 { axis_x, axis_y, axis_z }, 3, Vector3<f32>, f32);
implement_matrix!(Matrix4 { axis_x, axis_y, axis_z, axis_w }, 4, Vector4<f32>, f32);

/*
impl Matrix3<f32> {
    pub fn identity() -> Self {
        Matrix3::new(Vector3::new(1.0, 0.0, 0.0),
                     Vector3::new(0.0, 1.0, 0.0),
                     Vector3::new(0.0, 0.0, 1.0))
    }
}

impl Matrix4<f32> {
    pub fn identity() -> Self {
        Matrix4::new(Vector4::new(1.0, 0.0, 0.0, 0.0),
                     Vector4::new(0.0, 1.0, 0.0, 0.0),
                     Vector4::new(0.0, 0.0, 1.0, 0.0),
                     Vector4::new(0.0, 0.0, 0.0, 1.0))
    }
}

*/