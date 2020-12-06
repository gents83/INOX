use core::num::Wrapping;

pub trait Boundaries {
    fn min_value() -> Self;
    fn max_value() -> Self;
}

macro_rules! implement_boundaries {
    ($t:ty, $min:expr, $max:expr) => {
        impl Boundaries for $t {
            #[inline]
            fn min_value() -> $t {
                $min
            }

            #[inline]
            fn max_value() -> $t {
                $max
            }
        }
    };
}

implement_boundaries!(usize, usize::MIN, usize::MAX);
implement_boundaries!(u8, u8::MIN, u8::MAX);
implement_boundaries!(u16, u16::MIN, u16::MAX);
implement_boundaries!(u32, u32::MIN, u32::MAX);
implement_boundaries!(u64, u64::MIN, u64::MAX);

implement_boundaries!(isize, isize::MIN, isize::MAX);
implement_boundaries!(i8, i8::MIN, i8::MAX);
implement_boundaries!(i16, i16::MIN, i16::MAX);
implement_boundaries!(i32, i32::MIN, i32::MAX);
implement_boundaries!(i64, i64::MIN, i64::MAX);
implement_boundaries!(f32, f32::MIN, f32::MAX);
implement_boundaries!(f64, f64::MIN, f64::MAX);


impl<T: Boundaries> Boundaries for Wrapping<T> {
    fn min_value() -> Self {
        Wrapping(T::min_value())
    }
    fn max_value() -> Self {
        Wrapping(T::max_value())
    }
}
