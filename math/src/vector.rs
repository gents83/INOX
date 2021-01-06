#![allow(dead_code)]
#![allow(unused_must_use)]

use crate::implement_zero_as_default;
use super::cast_to::*;
use super::number::*;
use super::float::*;
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
    ($VectorN:ident { $($field:ident),+ }, $n:expr) => {  
        impl<T> $VectorN<T> 
        where T: Number {  
            #[inline]
            pub fn new($($field: T),+) -> $VectorN<T> {
                $VectorN { $($field: $field),+ }
            }
            
            #[inline]
            pub fn squared_length(&self) -> T {
                let array: [T; $n] = self.into();
                array.iter().fold(T::zero(), |acc, el| acc + *el * *el)
            }
            
            #[inline]
            pub fn length(&self) -> T 
            where T: Float {
                Float::sqrt(self.squared_length())
            }            
            
            #[inline]
            pub fn sqrt(&mut self) -> Self
            where T: Float {
                let vec = $VectorN { $($field: (self.$field.sqrt())),+ };
                *self = vec;
                *self
            }    
            
            #[inline]
            pub fn trunc(&mut self) -> Self
            where T: Float {
                let vec = $VectorN { $($field: (self.$field.trunc())),+ };
                *self = vec;
                *self
            }
            
            #[inline]
            pub fn normalize(&mut self) -> Self 
            where T: Float {
                self.normalize_to(T::one());
                *self
            }

            #[inline]
            pub fn normalize_to(&mut self, magnitude: T) -> Self 
            where T: Float {
                *self = self.clone() * (magnitude / self.length());
                *self
            }
            
            #[inline]
            pub fn get_normalized(&self) -> $VectorN<T> 
            where T: Float {
                self.get_normalized_to(T::one())
            }

            #[inline]
            pub fn get_normalized_to(&self, magnitude: T) -> $VectorN<T> 
            where T: Float {
                $VectorN { $($field: (self.$field * (magnitude / self.length()))),+ }
            }
            
            #[inline]
            pub fn angle(&self, other: $VectorN<T>) -> T 
            where T: Float {
                Float::acos(self.dot(other) / (self.length() * other.length()))
            }

            #[inline]
            pub fn dot(&self, other: $VectorN<T>) -> T {
                (*self * other).sum_fields()
            }

            #[inline]
            pub fn sum_fields(&self) -> T {
                let array: [T; $n] = self.into();
                array.iter().fold(T::zero(), |acc, el| acc + *el)
            }

            #[inline]
            pub fn mul_fields(&self) -> T {
                let array: [T; $n] = self.into();
                array.iter().fold(T::zero(), |acc, el| acc * *el)
            }
            
            #[inline]
            pub fn sub_fields(&self) -> T {
                let array: [T; $n] = self.into();
                array.iter().fold(T::zero(), |acc, el| acc - *el)
            }
            
            #[inline]
            pub fn div_fields(&self) -> T {
                let array: [T; $n] = self.into();
                array.iter().fold(T::zero(), |acc, el| acc / *el)
            }

            #[inline]
            fn for_each_fields<F>(&mut self, mut f: F)
            where F: FnMut(&mut T) {
                let mut array: [T; $n] = self.into();
                array.iter_mut().for_each(|el| f(el));
                *self = array.into();
            }
        }

        impl<T> Default for $VectorN<T> 
        where T: Number {  
            fn default() -> $VectorN<T>
            {
                $VectorN { $($field : T::zero()),+ }
            }
        }

        impl<T> AsRef<[T; $n]> for $VectorN<T> 
        where T: Number {  
            #[inline]
            fn as_ref(&self) -> &[T; $n] {
                unsafe { ::std::mem::transmute(self) }
            }
        }

        impl<T> AsMut<[T; $n]> for $VectorN<T> 
        where T: Number {  
            #[inline]
            fn as_mut(&mut self) -> &mut [T; $n] {
                unsafe { ::std::mem::transmute(self) }
            }
        }
        
        impl<T> From<[T; $n]> for $VectorN<T> 
        where T: Number {  
            #[inline]
            fn from(v: [T; $n]) -> $VectorN<T> {
                match v { [$($field),+] => $VectorN { $($field),+ } }
            }
        }

        impl<T> From<$VectorN<T>> for [T; $n] 
        where T: Number {  
            #[inline]
            fn from(v: $VectorN<T>) -> [T; $n] {
                match v { $VectorN { $($field),+ } => [$($field),+] }
            }
        }

        impl<T> From<& $VectorN<T>> for [T; $n] 
        where T: Number {  
            #[inline]
            fn from(v: & $VectorN<T>) -> [T; $n] {
                match *v { $VectorN { $($field),+ } => [$($field),+] }
            }
        }

        impl<T> From<&mut $VectorN<T>> for [T; $n] 
        where T: Number {  
            #[inline]
            fn from(v: &mut $VectorN<T>) -> [T; $n] {
                match *v { $VectorN { $($field),+ } => [$($field),+] }
            }
        }

        impl<T> From<T> for $VectorN<T> 
        where T: Number {  
            #[inline]
            fn from(v: T) -> $VectorN<T> {
                 $VectorN { $($field: v),+ }
            }
        }

        impl<'a, T> From<&[T]> for &'a $VectorN<T> 
        where T: Number, [T]: std::marker::Sized {  
            #[inline]
            fn from(v: &[T]) -> &'a $VectorN<T> {
                unsafe { ::std::mem::transmute(v) }
            }
        }

        impl<'a, T> From<&[T;$n]> for &'a $VectorN<T> 
        where T: Number, [T]: std::marker::Sized {  
            #[inline]
            fn from(v: &[T;$n]) -> &'a $VectorN<T> {
                unsafe { ::std::mem::transmute(v) }
            }
        }
        
        impl<'a, T> From<&mut [T;$n]> for &'a mut $VectorN<T> 
        where T: Number, [T]: std::marker::Sized {  
            #[inline]
            fn from(v: &mut [T;$n]) -> &'a mut $VectorN<T> {
                unsafe { ::std::mem::transmute(v) }
            }
        }

        impl<'a, T> Into<&'a $VectorN<T>> for &[T] 
        where [T]: std::marker::Sized {
            #[inline]
            fn into(self) -> &'a $VectorN<T> {
                unsafe { ::std::mem::transmute(self) }
            }
        }

        impl<T> ::std::ops::Add<$VectorN<T>> for $VectorN<T> 
        where T: Number {  
            type Output = $VectorN<T>;
            #[inline]
            fn add(self, other: $VectorN<T>) -> $VectorN<T> {
                $VectorN { $($field: self.$field + other.$field),+ }
            }
        }
        impl<T> ::std::ops::AddAssign<$VectorN<T>> for $VectorN<T> 
        where T: Number {  
            #[inline]
            fn add_assign(&mut self, other: $VectorN<T>) {
                let vec = $VectorN { $($field: self.$field + other.$field),+ };
                *self = vec;
            }
        }
        impl<T> ::std::ops::AddAssign<T> for $VectorN<T> 
        where T: Number {  
            #[inline]
            fn add_assign(&mut self, other: T) {
                let vec = $VectorN { $($field: self.$field + other),+ };
                *self = vec;
            }
        }
        impl<T> ::std::ops::Sub<$VectorN<T>> for $VectorN<T> 
        where T: Number {  
            type Output = $VectorN<T>;
            #[inline]
            fn sub(self, other: $VectorN<T>) -> $VectorN<T> {
                $VectorN { $($field: self.$field - other.$field),+ }
            }
        }
        impl<T> ::std::ops::SubAssign<$VectorN<T>> for $VectorN<T> 
        where T: Number {  
            #[inline]
            fn sub_assign(&mut self, other: $VectorN<T>) {
                let vec = $VectorN { $($field: self.$field - other.$field),+ };
                *self = vec;
            }
        }
        impl<T> ::std::ops::SubAssign<T> for $VectorN<T> 
        where T: Number {  
            #[inline]
            fn sub_assign(&mut self, other: T) {
                let vec = $VectorN { $($field: self.$field - other),+ };
                *self = vec;
            }
        }
        impl<T> ::std::ops::Mul<$VectorN<T>> for $VectorN<T> 
        where T: Number {  
            type Output = $VectorN<T>;
            #[inline]
            fn mul(self, other: $VectorN<T>) -> $VectorN<T> {
                $VectorN { $($field: self.$field * other.$field),+ }
            }
        }
        impl<T> ::std::ops::MulAssign<$VectorN<T>> for $VectorN<T> 
        where T: Number {  
            #[inline]
            fn mul_assign(&mut self, other: $VectorN<T>) {
                let vec = $VectorN { $($field: self.$field * other.$field),+ };
                *self = vec
            }
        }
        impl<T> ::std::ops::MulAssign<T> for $VectorN<T> 
        where T: Number {  
            #[inline]
            fn mul_assign(&mut self, other: T) {
                let vec = $VectorN { $($field: self.$field * other),+ };
                *self = vec
            }
        }
        impl<T> ::std::ops::Div<$VectorN<T>> for $VectorN<T> 
        where T: Number {  
            type Output = $VectorN<T>;
            #[inline]
            fn div(self, other: $VectorN<T>) -> $VectorN<T> {
                $VectorN { $($field: self.$field / other.$field),+ }
            }
        }
        impl<T> ::std::ops::DivAssign<$VectorN<T>> for $VectorN<T> 
        where T: Number {  
            #[inline]
            fn div_assign(&mut self, other: $VectorN<T>) {
                let vec = $VectorN { $($field: self.$field / other.$field),+ };
                *self = vec;
            }
        }
        impl<T> ::std::ops::DivAssign<T> for $VectorN<T> 
        where T: Number {  
            #[inline]
            fn div_assign(&mut self, other: T) {
                let vec = $VectorN { $($field: self.$field / other),+ };
                *self = vec;
            }
        }
        impl<T: ::std::ops::Neg<Output = T>> ::std::ops::Neg for $VectorN<T> {
            type Output = $VectorN<T>;
            #[inline]
            fn neg(self) -> $VectorN<T> {
                $VectorN { $($field: -self.$field),+ }
            }
        }  
        
        impl<T> ::std::ops::Mul<T> for $VectorN<T> 
        where T: Number {  
            type Output = $VectorN<T>;
            #[inline]
            fn mul(self, other: T) -> $VectorN<T> {
                $VectorN { $($field: self.$field * other),+ }
            }
        }
        impl<T> ::std::ops::Div<T> for $VectorN<T> 
        where T: Number {  
            type Output = $VectorN<T>;
            #[inline]
            fn div(self, other: T) -> $VectorN<T> {
                $VectorN { $($field: self.$field / other),+ }
            }
        }

        impl<T, Idx> ::std::ops::Index<Idx> for $VectorN<T> 
        where T: Number, Idx: std::slice::SliceIndex<[T]> + std::slice::SliceIndex<[T], Output = T> {
            type Output = T;

            #[inline]
            fn index<'a>(&'a self, i: Idx) -> &'a T {
                let v: &[T; $n] = self.as_ref();
                &v[i]
            }
        }
        impl<T, Idx> ::std::ops::IndexMut<Idx> for $VectorN<T> 
        where T: Number, Idx: std::slice::SliceIndex<[T]> + std::slice::SliceIndex<[T], Output = T> {
            #[inline]
            fn index_mut<'a>(&'a mut self, i: Idx) -> &'a mut T {
                let v: &mut [T; $n] = self.as_mut();
                &mut v[i]
            }
        }       

        impl<T: NumberCast + Copy> $VectorN<T> {
            #[inline]
            pub fn cast<F: NumberCast>(&self) -> Option<$VectorN<F>> {
                $(
                    let $field = match NumberCast::from(self.$field) {
                        Some(field) => field,
                        None => return None
                    };
                )+
                Some($VectorN { $($field),+ })
            }
        }
        
        impl<T: ::std::fmt::Debug> ::std::fmt::Debug for $VectorN<T> 
        where T: Number {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result { 
                if let Some((first, elements)) = self.as_ref().split_first() {            
                    write!(f, "[{}", first );
                    for item in elements {
                        write!(f, ", {}", item);
                    }
                    writeln!(f, "]")
                }
                else {
                    writeln!(f, "")
                }
            }
        }

        implement_zero_as_default!($VectorN);
    }
}

implement_vector!(Vector1 { x }, 1);
implement_vector!(Vector2 { x, y }, 2);
implement_vector!(Vector3 { x, y, z }, 3);
implement_vector!(Vector4 { x, y, z, w }, 4);

pub type Vector1f = Vector1<f32>;
pub type Vector2f = Vector2<f32>;
pub type Vector3f = Vector3<f32>;
pub type Vector4f = Vector4<f32>;

pub type Vector1u = Vector1<u32>;
pub type Vector2u = Vector2<u32>;
pub type Vector3u = Vector3<u32>;
pub type Vector4u = Vector4<u32>;


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
    assert_eq!(vec3[0], 1.0);
    assert_eq!(vec3[1], 2.0);
    assert_eq!(vec3[2], 3.0);

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