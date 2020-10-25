
pub trait One: Sized + ::std::ops::Add<Self, Output = Self> {
    
    fn one() -> Self;

    fn set_one(&mut self) {
        *self = One::one();
    }

    fn is_one(&self) -> bool;
}

#[macro_export]
macro_rules! implement_one {
    ($Type:ty, $value:expr) => {
        impl One for $Type {
            #[inline]
            fn one() -> $Type {
                $value
            }
            #[inline]
            fn is_one(&self) -> bool {
                *self == $value
            }
        }
    };
}

implement_one!(usize, 1);
implement_one!(u8, 1);
implement_one!(u16, 1);
implement_one!(u32, 1);
implement_one!(u64, 1);
#[cfg(has_i128)]
implement_one!(u128, 1);

implement_one!(isize, 1);
implement_one!(i8, 1);
implement_one!(i16, 1);
implement_one!(i32, 1);
implement_one!(i64, 1);
#[cfg(has_i128)]
implement_one!(i128, 1);

implement_one!(f32, 1.0);
implement_one!(f64, 1.0);

