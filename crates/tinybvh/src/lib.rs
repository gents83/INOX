#![cfg_attr(not(doctest), doc = include_str!("../README.md"))]

//! Safe rust wrapper for the C++ [tinybvh](https://github.com/jbikker/tinybvh) library.
//!
//! # Notes
//!
//! BVH layouts are splitted into:
//! - `BVHData`: read-only operations
//!     - Read nodes, and primitives slice
//!     - SAH cost, etc...
//! - `BVH`: Read-write operations
//!     - Building, converting, refitting, etc...
//!     - Implements `Deref` to access `BVHData` directly
//!
//! This is a pattern used throughout this crate to ensure maximum safety,
//! since the tinybvh library stores reference to primitives and original BVH upon
//! build and conversion.
//!
//! BVH layouts that manage their own primitives have no lifetime constraint.
//! This is for instance the case for [`cwbvh::BVH`] and [`bvh8_cpu::BVH`].

mod cxx_ffi;
mod layouts;
mod ray;
mod traversal;

pub(crate) use cxx_ffi::ffi;
pub use layouts::*;
pub use ray::*;
pub use traversal::*;

/// Infinite value used for intersection.
///
/// **NOTE**: This is not the same as `f32::MAX`.
pub const INFINITE: f32 = 1e30; // Actual valid ieee range: 3.40282347E+38

/// Alias for a strided slice of positions.
///
/// Positions do not need to be strided, but the API accepts a strided
/// slice to support both use cases.
///
/// tinybvh-rs internally requires positions to be vectors of size **4**
/// and not **3**. This is a requirement of the underlying tinybvh library.
pub type Positions<'a> = pas::Slice<'a, [f32; 4]>;

/// Error type for Wald BVH operations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    /// Positions slice wasn't triangulated. Provided positions must be a multiple of 3.
    PrimitiveTriangulated(usize),
    /// BVH can only be re-bound to a positions slice with the same size.
    BindInvalidPositionsLen(u32, u32),
    /// BVH must have a maximum primitive count per leaf.
    InvalidLeafCount(u32, u32),
}

impl Error {
    pub(crate) fn validate_triangulated(prim_len: usize) -> Result<(), Error> {
        if !prim_len.is_multiple_of(3) {
            Err(Error::PrimitiveTriangulated(prim_len))
        } else {
            Ok(())
        }
    }
    pub(crate) fn validate_primitives_len(expected: u32, prim_count: u32) -> Result<(), Error> {
        if expected != prim_count {
            Err(Error::BindInvalidPositionsLen(expected, prim_count))
        } else {
            Ok(())
        }
    }
    pub(crate) fn validate_leaf_count(expected: u32, count: Option<u32>) -> Result<(), Error> {
        if Some(expected) != count {
            Err(Error::InvalidLeafCount(expected, count.unwrap_or(u32::MAX)))
        } else {
            Ok(())
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::PrimitiveTriangulated(size) => {
                write!(
                    f,
                    "primitives slice must triangulated (size multiple of 3), got {}",
                    size
                )
            }
            Error::BindInvalidPositionsLen(expected, size) => {
                write!(
                    f,
                    "binding positions expected size {}, got {}",
                    expected, size
                )
            }
            Error::InvalidLeafCount(expected, size) => {
                write!(f, "expected at most {} per leaf, got {}", expected, size)
            }
        }
    }
}
