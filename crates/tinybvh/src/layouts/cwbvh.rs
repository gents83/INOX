//! CWBVH GPU-friendly layout
//!
//! Based on "Efficient Incoherent Ray Traversal on GPUs Through Compressed Wide BVHs".

use crate::{ffi, mbvh, Error};
use std::fmt::Debug;

pub struct PrimitiveIter {
    primitive_base_index: u32,
    child_meta: [u8; 8],

    curr_meta_idx: u8,
    curr_tri_count: u8,
}

impl PrimitiveIter {
    fn new(base_index: u32, meta: [u8; 8]) -> Self {
        Self {
            primitive_base_index: base_index,
            child_meta: meta,

            curr_meta_idx: 0,
            curr_tri_count: 0,
        }
    }
}

impl Iterator for PrimitiveIter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_meta_idx as usize >= self.child_meta.len() {
            return None;
        }
        while self.curr_meta_idx < self.child_meta.len() as u8 {
            let meta = self.child_meta[self.curr_meta_idx as usize];
            let triangles_count = (meta & 0b11100000).count_ones() as u8;
            let current_tri_count = self.curr_tri_count;
            self.curr_tri_count += 1;
            if current_tri_count < triangles_count {
                let start = meta & 0b00011111;
                return Some(self.primitive_base_index + start as u32 + current_tri_count as u32);
            }
            self.curr_meta_idx += 1;
            self.curr_tri_count = 0;
        }
        None
    }
}

/// Format specified in:
/// "Efficient Incoherent Ray Traversal on GPUs Through Compressed Wide BVHs", Ylitie et al. 2017.
///
/// Node layout used by [`BVH`].
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Node {
    /// AABB min.
    pub min: [f32; 3],
    /// Exponent used for child AABB decompression.
    pub exyz: [u8; 3],
    /// `1` for node, `0` for leaf.
    pub imask: u8,
    /// First child index.
    pub child_base_idx: u32,
    // First primitive index.
    pub primitive_base_idx: u32,
    /// Child [0..7] metadata.
    pub child_meta: [u8; 8],
    // AABB minimum x-axis compressed bound, one entry per child.
    pub qlo_x: [u8; 8],
    // AABB minimum y-axis compressed bound, one entry per child.
    pub qlo_y: [u8; 8],
    // AABB minimum z-axis compressed bound, one entry per child.
    pub qlo_z: [u8; 8],
    // AABB maximum x-axis compressed bound, one entry per child.
    pub qhi_x: [u8; 8],
    // AABB maximum y-axis compressed bound, one entry per child.
    pub qhi_y: [u8; 8],
    // AABB maximum z-axis compressed bound, one entry per child.
    pub qhi_z: [u8; 8],
}

impl Node {
    /// Returns `true` if the node is a leaf.
    pub fn is_leaf(&self) -> bool {
        self.imask == 0
    }

    pub fn primitives(&self) -> PrimitiveIter {
        if !self.is_leaf() {
            return PrimitiveIter::new(0, [0, 0, 0, 0, 0, 0, 0, 0]);
        }
        PrimitiveIter::new(self.primitive_base_idx, self.child_meta)
    }
}

/// Custom primitive used by [`BVH`].
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Primitive {
    pub edge_1: [f32; 3],
    pub padding_0: u32,
    pub edge_2: [f32; 3],
    pub padding_1: u32,
    pub vertex_0: [f32; 3],
    pub original_primitive: u32,
}

impl Debug for Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("cwbvh::Primitive")
            .field("vertex_0", &self.vertex_0)
            .field("edge_1", &self.edge_1)
            .field("edge_2", &self.edge_2)
            .field("original_primitive", &self.original_primitive)
            .finish()
    }
}

/// CWBVH with node layout [`Node`].
pub struct BVH {
    inner: cxx::UniquePtr<ffi::BVH8_CWBVH>,
}

impl BVH {
    pub fn new(original: &mbvh::BVH) -> Result<Self, Error> {
        let bvh = BVH {
            inner: ffi::CWBVH_new(),
        };
        bvh.convert(original)
    }

    pub fn convert(mut self, original: &mbvh::BVH) -> Result<Self, Error> {
        Error::validate_leaf_count(3, original.max_primitives_per_leaf)?;
        self.inner
            .pin_mut()
            .ConvertFrom(original.inner.as_ref().unwrap(), true);
        Ok(self)
    }

    pub fn nodes(&self) -> &[Node] {
        // TODO: Create CWBVH node in tinybvh to avoid that.
        let ptr = ffi::CWBVH_nodes(&self.inner) as *const Node;
        let count = ffi::CWBVH_nodes_count(&self.inner);
        unsafe { std::slice::from_raw_parts(ptr, count as usize) }
    }

    /// Encoded primitive data.
    ///
    /// This layout is intersected using a custom primitive array
    /// instead of the original list used during building.
    pub fn primitives(&self) -> &[Primitive] {
        // TODO: Create struct in tinybvh to avoid that.
        let ptr = ffi::CWBVH_primitives(&self.inner) as *const Primitive;
        let count = ffi::CWBVH_primitives_count(&self.inner);
        unsafe { std::slice::from_raw_parts(ptr, count as usize) }
    }
}
