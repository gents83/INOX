pub mod bvh;
pub mod bvh8_cpu;
pub mod cwbvh;
pub mod mbvh;

/// Implements `Deref` for a BVH type that wraps a `BVHData`.
macro_rules! impl_bvh_deref {
    ($bvh:ty, $target:ty) => {
        impl<'a> std::ops::Deref for $bvh {
            type Target = $target;

            fn deref(&self) -> &Self::Target {
                &self.bvh
            }
        }
    };
}

pub(crate) use impl_bvh_deref;
