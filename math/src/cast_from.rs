
use core::num::Wrapping;
use super::cast_to::*;

pub trait CastFromType: Sized {
    #[inline]
    fn from_isize(n: isize) -> Option<Self> {
        n.to_i64().and_then(CastFromType::from_i64)
    }
    #[inline]
    fn from_i8(n: i8) -> Option<Self> {
        CastFromType::from_i64(From::from(n))
    }
    #[inline]
    fn from_i16(n: i16) -> Option<Self> {
        CastFromType::from_i64(From::from(n))
    }
    #[inline]
    fn from_i32(n: i32) -> Option<Self> {
        CastFromType::from_i64(From::from(n))
    }
    fn from_i64(n: i64) -> Option<Self>;
    #[inline]
    fn from_usize(n: usize) -> Option<Self> {
        n.to_u64().and_then(CastFromType::from_u64)
    }
    #[inline]
    fn from_u8(n: u8) -> Option<Self> {
        CastFromType::from_u64(From::from(n))
    }
    #[inline]
    fn from_u16(n: u16) -> Option<Self> {
        CastFromType::from_u64(From::from(n))
    }
    #[inline]
    fn from_u32(n: u32) -> Option<Self> {
        CastFromType::from_u64(From::from(n))
    }
    fn from_u64(n: u64) -> Option<Self>;
    #[inline]
    fn from_f32(n: f32) -> Option<Self> {
        CastFromType::from_f64(From::from(n))
    }
    #[inline]
    fn from_f64(n: f64) -> Option<Self> {
        match n.to_i64() {
            Some(i) => CastFromType::from_i64(i),
            None => n.to_u64().and_then(CastFromType::from_u64),
        }
    }
}

macro_rules! implement_cast_from {
    ($T:ty, $to_ty:ident) => {
        #[allow(deprecated)]
        impl CastFromType for $T {
            #[inline]
            fn from_isize(n: isize) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_i8(n: i8) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_i16(n: i16) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_i32(n: i32) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_i64(n: i64) -> Option<$T> {
                n.$to_ty()
            }
            #[cfg(has_i128)]
            #[inline]
            fn from_i128(n: i128) -> Option<$T> {
                n.$to_ty()
            }

            #[inline]
            fn from_usize(n: usize) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_u8(n: u8) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_u16(n: u16) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_u32(n: u32) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_u64(n: u64) -> Option<$T> {
                n.$to_ty()
            }
            #[cfg(has_i128)]
            #[inline]
            fn from_u128(n: u128) -> Option<$T> {
                n.$to_ty()
            }

            #[inline]
            fn from_f32(n: f32) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_f64(n: f64) -> Option<$T> {
                n.$to_ty()
            }
        }
    };
}

implement_cast_from!(isize, to_isize);
implement_cast_from!(i8, to_i8);
implement_cast_from!(i16, to_i16);
implement_cast_from!(i32, to_i32);
implement_cast_from!(i64, to_i64);
implement_cast_from!(usize, to_usize);
implement_cast_from!(u8, to_u8);
implement_cast_from!(u16, to_u16);
implement_cast_from!(u32, to_u32);
implement_cast_from!(u64, to_u64);
implement_cast_from!(f32, to_f32);
implement_cast_from!(f64, to_f64);

macro_rules! implement_cast_from_with_wrapping {
    ($( $(#[$cfg:meta])* fn $method:ident ( $i:ident ); )*) => {$(
        #[inline]
        $(#[$cfg])*
        fn $method(n: $i) -> Option<Self> {
            T::$method(n).map(Wrapping)
        }
    )*}
}

impl<T: CastFromType> CastFromType for Wrapping<T> {
    implement_cast_from_with_wrapping! {
        fn from_isize(isize);
        fn from_i8(i8);
        fn from_i16(i16);
        fn from_i32(i32);
        fn from_i64(i64);

        fn from_usize(usize);
        fn from_u8(u8);
        fn from_u16(u16);
        fn from_u32(u32);
        fn from_u64(u64);

        fn from_f32(f32);
        fn from_f64(f64);
    }
}


pub trait AsNumber<T>: 'static + Copy
where
    T: 'static + Copy,
{
    /// Convert a value to another, using the `as` operator.
    fn as_number(self) -> T;
}

macro_rules! implement_as_number {
    (@ $T: ty => $(#[$cfg:meta])* impl $U: ty ) => {
        $(#[$cfg])*
        impl AsNumber<$U> for $T {
            #[inline] fn as_number(self) -> $U { self as $U }
        }
    };
    (@ $T: ty => { $( $U: ty ),* } ) => {$(
        implement_as_number!(@ $T => impl $U);
    )*};
    ($T: ty => { $( $U: ty ),* } ) => {
        implement_as_number!(@ $T => { $( $U ),* });
        implement_as_number!(@ $T => { u8, u16, u32, u64, usize });
        implement_as_number!(@ $T => { i8, i16, i32, i64, isize });
    };
}

implement_as_number!(u8 => { char, f32, f64 });
implement_as_number!(i8 => { f32, f64 });
implement_as_number!(u16 => { f32, f64 });
implement_as_number!(i16 => { f32, f64 });
implement_as_number!(u32 => { f32, f64 });
implement_as_number!(i32 => { f32, f64 });
implement_as_number!(u64 => { f32, f64 });
implement_as_number!(i64 => { f32, f64 });
implement_as_number!(usize => { f32, f64 });
implement_as_number!(isize => { f32, f64 });
implement_as_number!(f32 => { f32, f64 });
implement_as_number!(f64 => { f32, f64 });
implement_as_number!(char => { char });
implement_as_number!(bool => {});