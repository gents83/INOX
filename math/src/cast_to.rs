use core::mem::size_of;
use core::num::Wrapping;

pub trait CastToType {    
    #[inline]
    fn to_isize(&self) -> Option<isize> {
        self.to_i64().as_ref().and_then(CastToType::to_isize)
    }
    #[inline]
    fn to_i8(&self) -> Option<i8> {
        self.to_i64().as_ref().and_then(CastToType::to_i8)
    }
    #[inline]
    fn to_i16(&self) -> Option<i16> {
        self.to_i64().as_ref().and_then(CastToType::to_i16)
    }
    #[inline]
    fn to_i32(&self) -> Option<i32> {
        self.to_i64().as_ref().and_then(CastToType::to_i32)
    }
    fn to_i64(&self) -> Option<i64>;
    #[inline]
    fn to_usize(&self) -> Option<usize> {
        self.to_u64().as_ref().and_then(CastToType::to_usize)
    }
    #[inline]
    fn to_u8(&self) -> Option<u8> {
        self.to_u64().as_ref().and_then(CastToType::to_u8)
    }
    #[inline]
    fn to_u16(&self) -> Option<u16> {
        self.to_u64().as_ref().and_then(CastToType::to_u16)
    }
    #[inline]
    fn to_u32(&self) -> Option<u32> {
        self.to_u64().as_ref().and_then(CastToType::to_u32)
    }
    fn to_u64(&self) -> Option<u64>;
    #[inline]
    fn to_f32(&self) -> Option<f32> {
        self.to_f64().as_ref().and_then(CastToType::to_f32)
    }
    #[inline]
    fn to_f64(&self) -> Option<f64> {
        match self.to_i64() {
            Some(i) => i.to_f64(),
            None => self.to_u64().as_ref().and_then(CastToType::to_f64),
        }
    }
}


macro_rules! implement_cast_int_to_int {
    ($SrcT:ident : $( $(#[$cfg:meta])* fn $method:ident -> $DstT:ident ; )*) => {$(
        #[inline]
        $(#[$cfg])*
        fn $method(&self) -> Option<$DstT> {
            let min = $DstT::MIN as $SrcT;
            let max = $DstT::MAX as $SrcT;
            if size_of::<$SrcT>() <= size_of::<$DstT>() || (min <= *self && *self <= max) {
                Some(*self as $DstT)
            } else {
                None
            }
        }
    )*}
}

macro_rules! implement_cast_int_to_uint {
    ($SrcT:ident : $( $(#[$cfg:meta])* fn $method:ident -> $DstT:ident ; )*) => {$(
        #[inline]
        $(#[$cfg])*
        fn $method(&self) -> Option<$DstT> {
            let max = $DstT::MAX as $SrcT;
            if 0 <= *self && (size_of::<$SrcT>() <= size_of::<$DstT>() || *self <= max) {
                Some(*self as $DstT)
            } else {
                None
            }
        }
    )*}
}

macro_rules! implement_cast_int {
    ($T:ident) => {
        impl CastToType for $T {
            implement_cast_int_to_int! { $T:
                fn to_isize -> isize;
                fn to_i8 -> i8;
                fn to_i16 -> i16;
                fn to_i32 -> i32;
                fn to_i64 -> i64;
            }

            implement_cast_int_to_uint! { $T:
                fn to_usize -> usize;
                fn to_u8 -> u8;
                fn to_u16 -> u16;
                fn to_u32 -> u32;
                fn to_u64 -> u64;
            }

            #[inline]
            fn to_f32(&self) -> Option<f32> {
                Some(*self as f32)
            }
            #[inline]
            fn to_f64(&self) -> Option<f64> {
                Some(*self as f64)
            }
        }
    };
}

implement_cast_int!(isize);
implement_cast_int!(i8);
implement_cast_int!(i16);
implement_cast_int!(i32);
implement_cast_int!(i64);

macro_rules! implement_cast_uint_to_int {
    ($SrcT:ident : $( $(#[$cfg:meta])* fn $method:ident -> $DstT:ident ; )*) => {$(
        #[inline]
        $(#[$cfg])*
        fn $method(&self) -> Option<$DstT> {
            let max = $DstT::MAX as $SrcT;
            if size_of::<$SrcT>() < size_of::<$DstT>() || *self <= max {
                Some(*self as $DstT)
            } else {
                None
            }
        }
    )*}
}

macro_rules! implement_cast_uint_to_uint {
    ($SrcT:ident : $( $(#[$cfg:meta])* fn $method:ident -> $DstT:ident ; )*) => {$(
        #[inline]
        $(#[$cfg])*
        fn $method(&self) -> Option<$DstT> {
            let max = $DstT::MAX as $SrcT;
            if size_of::<$SrcT>() <= size_of::<$DstT>() || *self <= max {
                Some(*self as $DstT)
            } else {
                None
            }
        }
    )*}
}

macro_rules! implement_cast_uint {
    ($T:ident) => {
        impl CastToType for $T {
            implement_cast_uint_to_int! { $T:
                fn to_isize -> isize;
                fn to_i8 -> i8;
                fn to_i16 -> i16;
                fn to_i32 -> i32;
                fn to_i64 -> i64;
            }

            implement_cast_uint_to_uint! { $T:
                fn to_usize -> usize;
                fn to_u8 -> u8;
                fn to_u16 -> u16;
                fn to_u32 -> u32;
                fn to_u64 -> u64;
            }

            #[inline]
            fn to_f32(&self) -> Option<f32> {
                Some(*self as f32)
            }
            #[inline]
            fn to_f64(&self) -> Option<f64> {
                Some(*self as f64)
            }
        }
    };
}

implement_cast_uint!(usize);
implement_cast_uint!(u8);
implement_cast_uint!(u16);
implement_cast_uint!(u32);
implement_cast_uint!(u64);

macro_rules! implement_cast_float_to_float {
    ($SrcT:ident : $( fn $method:ident -> $DstT:ident ; )*) => {$(
        #[inline]
        fn $method(&self) -> Option<$DstT> {
            // We can safely cast all values, whether NaN, +-inf, or finite.
            // Finite values that are reducing size may saturate to +-inf.
            Some(*self as $DstT)
        }
    )*}
}

#[cfg(has_to_int_unchecked)]
macro_rules! float_to_int_unchecked {
    // SAFETY: Must not be NaN or infinite; must be representable as the integer after truncating.
    // We already checked that the float is in the exclusive range `(MIN-1, MAX+1)`.
    ($float:expr => $int:ty) => {
        unsafe { $float.to_int_unchecked::<$int>() }
    };
}

#[cfg(not(has_to_int_unchecked))]
macro_rules! float_to_int_unchecked {
    ($float:expr => $int:ty) => {
        $float as $int
    };
}

macro_rules! implement_cast_float_to_signed_int {
    ($f:ident : $( $(#[$cfg:meta])* fn $method:ident -> $i:ident ; )*) => {$(
        #[inline]
        $(#[$cfg])*
        fn $method(&self) -> Option<$i> {
            // Float as int truncates toward zero, so we want to allow values
            // in the exclusive range `(MIN-1, MAX+1)`.
            if size_of::<$f>() > size_of::<$i>() {
                // With a larger size, we can represent the range exactly.
                const MIN_M1: $f = $i::MIN as $f - 1.0;
                const MAX_P1: $f = $i::MAX as $f + 1.0;
                if *self > MIN_M1 && *self < MAX_P1 {
                    return Some(float_to_int_unchecked!(*self => $i));
                }
            } else {
                // We can't represent `MIN-1` exactly, but there's no fractional part
                // at this magnitude, so we can just use a `MIN` inclusive boundary.
                const MIN: $f = $i::MIN as $f;
                // We can't represent `MAX` exactly, but it will round up to exactly
                // `MAX+1` (a power of two) when we cast it.
                const MAX_P1: $f = $i::MAX as $f;
                if *self >= MIN && *self < MAX_P1 {
                    return Some(float_to_int_unchecked!(*self => $i));
                }
            }
            None
        }
    )*}
}

macro_rules! implement_cast_float_to_unsigned_int {
    ($f:ident : $( $(#[$cfg:meta])* fn $method:ident -> $u:ident ; )*) => {$(
        #[inline]
        $(#[$cfg])*
        fn $method(&self) -> Option<$u> {
            // Float as int truncates toward zero, so we want to allow values
            // in the exclusive range `(-1, MAX+1)`.
            if size_of::<$f>() > size_of::<$u>() {
                // With a larger size, we can represent the range exactly.
                const MAX_P1: $f = $u::MAX as $f + 1.0;
                if *self > -1.0 && *self < MAX_P1 {
                    return Some(float_to_int_unchecked!(*self => $u));
                }
            } else {
                // We can't represent `MAX` exactly, but it will round up to exactly
                // `MAX+1` (a power of two) when we cast it.
                // (`u128::MAX as f32` is infinity, but this is still ok.)
                const MAX_P1: $f = $u::MAX as $f;
                if *self > -1.0 && *self < MAX_P1 {
                    return Some(float_to_int_unchecked!(*self => $u));
                }
            }
            None
        }
    )*}
}

macro_rules! implement_cast_float {
    ($T:ident) => {
        impl CastToType for $T {
            implement_cast_float_to_signed_int! { $T:
                fn to_isize -> isize;
                fn to_i8 -> i8;
                fn to_i16 -> i16;
                fn to_i32 -> i32;
                fn to_i64 -> i64;
            }

            implement_cast_float_to_unsigned_int! { $T:
                fn to_usize -> usize;
                fn to_u8 -> u8;
                fn to_u16 -> u16;
                fn to_u32 -> u32;
                fn to_u64 -> u64;
            }

            implement_cast_float_to_float! { $T:
                fn to_f32 -> f32;
                fn to_f64 -> f64;
            }
        }
    };
}

implement_cast_float!(f32);
implement_cast_float!(f64);



macro_rules! implement_cast_with_wrapping {
    ($( $(#[$cfg:meta])* fn $method:ident -> $i:ident ; )*) => {$(
        #[inline]
        $(#[$cfg])*
        fn $method(&self) -> Option<$i> {
            (self.0).$method()
        }
    )*}
}

impl<T: CastToType> CastToType for Wrapping<T> {
    implement_cast_with_wrapping! {
        fn to_isize -> isize;
        fn to_i8 -> i8;
        fn to_i16 -> i16;
        fn to_i32 -> i32;
        fn to_i64 -> i64;

        fn to_usize -> usize;
        fn to_u8 -> u8;
        fn to_u16 -> u16;
        fn to_u32 -> u32;
        fn to_u64 -> u64;

        fn to_f32 -> f32;
        fn to_f64 -> f64;
    }
}



#[inline]
pub fn cast<T: NumberCast, U: NumberCast>(n: T) -> Option<U> {
    NumberCast::from(n)
}


pub trait NumberCast: Sized + CastToType {
    fn from<T: CastToType>(n: T) -> Option<Self>;
}

macro_rules! implement_number_cast {
    ($T:ty, $conv:ident) => {
        impl NumberCast for $T {
            #[inline]
            fn from<N: CastToType>(n: N) -> Option<$T> {
                n.$conv()
            }
        }
    };
}

implement_number_cast!(u8, to_u8);
implement_number_cast!(u16, to_u16);
implement_number_cast!(u32, to_u32);
implement_number_cast!(u64, to_u64);
implement_number_cast!(usize, to_usize);
implement_number_cast!(i8, to_i8);
implement_number_cast!(i16, to_i16);
implement_number_cast!(i32, to_i32);
implement_number_cast!(i64, to_i64);
implement_number_cast!(isize, to_isize);
implement_number_cast!(f32, to_f32);
implement_number_cast!(f64, to_f64);

impl<T: NumberCast> NumberCast for Wrapping<T> {
    fn from<U: CastToType>(n: U) -> Option<Self> {
        T::from(n).map(Wrapping)
    }
}

