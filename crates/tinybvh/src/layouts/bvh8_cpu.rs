//! 8-way BVH
//!
//! Based on "Accelerated Single Ray Tracing for Wide Vector Units".

use crate::{ffi, mbvh, Error};

/// Read-write BVH.
///
/// At the opposite of [`crate::bvh::BVH`] and [`mbvh::BVH`], has no lifetime bound
/// because it manages its own primitives,
pub struct BVH {
    pub(crate) inner: cxx::UniquePtr<ffi::BVH8_CPU>,
}

impl BVH {
    /// Create a new BVH converting `original`.
    pub fn new(original: &mbvh::BVH) -> Result<Self, Error> {
        Self {
            inner: ffi::BVH8_CPU_new(),
        }
        .convert(original)
    }

    /// Convert (i.e., build) the BVH and bind it to `original`.
    ///
    /// More information on the tinybvh repository (`BVH8_CPU::ConvertFrom()` method).
    pub fn convert(mut self, original: &mbvh::BVH) -> Result<Self, Error> {
        Error::validate_leaf_count(4, original.max_primitives_per_leaf)?;
        self.inner
            .pin_mut()
            .ConvertFrom(original.inner.as_ref().unwrap());
        Ok(self)
    }
}

#[cfg(target_feature = "avx2")]
impl crate::Intersector for BVH {
    /// At the opposite of other layouts, always returns `0`
    fn intersect(&self, ray: &mut crate::Ray) -> u32 {
        self.inner.Intersect(ray) as u32
    }
}
