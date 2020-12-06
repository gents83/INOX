
use core::ops::*;
use core::num::FpCategory;
use crate::implement_method_with_base;
use crate::implement_method_with_return_value;
use super::number::*;
use super::cast_to::*;

pub trait Float: Number + Copy + NumberCast + PartialOrd + Neg<Output = Self> {
    
    fn nan() -> Self;
    fn infinity() -> Self;
    fn neg_infinity() -> Self;
    fn neg_zero() -> Self;
    fn min_value() -> Self;
    fn min_positive_value() -> Self;
    fn epsilon() -> Self;
    fn max_value() -> Self;
    fn is_nan(self) -> bool;
    fn is_infinite(self) -> bool;
    fn is_finite(self) -> bool;
    fn is_normal(self) -> bool;
    fn classify(self) -> FpCategory;
    fn floor(self) -> Self;
    fn ceil(self) -> Self;
    fn round(self) -> Self;
    fn trunc(self) -> Self;
    fn fract(self) -> Self;
    fn abs(self) -> Self;
    fn signum(self) -> Self;
    fn is_sign_positive(self) -> bool;
    fn is_sign_negative(self) -> bool;
    fn mul_add(self, a: Self, b: Self) -> Self;
    fn recip(self) -> Self;
    fn powi(self, n: i32) -> Self;
    fn powf(self, n: Self) -> Self;
    fn sqrt(self) -> Self;
    fn exp(self) -> Self;
    fn exp2(self) -> Self;
    fn ln(self) -> Self;
    fn log(self, base: Self) -> Self;
    fn log2(self) -> Self;
    fn log10(self) -> Self;
    fn to_degrees(self) -> Self;
    fn to_radians(self) -> Self;
    fn max(self, other: Self) -> Self;
    fn min(self, other: Self) -> Self;
    fn cbrt(self) -> Self;
    fn hypot(self, other: Self) -> Self;
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn tan(self) -> Self;
    fn asin(self) -> Self;
    fn acos(self) -> Self;
    fn atan(self) -> Self;
    fn atan2(self, other: Self) -> Self;
    fn sin_cos(self) -> (Self, Self);
    fn exp_m1(self) -> Self;
    fn ln_1p(self) -> Self;
    fn sinh(self) -> Self;
    fn cosh(self) -> Self;
    fn tanh(self) -> Self;
    fn asinh(self) -> Self;
    fn acosh(self) -> Self;
    fn atanh(self) -> Self;
}

macro_rules! implement_std_float {
    ($T:ident) => {
        impl Float for $T {
            implement_method_with_return_value! {
                nan = $T::NAN,
                infinity = $T::INFINITY,
                neg_infinity = $T::NEG_INFINITY,
                neg_zero = -0.0,
                min_value = $T::MIN,
                min_positive_value = $T::MIN_POSITIVE,
                epsilon = $T::EPSILON,
                max_value = $T::MAX,
            }

            implement_method_with_base! {
                Self::is_nan(self) -> bool;
                Self::is_infinite(self) -> bool;
                Self::is_finite(self) -> bool;
                Self::is_normal(self) -> bool;
                Self::classify(self) -> FpCategory;
                Self::floor(self) -> Self;
                Self::ceil(self) -> Self;
                Self::round(self) -> Self;
                Self::trunc(self) -> Self;
                Self::fract(self) -> Self;
                Self::abs(self) -> Self;
                Self::signum(self) -> Self;
                Self::is_sign_positive(self) -> bool;
                Self::is_sign_negative(self) -> bool;
                Self::mul_add(self, a: Self, b: Self) -> Self;
                Self::recip(self) -> Self;
                Self::powi(self, n: i32) -> Self;
                Self::powf(self, n: Self) -> Self;
                Self::sqrt(self) -> Self;
                Self::exp(self) -> Self;
                Self::exp2(self) -> Self;
                Self::ln(self) -> Self;
                Self::log(self, base: Self) -> Self;
                Self::log2(self) -> Self;
                Self::log10(self) -> Self;
                Self::to_degrees(self) -> Self;
                Self::to_radians(self) -> Self;
                Self::max(self, other: Self) -> Self;
                Self::min(self, other: Self) -> Self;
                Self::cbrt(self) -> Self;
                Self::hypot(self, other: Self) -> Self;
                Self::sin(self) -> Self;
                Self::cos(self) -> Self;
                Self::tan(self) -> Self;
                Self::asin(self) -> Self;
                Self::acos(self) -> Self;
                Self::atan(self) -> Self;
                Self::atan2(self, other: Self) -> Self;
                Self::sin_cos(self) -> (Self, Self);
                Self::exp_m1(self) -> Self;
                Self::ln_1p(self) -> Self;
                Self::sinh(self) -> Self;
                Self::cosh(self) -> Self;
                Self::tanh(self) -> Self;
                Self::asinh(self) -> Self;
                Self::acosh(self) -> Self;
                Self::atanh(self) -> Self;
            }
        }
    };
}


implement_std_float!(f32);
implement_std_float!(f64);