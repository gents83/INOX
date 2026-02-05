//! "Wald" BVH layout
//!
//! Main BVH layout used to construct other layouts.
//!
//! This layout is split into:
//! - [`BVHData`]: Read-only operations
//! - [`BVH`]: Read-write operations
//!     - Read operations mixing layout and referenced data, such as vertices
//!     - Build, refit

use crate::{ffi, layouts::impl_bvh_deref, Error};
use std::{fmt::Debug, marker::PhantomData};

/// "Traditional" 32-bytes BVH node layout, as proposed by Ingo Wald.
///
/// Node layout used by [`BVH`].
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Node {
    /// AABB min position.
    pub min: [f32; 3],
    /// If the node is a leaf, this is the start index of the primitive.
    /// Otherwise, this is the start index of the first child node.
    pub left_first: u32,
    /// AABB max position.
    pub max: [f32; 3],
    /// If the node is a leaf, number of triangles in the node.
    /// `0` otherwise.
    pub tri_count: u32,
}

impl Node {
    /// Returns `true` if the node is a leaf.
    pub fn is_leaf(&self) -> bool {
        self.tri_count > 0
    }
}

/// Read-only BVH data.
pub struct BVHData {
    pub(crate) inner: cxx::UniquePtr<ffi::BVH>,
    pub(crate) max_primitives_per_leaf: Option<u32>,
    /// Primitive slice len, **not** primitive count.
    pub(crate) primitives_len: u32,
}

impl BVHData {
    fn new() -> Self {
        Self {
            inner: ffi::BVH_new(),
            max_primitives_per_leaf: None,
            primitives_len: 0,
        }
    }

    /// Move this instance into a writable BVH.
    ///
    /// Note: `primitives` slice must have a length multiple of 3.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use tinybvh_rs::bvh;
    ///
    /// let triangles = vec![[-1.0, 1.0, 0.0, 0.0], [1.0, 1.0, 0.0, 0.0], [-1.0, 0.0, 0.0, 0.0]];
    /// # let bvh = bvh::BVH::new(triangles.as_slice().into()).unwrap();
    /// let data = bvh.data();
    /// let mut bvh = data.bvh(triangles.as_slice().into()).unwrap();
    /// bvh.refit();
    /// ```
    pub fn bvh<'a>(self, primitives: crate::Positions<'a>) -> Result<BVH<'a>, Error> {
        BVH {
            bvh: self,
            _phantom: PhantomData,
        }
        .bind(primitives)
    }

    /// Number of primitives for a given node.
    pub fn primitive_count(&self, id: u32) -> u32 {
        self.inner.PrimCount(id) as u32
    }

    /// SAH cost for a subtree.
    pub fn sah_cost(&self, id: u32) -> f32 {
        self.inner.SAHCost(id)
    }

    /// BVH nodes.
    ///
    /// Useful to upload to the BVH to the GPU.
    pub fn nodes(&self) -> &[Node] {
        ffi::BVH_nodes(&self.inner)
    }

    /// BVH indices.
    ///
    /// Map from primitive index to first vertex index.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use tinybvh_rs::bvh;
    /// # let primitives = vec![[0.0; 4], [0.0; 4], [0.0; 4]];
    /// # let bvh = bvh::BVH::new(primitives.as_slice().into()).unwrap();
    /// # let node = bvh.nodes()[0];
    /// for i in 0..node.tri_count {
    ///     let vertex_start = bvh.indices()[(node.left_first + i) as usize] as usize * 3;
    ///     let vertex = [
    ///         primitives[vertex_start],
    ///         primitives[vertex_start + 1],
    ///         primitives[vertex_start + 2]
    ///     ];
    ///     println!("Vertex {:?}", vertex);
    /// }
    /// ```
    pub fn indices(&self) -> &[u32] {
        ffi::BVH_indices(&self.inner)
    }

    /// Returns `true` if the BVH can be refitted.
    ///
    /// A BVH is refittable if it was built without spatial splits (i.e., not an SBVH).
    pub fn refittable(&self) -> bool {
        ffi::BVH_refittable(&self.inner)
    }
}

/// Read-write BVH data.
///
/// The BVH is bound to the positions until moved into a [`BVHData`].
/// More information on the [`BVH::bind`] method.
///
/// # Examples
///
/// ```rust
/// use tinybvh_rs::bvh;
///
/// let triangles = vec![[-1.0, 1.0, 0.0, 0.0], [1.0, 1.0, 0.0, 0.0], [-1.0, 0.0, 0.0, 0.0]];
/// let bvh = bvh::BVH::new(triangles.as_slice().into());
/// ```
pub struct BVH<'a> {
    pub(crate) bvh: BVHData,
    _phantom: PhantomData<&'a [f32; 4]>,
}
impl_bvh_deref!(BVH<'a>, BVHData);

impl<'a> BVH<'a> {
    /// Create a new BVH and build it using [`BVH::build`].
    pub fn new(primitives: crate::Positions<'a>) -> Result<Self, Error> {
        Self {
            bvh: BVHData::new(),
            _phantom: PhantomData,
        }
        .build(primitives)
    }

    /// Create a new BVH and build it using [`BVH::build_hq`].
    pub fn new_hq(primitives: crate::Positions<'a>) -> Result<Self, Error> {
        Self {
            bvh: BVHData::new(),
            _phantom: PhantomData,
        }
        .build_hq(primitives)
    }

    /// Build the BVH and bind it to `primitives`.
    ///
    /// More information on the tinybvh repository (`BVH::Build()` method).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tinybvh_rs::bvh;
    ///
    /// let triangles = vec![[-1.0, 1.0, 0.0, 0.0], [1.0, 1.0, 0.0, 0.0], [-1.0, 0.0, 0.0, 0.0]];
    /// # let bvh = bvh::BVH::new(triangles.as_slice().into()).unwrap();
    /// let bvh = bvh.build(triangles.as_slice().into());
    /// ```
    pub fn build<'b>(self, primitives: crate::Positions<'b>) -> Result<BVH<'b>, Error> {
        Error::validate_triangulated(primitives.len())?;
        let slice = primitives.into();
        let mut bvh = self.bvh;
        bvh.inner.pin_mut().Build(&slice);
        Ok(BVH::build_internal(bvh, primitives))
    }

    /// Build the BVH and bind it to `primitives`.
    ///
    /// More information on the tinybvh repository (`BVH::BuildHQ()` method).
    pub fn build_hq<'b>(self, primitives: crate::Positions<'b>) -> Result<BVH<'b>, Error> {
        Error::validate_triangulated(primitives.len())?;
        let slice = primitives.into();
        let mut bvh = self.bvh;
        bvh.inner.pin_mut().BuildHQ(&slice);
        Ok(BVH::build_internal(bvh, primitives))
    }

    /// Update the primitives reference.
    ///
    /// Note: At the opposite of [`BVH::build`], no rebuild is performed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use tinybvh_rs::{bvh, Intersector, Ray};
    ///
    /// let mut triangles = vec![
    ///     [-1.0, 1.0, 0.0, 0.0],
    ///     [1.0, 1.0, 0.0, 0.0],
    ///     [-1.0, 0.0, 0.0, 0.0],
    /// ];
    /// let bvh = bvh::BVH::new(triangles.as_slice().into()).unwrap();
    ///
    /// let mut other_triangles = vec![
    ///     [-2.0, 2.0, 0.0, 0.0],
    ///     [2.0, 2.0, 0.0, 0.0],
    ///     [-2.0, 0.0, 0.0, 0.0]
    /// ];
    /// let bvh = bvh.bind(other_triangles.as_slice().into()).unwrap();
    ///
    /// let mut ray = Ray::new([0.0; 3], [0.0, 0.0, -1.0]);
    /// bvh.intersect(&mut ray);
    /// ```
    pub fn bind<'b>(self, primitives: crate::Positions<'b>) -> Result<BVH<'b>, Error> {
        let mut bvh = self.bvh;
        Error::validate_primitives_len(bvh.primitives_len, primitives.len() as u32)?;

        let slice = primitives.into();
        ffi::BVH_setPrimitives(bvh.inner.pin_mut(), &slice);
        Ok(BVH {
            bvh,
            _phantom: PhantomData,
        })
    }

    /// Remove unused nodes and reduce BVH size.
    ///
    /// More information on the tinybvh repository (`BVH::Compact()` method).
    pub fn compact(&mut self) {
        self.bvh.inner.pin_mut().Compact();
    }

    /// Split BVH leaves into a at most `max_primitives` primitives.
    ///
    /// More information on the tinybvh repository (`BVH::SplitLeafs()` method).
    pub fn split_leaves(&mut self, max_primitives: u32) {
        self.bvh.inner.pin_mut().SplitLeafs(max_primitives);

        let max_prim = self.bvh.max_primitives_per_leaf.unwrap_or(u32::MAX);
        self.bvh.max_primitives_per_leaf = Some(u32::min(max_prim, max_primitives));
    }

    /// Refits the BVH.
    ///
    /// Note: This library diverges from tinybvh here that asserts if it's not refittable.
    /// tinybvh_rs skips refitting instead.
    ///
    /// More information on the tinybvh repository (`BVH::Refit()` method).
    pub fn refit(&mut self) {
        if self.refittable() {
            self.bvh.inner.pin_mut().Refit(0);
        }
    }

    /// Move the BVH into read-only state.
    ///
    /// Allows to relax the lifetime bound, for instance to modify
    /// the positions before a refit:
    ///
    /// ```rust
    /// # use tinybvh_rs::bvh;
    /// let data = {
    ///     let mut triangles = vec![[-1.0, 1.0, 0.0, 0.0], [1.0, 1.0, 0.0, 0.0], [-1.0, 0.0, 0.0, 0.0]];
    ///     let bvh = bvh::BVH::new(triangles.as_slice().into()).unwrap();
    ///     bvh.data()
    /// }; // `triangles` is dropped by now
    /// println!("Is leaf: {}", data.nodes()[0].is_leaf()); // false
    /// ```
    pub fn data(self) -> BVHData {
        self.bvh
    }

    fn build_internal<'b>(mut bvh: BVHData, primitives: crate::Positions<'b>) -> BVH<'b> {
        bvh.max_primitives_per_leaf = None;
        bvh.primitives_len = primitives.len() as u32;
        BVH {
            bvh,
            _phantom: PhantomData,
        }
    }
}

impl crate::Intersector for BVH<'_> {
    fn intersect(&self, ray: &mut crate::Ray) -> u32 {
        self.bvh.inner.Intersect(ray) as u32
    }
}
