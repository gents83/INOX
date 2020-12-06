
use core::ops::*;
use core::num::Wrapping;

pub trait Zero: Sized + Add<Self, Output = Self> {
    
    fn zero() -> Self;

    fn set_zero(&mut self) {
        *self = Zero::zero();
    }

    fn is_zero(&self) -> bool;
}

#[inline(always)]
pub fn zero<T: Zero>() -> T {
    Zero::zero()
}

#[macro_export]
macro_rules! implement_zero {
    ($Type:ty, $value:expr) => {
        impl Zero for $Type {
            #[inline]
            fn zero() -> $Type {
                $value
            }
            #[inline]
            fn is_zero(&self) -> bool {
                *self == $value
            }
        }
    };
}

#[macro_export]
macro_rules! implement_zero_as_default {
    ($Type:ident) => {
        impl<T> Zero for $Type<T> 
        where T: Number {  
            #[inline]
            fn zero() -> $Type<T> {
                $Type::default()
            }
            #[inline]
            fn is_zero(&self) -> bool {
                *self == $Type::default()
            }
        }
    };
}

implement_zero!(usize, 0);
implement_zero!(u8, 0);
implement_zero!(u16, 0);
implement_zero!(u32, 0);
implement_zero!(u64, 0);
#[cfg(has_i128)]
implement_zero!(u128, 0);

implement_zero!(isize, 0);
implement_zero!(i8, 0);
implement_zero!(i16, 0);
implement_zero!(i32, 0);
implement_zero!(i64, 0);
#[cfg(has_i128)]
implement_zero!(i128, 0);

implement_zero!(f32, 0.0);
implement_zero!(f64, 0.0);



impl<T: Zero> Zero for Wrapping<T>
where
    Wrapping<T>: Add<Output = Wrapping<T>>,
{
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    fn set_zero(&mut self) {
        self.0.set_zero();
    }

    fn zero() -> Self {
        Wrapping(T::zero())
    }
}
