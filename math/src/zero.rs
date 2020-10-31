
pub trait Zero: Sized + ::std::ops::Add<Self, Output = Self> {
    
    fn zero() -> Self;

    fn set_zero(&mut self) {
        *self = Zero::zero();
    }

    fn is_zero(&self) -> bool;
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
    ($Type:ty) => {
        impl Zero for $Type {
            #[inline]
            fn zero() -> $Type {
                type Type = $Type;
                Type::default()
            }
            #[inline]
            fn is_zero(&self) -> bool {
                type Type = $Type;
                *self == Type::default()
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

