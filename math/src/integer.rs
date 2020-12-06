
use core::ops::*;
use super::boundaries::*;
use super::cast_to::*;
use super::number::*;

pub trait Integer:
Sized
+ Copy
+ Number
+ NumberCast
+ Boundaries
+ PartialOrd
+ Ord
+ Eq
+ Not<Output = Self>
+ BitAnd<Output = Self>
+ BitOr<Output = Self>
+ BitXor<Output = Self>
+ Shl<usize, Output = Self>
+ Shr<usize, Output = Self>
{
    /// Returns the number of ones in the binary representation
    fn count_ones(self) -> u32;

    /// Returns the number of zeros in the binary representation
    fn count_zeros(self) -> u32;

    /// Returns the number of leading zeros in the binary representation
    fn leading_zeros(self) -> u32;

    /// Returns the number of trailing zeros in the binary representation
    fn trailing_zeros(self) -> u32;

    /// Shifts the bits to the left by a specified amount, `n`, wrapping
    /// the truncated bits to the end of the resulting integer
    fn rotate_left(self, n: u32) -> Self;

    /// Shifts the bits to the right by a specified amount, `n`, wrapping
    /// the truncated bits to the beginning of the resulting integer
    fn rotate_right(self, n: u32) -> Self;

    /// Shifts the bits to the left by a specified amount, `n`, filling
    /// zeros in the least significant bits
    fn signed_shl(self, n: u32) -> Self;

    /// Shifts the bits to the right by a specified amount, `n`, copying
    /// the "sign bit" in the most significant bits even for unsigned types
    fn signed_shr(self, n: u32) -> Self;

    /// Shifts the bits to the left by a specified amount, `n`, filling
    /// zeros in the least significant bits
    fn unsigned_shl(self, n: u32) -> Self;

    /// Shifts the bits to the right by a specified amount, `n`, filling
    /// zeros in the most significant bits
    fn unsigned_shr(self, n: u32) -> Self;

    /// Reverses the byte order of the integer
    fn swap_bytes(self) -> Self;
    
    fn pow(self, exp: u32) -> Self;
}

macro_rules! implement_integer {
    ($T:ty, $S:ty, $U:ty) => {
        impl Integer for $T {
            #[inline]
            fn count_ones(self) -> u32 {
                <$T>::count_ones(self)
            }

            #[inline]
            fn count_zeros(self) -> u32 {
                <$T>::count_zeros(self)
            }

            #[inline]
            fn leading_zeros(self) -> u32 {
                <$T>::leading_zeros(self)
            }

            #[inline]
            fn trailing_zeros(self) -> u32 {
                <$T>::trailing_zeros(self)
            }

            #[inline]
            fn rotate_left(self, n: u32) -> Self {
                <$T>::rotate_left(self, n)
            }

            #[inline]
            fn rotate_right(self, n: u32) -> Self {
                <$T>::rotate_right(self, n)
            }

            #[inline]
            fn signed_shl(self, n: u32) -> Self {
                ((self as $S) << n) as $T
            }

            #[inline]
            fn signed_shr(self, n: u32) -> Self {
                ((self as $S) >> n) as $T
            }

            #[inline]
            fn unsigned_shl(self, n: u32) -> Self {
                ((self as $U) << n) as $T
            }

            #[inline]
            fn unsigned_shr(self, n: u32) -> Self {
                ((self as $U) >> n) as $T
            }

            #[inline]
            fn swap_bytes(self) -> Self {
                <$T>::swap_bytes(self)
            }
            
            #[inline]
            fn pow(self, exp: u32) -> Self {
                <$T>::pow(self, exp)
            }
        }
    };
}

implement_integer!(u8, i8, u8);
implement_integer!(u16, i16, u16);
implement_integer!(u32, i32, u32);
implement_integer!(u64, i64, u64);
implement_integer!(usize, isize, usize);
implement_integer!(i8, i8, u8);
implement_integer!(i16, i16, u16);
implement_integer!(i32, i32, u32);
implement_integer!(i64, i64, u64);
implement_integer!(isize, isize, usize);