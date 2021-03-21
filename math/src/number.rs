use super::cast_from::*;
use super::cast_to::*;
use super::one::*;
use super::zero::*;
use core::num::Wrapping;
use core::ops::*;

pub trait Operations<Rhs = Self, Output = Self>:
    Add<Rhs, Output = Output>
    + Sub<Rhs, Output = Output>
    + Mul<Rhs, Output = Output>
    + Div<Rhs, Output = Output>
    + Rem<Rhs, Output = Output>
{
}

impl<T, Rhs, Output> Operations<Rhs, Output> for T where
    T: Add<Rhs, Output = Output>
        + Sub<Rhs, Output = Output>
        + Mul<Rhs, Output = Output>
        + Div<Rhs, Output = Output>
        + Rem<Rhs, Output = Output>
{
}

pub trait Number:
    Copy
    + Clone
    + ::std::fmt::Debug
    + ::std::fmt::Display
    + NumberCast
    + CastFromType
    + PartialOrd
    + AddAssign
    + SubAssign
    + MulAssign
    + DivAssign
    + RemAssign
    + PartialEq
    + Zero
    + One
    + Operations
{
}

pub trait NumberRef: Number + for<'r> Operations<&'r Self> {}
impl<T> NumberRef for T where T: Number + for<'r> Operations<&'r T> {}

pub trait RefNumber<Base>: Operations<Base, Base> + for<'r> Operations<&'r Base, Base> {}
impl<T, Base> RefNumber<Base> for T where
    T: Operations<Base, Base> + for<'r> Operations<&'r Base, Base>
{
}

pub trait NumberAssignOperations<Rhs = Self>:
    AddAssign<Rhs> + SubAssign<Rhs> + MulAssign<Rhs> + DivAssign<Rhs> + RemAssign<Rhs>
{
}

impl<T, Rhs> NumberAssignOperations<Rhs> for T where
    T: AddAssign<Rhs> + SubAssign<Rhs> + MulAssign<Rhs> + DivAssign<Rhs> + RemAssign<Rhs>
{
}

pub trait NumberAssign: Number + NumberAssignOperations {}
impl<T> NumberAssign for T where T: Number + NumberAssignOperations {}

pub trait NumberAssignRef: Number + for<'r> NumberAssignOperations<&'r Self> {}
impl<T> NumberAssignRef for T where T: Number + for<'r> NumberAssignOperations<&'r T> {}

macro_rules! implement_number_for {
    ($name:ident for $($t:ty)*) => ($(
        impl $name for $t {}
    )*)
}

implement_number_for!(Number for usize u8 u16 u32 u64 isize i8 i16 i32 i64);
implement_number_for!(Number for f32 f64);

impl<T: Number> Number for Wrapping<T> where
    Wrapping<T>: Add<Output = Wrapping<T>>
        + Sub<Output = Wrapping<T>>
        + Mul<Output = Wrapping<T>>
        + Div<Output = Wrapping<T>>
        + Rem<Output = Wrapping<T>>
        + AddAssign
        + SubAssign
        + MulAssign
        + DivAssign
        + RemAssign
{
}
