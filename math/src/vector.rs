use crate::implement_zero_as_default;
use super::zero::*;

#[derive(PartialEq, Eq, Copy, Clone, Hash)]
pub struct Vector1<T> {
    pub x: T,
}

#[derive(PartialEq, Eq, Copy, Clone, Hash)]
pub struct Vector2<T> {
    pub x: T,
    pub y: T,
}

#[derive(PartialEq, Eq, Copy, Clone, Hash)]
pub struct Vector3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

#[derive(PartialEq, Eq, Copy, Clone, Hash)]
pub struct Vector4<T> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}

macro_rules! implement_vector {
    ($VectorType:ident, $Type:ty, $VectorN:ident { $($field:ident),+ }, $n:expr) => {
    
        pub type $VectorType = $VectorN<$Type>;

        impl $VectorType {  
            #[inline]
            pub const fn new($($field: $Type),+) -> $VectorType {
                $VectorN { $($field),+ }
            }

            pub fn print(&self) {    
                if let Some((first, elements)) = self.as_ref().split_first() {            
                    print!("[{}", first);
                    for item in elements {
                        print!(", {}", item);
                    }
                    println!("]");
                }
            }
        }

        impl Default for $VectorType {
            fn default() -> $VectorType
            {
                type ParamType = $Type;
                $VectorN { $($field : ParamType::zero()),+ }
            }
        }

        impl AsRef<[$Type; $n]> for $VectorType {
            #[inline]
            fn as_ref(&self) -> &[$Type; $n] {
                unsafe { ::std::mem::transmute(self) }
            }
        }

        impl AsMut<[$Type; $n]> for $VectorType {
            #[inline]
            fn as_mut(&mut self) -> &mut [$Type; $n] {
                unsafe { ::std::mem::transmute(self) }
            }
        }
        
        impl Into<[$Type; $n]> for $VectorType {
            #[inline]
            fn into(self) -> [$Type; $n] {
                match self { $VectorN { $($field),+ } => [$($field),+] }
            }
        }

        impl From<[$Type; $n]> for $VectorType {
            #[inline]
            fn from(v: [$Type; $n]) -> $VectorType {
                match v { [$($field),+] => $VectorN { $($field),+ } }
            }
        }

        impl From<$Type> for $VectorType {
            #[inline]
            fn from(v: $Type) -> $VectorType {
                 $VectorN { $($field: v),+ }
            }
        }

        impl ::std::ops::Add for $VectorType {
            type Output = $VectorType;
            #[inline]
            fn add(self, other: $VectorType) -> $VectorType {
                $VectorN { $($field: self.$field + other.$field),+ }
            }
        }
        
        implement_zero_as_default!($VectorType);
    }
}

implement_vector!(Vector1f, f32, Vector1 { x }, 1);
implement_vector!(Vector2f, f32, Vector2 { x, y }, 2);
implement_vector!(Vector3f, f32, Vector3 { x, y, z }, 3);
implement_vector!(Vector4f, f32, Vector4 { x, y, z, w }, 4);

implement_vector!(Vector1u, u32, Vector1 { x }, 1);
implement_vector!(Vector2u, u32, Vector2 { x, y }, 2);
implement_vector!(Vector3u, u32, Vector3 { x, y, z }, 3);
implement_vector!(Vector4u, u32, Vector4 { x, y, z, w }, 4);
