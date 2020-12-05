#![allow(dead_code)]

use crate::implement_zero_as_default;
use super::zero::*;

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
pub struct Vector1<T> {
    pub x: T,
}

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
pub struct Vector2<T> {
    pub x: T,
    pub y: T,
}

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
pub struct Vector3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
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

            #[inline]
            pub fn map<F>(self, mut f: F) -> $VectorType
                where F: FnMut($Type) -> $Type
            {
                $VectorN { $($field: f(self.$field)),+ }
            }

            #[inline]
            pub fn zip<T2, T3, F>(self, v2: $VectorN<T2>, mut f: F) -> $VectorN<T3>
                where F: FnMut($Type, T2) -> T3
            {
                $VectorN { $($field: f(self.$field, v2.$field)),+ }
            }

            #[inline]
            fn add(&mut self, other: $Type) -> Self {
                self.for_each_fields(|el| *el += other);
                *self
            }
            #[inline]
            fn sub(&mut self, other: $Type) -> Self {
                self.for_each_fields(|el| *el -= other);
                *self
            }
            #[inline]
            fn mul(&mut self, other: $Type) -> Self {
                self.for_each_fields(|el| *el *= other);
                *self
            }
            #[inline]
            fn div(&mut self, other: $Type) -> Self {
                self.for_each_fields(|el| *el /= other);
                *self
            }

            
            #[inline]
            pub fn squared_length(&self) -> $Type {
                type ParamType = $Type;
                let array: [ParamType; $n] = self.into();
                array.iter().fold(ParamType::zero(), |acc, el| acc + el*el)
            }
            
            #[inline]
            pub fn length(&self) -> $Type {
                (self.squared_length() as f64).sqrt() as $Type
            }
            
            #[inline]
            fn normalize(&mut self) -> Self {
                self.normalize_to(1.0 as $Type);
                *self
            }

            #[inline]
            fn normalize_to(&mut self, magnitude: $Type) -> Self {
                self.mul(magnitude / self.length());
                *self
            }
            
            #[inline]
            fn get_normalized(&self) -> $VectorType {
                self.get_normalized_to(1.0 as $Type)
            }

            #[inline]
            fn get_normalized_to(&self, magnitude: $Type) -> $VectorType {
                $VectorN { $($field: (self.$field * (magnitude / self.length()))),+ }
            }
            
            #[inline]
            pub fn angle(&self, other: $VectorType) -> $Type {
                f64::acos(self.dot(other) as f64 / (self.length() * other.length())  as f64) as _
            }

            #[inline]
            pub fn dot(&self, other: $VectorType) -> $Type {
                (*self * other).sum_fields()
            }

            #[inline]
            pub fn sum_fields(&self) -> $Type {
                type ParamType = $Type;
                let array: [ParamType; $n] = self.into();
                array.iter().fold(ParamType::zero(), |acc, el| acc + el)
            }

            #[inline]
            pub fn mul_fields(&self) -> $Type {
                type ParamType = $Type;
                let array: [ParamType; $n] = self.into();
                array.iter().fold(ParamType::zero(), |acc, el| acc * el)
            }
            
            #[inline]
            pub fn sub_fields(&self) -> $Type {
                type ParamType = $Type;
                let array: [ParamType; $n] = self.into();
                array.iter().fold(ParamType::zero(), |acc, el| acc - el)
            }
            
            #[inline]
            pub fn div_fields(&self) -> $Type {
                type ParamType = $Type;
                let array: [ParamType; $n] = self.into();
                array.iter().fold(ParamType::zero(), |acc, el| acc / el)
            }

            #[inline]
            fn for_each_fields<F>(&mut self, mut f: F)
            where F: FnMut(&mut $Type) {
                type ParamType = $Type;
                let mut array: [ParamType; $n] = self.into();
                array.iter_mut().for_each(|el| f(el));
                *self = array.into();
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

        impl From<&[$Type; $n]> for $VectorType {
            #[inline]
            fn from(v: &[$Type; $n]) -> $VectorType {
                match *v { [$($field),+] => $VectorN { $($field),+ } }
            }
        }

        impl From<& $VectorType> for [$Type; $n] {
            #[inline]
            fn from(v: & $VectorType) -> [$Type; $n] {
                match *v { $VectorN { $($field),+ } => [$($field),+] }
            }
        }

        impl From<&mut $VectorType> for [$Type; $n] {
            #[inline]
            fn from(v: &mut $VectorType) -> [$Type; $n] {
                match *v { $VectorN { $($field),+ } => [$($field),+] }
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
        impl ::std::ops::Sub for $VectorType {
            type Output = $VectorType;
            #[inline]
            fn sub(self, other: $VectorType) -> $VectorType {
                $VectorN { $($field: self.$field - other.$field),+ }
            }
        }
        impl ::std::ops::Mul for $VectorType {
            type Output = $VectorType;
            #[inline]
            fn mul(self, other: $VectorType) -> $VectorType {
                $VectorN { $($field: self.$field * other.$field),+ }
            }
        }
        impl ::std::ops::Div for $VectorType {
            type Output = $VectorType;
            #[inline]
            fn div(self, other: $VectorType) -> $VectorType {
                $VectorN { $($field: self.$field / other.$field),+ }
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

impl<T: std::ops::Mul<Output = T> + std::ops::Sub<Output = T> + std::ops::Deref> Vector2<T> {
    pub fn perpendicular_dot(self, other: Vector2<T>) -> T {
        (self.x * other.y) - (self.y * other.x)
    }
}

impl<T: std::ops::Mul<Output = T> + std::ops::Sub<Output = T> + std::clone::Clone> Vector3<T> {
    pub fn cross(self, other: Vector3<T>) -> Self { 
        Self{
            x: (self.y.clone() * other.z.clone()) - (self.z.clone() * other.y.clone()),
            y: (self.z.clone() * other.x.clone()) - (self.x.clone() * other.z.clone()),
            z: (self.x.clone() * other.y.clone()) - (self.y.clone() * other.x.clone()),
        }
    }
}


#[test]
fn test_vector()
{ 
    let mut vec2 = Vector2f::zero();
    assert_eq!(vec2, Vector2f::zero());

    let mut vec3 = Vector3f::zero();
    assert_eq!(vec3, Vector3f::zero());

    vec3.add(1.0);
    assert_eq!(vec3, Vector3f::new(1.0, 1.0, 1.0));
    
    vec3.mul(4.0);
    assert_eq!(vec3, Vector3f::new(4.0, 4.0, 4.0));
    
    vec3.div(2.0);
    assert_eq!(vec3, Vector3f::new(2.0, 2.0, 2.0));
    
    vec3.sub(2.0);
    assert_eq!(vec3, Vector3f::zero());

    vec3 = vec3 + Vector3f::new(1.0, 2.0, 3.0);
    assert_eq!(vec3, Vector3f::new(1.0, 2.0, 3.0));

    vec3 = Vector3f::new(3.0, 4.0, 0.0);
    assert_eq!(vec3.squared_length(), 25.0);
    assert_eq!(vec3.length(), 5.0);

    assert_eq!(vec3.get_normalized().length(), 1.0);
    assert_eq!(vec3.length(), 5.0);

    vec3.normalize_to(1.0);
    assert_ne!(vec3.length(), 5.0);
    assert_eq!(vec3.length(), 1.0);

    let dot = Vector3f::new(1.0, 0.0, 0.0).dot(Vector3f::new(1.0, 0.0, 0.0));    
    assert_eq!(dot, 1.0);
    let dot = Vector3f::new(0.0, 1.0, 0.0).dot(Vector3f::new(1.0, 0.0, 0.0));    
    assert_eq!(dot, 0.0);
    let dot = Vector3f::new(1.0, 0.0, 0.0).dot(Vector3f::new(-1.0, 0.0, 0.0));    
    assert_eq!(dot, -1.0);

    let cross = Vector3f::new(1.0, 0.0, 0.0).cross(Vector3f::new(0.0, 1.0, 0.0));   
    assert_eq!(cross, Vector3f::new(0.0, 0.0, 1.0));

}