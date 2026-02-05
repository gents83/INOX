use std::{fmt::Debug, marker::PhantomData};

use crate::{bvh, ffi, layouts::impl_bvh_deref, Error};

/// M-wide (aka 'shallow') BVH layout.
///
/// More information on the tinybvh repository (`MBVH::MBVHNode` struct).
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Node {
    pub aabb_min: [f32; 3],
    pub first_tri: u32,
    pub aabb_max: [f32; 3],
    pub tri_count: u32,
    pub child: [u32; 8],
    pub child_count: u32,
    pub dummy: [u32; ((30 - 8) & 3) + 1],
}

impl Node {
    /// Returns `true` if the node is a leaf.
    pub fn is_leaf(&self) -> bool {
        self.tri_count > 0
    }
}

/// Read-write BVH data.
pub struct BVHData {
    pub(crate) inner: cxx::UniquePtr<ffi::MBVH8>,
    pub(crate) max_primitives_per_leaf: Option<u32>,
    pub(crate) primitives_len: u32,
}

impl BVHData {
    /// Number of leaf in the hierarchy starting at `node_index`.
    pub fn leaf_count(&self, node_index: u32) -> u32 {
        self.inner.LeafCount(node_index)
    }

    /// Move this instance into a writable BVH.
    ///
    /// More information on [`bvh::BVHData::bvh`].
    pub fn bvh<'a>(self, original: &'a bvh::BVH<'a>) -> Result<BVH<'a>, Error> {
        BVH::wrap(self).bind(original)
    }

    /// Shortcut for:
    ///
    /// ```ignore
    /// mbvh.bvh(&original).convert(&original)
    /// ```
    pub fn convert<'a>(self, original: &'a bvh::BVH<'a>) -> BVH<'a> {
        BVH::wrap(self).convert(original)
    }

    /// BVH nodes.
    pub fn nodes(&self) -> &[Node] {
        ffi::MBVH8_nodes(&self.inner)
    }
}

/// Read-write BVH data.
///
/// BVH isn't built from primitives but from converting [`bvh::BVH`].
pub struct BVH<'a> {
    bvh: BVHData,
    _phantom: PhantomData<bvh::BVH<'a>>,
}
impl_bvh_deref!(BVH<'a>, BVHData);

impl<'a> BVH<'a> {
    fn wrap(bvh: BVHData) -> Self {
        Self {
            bvh,
            _phantom: PhantomData,
        }
    }

    /// Create a new BVH converting `original`.
    pub fn new(original: &'a bvh::BVH) -> Self {
        let data = BVHData {
            inner: ffi::MBVH8_new(),
            max_primitives_per_leaf: None,
            primitives_len: 0,
        };
        data.convert(original)
    }

    /// Update the original BVH reference.
    ///
    /// Note: At the opposite of [`BVH::convert`], no rebuild is performed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use tinybvh_rs::{bvh, mbvh};
    /// # let triangles = vec![[0.0; 4], [0.0; 4], [0.0; 4]];
    /// let bvh = bvh::BVH::new(triangles.as_slice().into()).unwrap();
    /// let mbvh = mbvh::BVH::new(&bvh);
    ///
    /// let other_bvh = bvh::BVH::new(triangles.as_slice().into()).unwrap();
    /// let mbvh = mbvh.bind(&other_bvh);
    /// ```
    pub fn bind<'b>(self, original: &'b bvh::BVH) -> Result<BVH<'b>, Error> {
        let mut bvh = self.bvh;
        crate::Error::validate_primitives_len(bvh.primitives_len, original.primitives_len)?;
        ffi::MBVH8_setBVH(bvh.inner.pin_mut(), &original.bvh.inner);
        Ok(BVH::wrap(bvh))
    }

    /// Convert (i.e., build) the BVH and bind it to `original`.
    ///
    /// More information on the tinybvh repository (`MBVH::ConvertFrom()` method).
    pub fn convert<'b>(self, original: &'b bvh::BVH) -> BVH<'b> {
        let mut bvh = self.bvh;
        bvh.inner
            .pin_mut()
            .ConvertFrom(original.bvh.inner.as_ref().unwrap(), true);
        bvh.max_primitives_per_leaf = original.max_primitives_per_leaf;
        bvh.primitives_len = original.primitives_len;
        BVH::wrap(bvh)
    }

    /// Refits the BVH.
    ///
    /// More information on the tinybvh repository (`MBVH::Refit()` method).
    pub fn refit(&mut self, node_index: u32) {
        self.bvh.inner.pin_mut().Refit(node_index);
    }

    /// Move the BVH into read-only state.
    pub fn data(self) -> BVHData {
        self.bvh
    }
}
