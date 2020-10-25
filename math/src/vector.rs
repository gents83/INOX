use super::implement_zero;
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
    ($VectorN:ident { $($field:ident),+ }, $n:expr, $Type:ty) => {

        impl $VectorN<$Type> {            
            #[inline]
            pub const fn new($($field: $Type),+) -> $VectorN<$Type> {
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

        impl AsRef<[$Type; $n]> for $VectorN<$Type> {
            #[inline]
            fn as_ref(&self) -> &[$Type; $n] {
                unsafe { ::std::mem::transmute(self) }
            }
        }

        impl AsMut<[$Type; $n]> for $VectorN<$Type> {
            #[inline]
            fn as_mut(&mut self) -> &mut [$Type; $n] {
                unsafe { ::std::mem::transmute(self) }
            }
        }
        
        impl Into<[$Type; $n]> for $VectorN<$Type> {
            #[inline]
            fn into(self) -> [$Type; $n] {
                match self { $VectorN { $($field),+ } => [$($field),+] }
            }
        }

        impl From<[$Type; $n]> for $VectorN<$Type> {
            #[inline]
            fn from(v: [$Type; $n]) -> $VectorN<$Type> {
                match v { [$($field),+] => $VectorN { $($field),+ } }
            }
        }

        impl ::std::ops::Add for $VectorN<$Type> {
            type Output = $VectorN<$Type>;
            #[inline]
            fn add(self, other: $VectorN<$Type>) -> $VectorN<$Type> { self + other }
        }
        
        implement_zero!($VectorN<$Type>, $VectorN { $($field : 0.0),+ });
    }
}

implement_vector!(Vector1 { x }, 1, f32);
implement_vector!(Vector2 { x, y }, 2, f32);
implement_vector!(Vector3 { x, y, z }, 3, f32);
implement_vector!(Vector4 { x, y, z, w }, 4, f32);