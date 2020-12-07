
use crate::implement_zero_as_default;
use super::float::*;
use super::number::*;
use super::zero::*;


#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub struct Radians<T>(pub T);
#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub struct Degree<T>(pub T);

impl<T> From<Radians<T>> for Degree<T> 
where T: Float {
    fn from(angle: Radians<T>) -> Degree<T> {
        Degree::new(angle.0.to_degrees())
    }
}
impl<T> From<Degree<T>> for Radians<T>
where T: Float {
    fn from(angle: Degree<T>) -> Radians<T> {
        Radians::new(angle.0.to_radians())
    }
}

impl<T> Default for Degree<T> 
where T: Number {  
    fn default() -> Degree<T>{
        Degree(T::zero())
    }
}

impl<T> Default for Radians<T> 
where T: Number {  
    fn default() -> Radians<T>{
        Radians(T::zero())
    }
}

impl<T> Degree<T> 
where T:Float {
    pub fn new(angle_in_degrees: T) -> Degree<T> {
        Degree(angle_in_degrees)
    }
    pub fn pi() -> Degree<T> {
        Degree(T::from(core::f64::consts::PI.to_degrees()).unwrap())
    }
    pub fn half_pi() -> Degree<T> {
        Degree(T::from(core::f64::consts::PI.to_degrees() * 0.5).unwrap())
    }
}

impl<T> Radians<T> 
where T:Float {
    pub fn new(angle_in_radians: T) -> Radians<T> {
        Radians(angle_in_radians)
    }
    pub fn pi() -> Radians<T> {
        Radians(T::from(core::f64::consts::PI).unwrap() )
    }
    pub fn half_pi() -> Radians<T> {
        Radians(T::from(core::f64::consts::PI * 0.5).unwrap())
    }
}



impl<T> ::std::ops::Add<Degree<T> > for Degree<T> 
where T: Number {  
    type Output = Degree<T>;
    #[inline]
    fn add(self, other: Degree<T>) -> Degree<T> {
        Degree(self.0 + other.0)
    }
}
impl<T> ::std::ops::Sub<Degree<T> > for Degree<T> 
where T: Number {  
    type Output = Degree<T>;
    #[inline]
    fn sub(self, other: Degree<T>) -> Degree<T> {
        Degree(self.0 - other.0)
    }
}
impl<T> ::std::ops::Mul<Degree<T> > for Degree<T> 
where T: Number {  
    type Output = Degree<T>;
    #[inline]
    fn mul(self, other: Degree<T>) -> Degree<T> {
        Degree(self.0 * other.0)
    }
}
impl<T> ::std::ops::Div<Degree<T> > for Degree<T> 
where T: Number {  
    type Output = Degree<T>;
    #[inline]
    fn div(self, other: Degree<T>) -> Degree<T> {
        Degree(self.0 / other.0)
    }
}
impl<T> ::std::ops::Mul<T> for Degree<T> 
where T: Number {  
    type Output = Degree<T>;
    #[inline]
    fn mul(self, other: T) -> Degree<T> {
        Degree(self.0 * other)
    }
}
impl<T> ::std::ops::Div<T> for Degree<T> 
where T: Number {  
    type Output = Degree<T>;
    #[inline]
    fn div(self, other: T) -> Degree<T> {
        Degree(self.0 / other)
    }
}


impl<T> ::std::ops::Add<Radians<T>> for Radians<T> 
where T: Number {  
    type Output = Radians<T>;
    #[inline]
    fn add(self, other: Radians<T>) -> Radians<T> {
        Radians(self.0 + other.0)
    }
}
impl<T> ::std::ops::Sub<Radians<T>> for Radians<T> 
where T: Number {  
    type Output = Radians<T>;
    #[inline]
    fn sub(self, other: Radians<T>) -> Radians<T> {
        Radians(self.0 - other.0)
    }
}
impl<T> ::std::ops::Mul<Radians<T>> for Radians<T> 
where T: Number {  
    type Output = Radians<T>;
    #[inline]
    fn mul(self, other: Radians<T>) -> Radians<T> {
        Radians(self.0 * other.0)
    }
}
impl<T> ::std::ops::Div<Radians<T>> for Radians<T> 
where T: Number {  
    type Output = Radians<T>;
    #[inline]
    fn div(self, other: Radians<T>) -> Radians<T> {
        Radians(self.0 / other.0)
    }
}
impl<T> ::std::ops::Mul<T> for Radians<T> 
where T: Number {  
    type Output = Radians<T>;
    #[inline]
    fn mul(self, other: T) -> Radians<T> {
        Radians(self.0 * other)
    }
}
impl<T> ::std::ops::Div<T> for Radians<T> 
where T: Number {  
    type Output = Radians<T>;
    #[inline]
    fn div(self, other: T) -> Radians<T> {
        Radians(self.0 / other)
    }
}

implement_zero_as_default!(Degree);
implement_zero_as_default!(Radians);