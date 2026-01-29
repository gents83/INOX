#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec4Slice {
    data: *const i32,
    count: u32,
    stride: u32,
}

impl From<pas::Slice<'_, [f32; 4]>> for Vec4Slice {
    fn from(value: pas::Slice<[f32; 4]>) -> Self {
        Self {
            data: value.as_ptr() as *const i32,
            count: value.len() as u32,
            stride: value.stride() as u32,
        }
    }
}

// Ensure `bvhvec4slice` always has a trivial move ctor and no destructor
unsafe impl cxx::ExternType for Vec4Slice {
    type Id = cxx::type_id!("tinybvh::bvhvec4slice");
    type Kind = cxx::kind::Trivial;
}
// Ensure `Intersection` always has a trivial move ctor and no destructor
unsafe impl cxx::ExternType for crate::Intersection {
    type Id = cxx::type_id!("tinybvh::Intersection");
    type Kind = cxx::kind::Trivial;
}
// Ensure `Ray` always has a trivial move ctor and no destructor
unsafe impl cxx::ExternType for crate::Ray {
    type Id = cxx::type_id!("tinybvh::Ray");
    type Kind = cxx::kind::Trivial;
}
// Ensure `BVH::BVHNode` always has a trivial move ctor and no destructor
unsafe impl cxx::ExternType for crate::bvh::Node {
    type Id = cxx::type_id!("tinybvh::BVHNode");
    type Kind = cxx::kind::Trivial;
}
unsafe impl cxx::ExternType for crate::mbvh::Node {
    type Id = cxx::type_id!("tinybvh::MBVH8Node");
    type Kind = cxx::kind::Trivial;
}

#[cxx::bridge(namespace = "tinybvh")]
pub(crate) mod ffi {
    unsafe extern "C++" {
        include!("tinybvh-rs/ffi/include/tinybvh.h");

        // Utils
        pub type bvhvec4slice = super::Vec4Slice;
        pub type Ray = crate::Ray;
        pub fn ray_new(origin: &[f32; 3], dir: &[f32; 3]) -> Ray;

        // BVH
        pub type BVH;
        pub type BVHNode = crate::bvh::Node;
        pub fn BVH_new() -> UniquePtr<BVH>;
        pub fn BVH_nodes(bvh: &BVH) -> &[BVHNode];
        pub fn BVH_indices(bvh: &BVH) -> &[u32];
        pub fn BVH_refittable(bvh: &BVH) -> bool;
        pub fn BVH_setPrimitives(out: Pin<&mut BVH>, primitives: &bvhvec4slice);
        pub fn Build(self: Pin<&mut BVH>, primitives: &bvhvec4slice);
        pub fn BuildHQ(self: Pin<&mut BVH>, primitives: &bvhvec4slice);
        pub fn Compact(self: Pin<&mut BVH>);
        pub fn SplitLeafs(self: Pin<&mut BVH>, count: u32);
        pub fn Refit(self: Pin<&mut BVH>, node_idx: u32);
        pub fn SAHCost(self: &BVH, node_idx: u32) -> f32;
        pub fn PrimCount(self: &BVH, node_idx: u32) -> i32;
        pub fn Intersect(self: &BVH, original: &mut Ray) -> i32;

        // MBVH8
        pub type MBVH8;
        pub type MBVH8Node = crate::mbvh::Node;
        pub fn MBVH8_new() -> UniquePtr<MBVH8>;
        pub fn MBVH8_setBVH(out: Pin<&mut MBVH8>, bvh: &BVH);
        pub fn MBVH8_nodes(bvh: &MBVH8) -> &[MBVH8Node];
        pub fn ConvertFrom(self: Pin<&mut MBVH8>, bvh: &BVH, compact: bool);
        pub fn Refit(self: Pin<&mut MBVH8>, node_index: u32);
        pub fn LeafCount(self: &MBVH8, node_index: u32) -> u32;

        // BVH8_CPU
        pub type BVH8_CPU;
        pub fn BVH8_CPU_new() -> UniquePtr<BVH8_CPU>;
        pub fn ConvertFrom(self: Pin<&mut BVH8_CPU>, bvh: &MBVH8);
        #[allow(dead_code)]
        pub fn Intersect(self: &BVH8_CPU, ray: &mut Ray) -> i32;

        // CWBVH
        pub type BVH8_CWBVH;
        pub fn CWBVH_new() -> UniquePtr<BVH8_CWBVH>;
        pub fn CWBVH_nodes(bvh: &BVH8_CWBVH) -> *const u8;
        pub fn CWBVH_nodes_count(bvh: &BVH8_CWBVH) -> u32;
        pub fn CWBVH_primitives(bvh: &BVH8_CWBVH) -> *const u8;
        pub fn CWBVH_primitives_count(bvh: &BVH8_CWBVH) -> u32;
        pub fn ConvertFrom(self: Pin<&mut BVH8_CWBVH>, bvh: &MBVH8, compact: bool);
    }
}
