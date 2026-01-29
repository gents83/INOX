#ifndef TINYBVH_RUST
#define TINYBVH_RUST

#include <array>
#include <memory>

#include "rust/cxx.h"
#include "tinybvh-rs/ffi/tinybvh/tiny_bvh.h"

namespace tinybvh {

/* Math */
Ray ray_new(const std::array<float, 3>& origin, const std::array<float, 3>& dir);

/* BVH Wald 32 */

using BVHNode = BVH::BVHNode;
std::unique_ptr<BVH> BVH_new();
void BVH_setPrimitives(BVH& out, const bvhvec4slice& primitives);
rust::Slice<const BVHNode> BVH_nodes(const BVH&);
rust::Slice<const uint32_t> BVH_indices(const BVH&);
bool BVH_refittable(const BVH&);

/* MBVH8 */

using MBVH8 = MBVH<8>;
using MBVH8Node = MBVH<8>::MBVHNode;
template class MBVH<8>;

std::unique_ptr<MBVH8> MBVH8_new();
void MBVH8_setBVH(MBVH8& out, const BVH& bvh);
rust::Slice<const MBVH8Node> MBVH8_nodes(const MBVH8&);
uint32_t MBVH8_nodes_count(const MBVH8&);

/* BVH8_CPU */

std::unique_ptr<BVH8_CPU> BVH8_CPU_new();

/* CWBVH */

struct NodeCWBVH; // TODO: Remove once tinybvh provides a struct for CWBVH node.

std::unique_ptr<BVH8_CWBVH> CWBVH_new();
const uint8_t* CWBVH_nodes(const BVH8_CWBVH&);
uint32_t CWBVH_nodes_count(const BVH8_CWBVH&);
const uint8_t* CWBVH_primitives(const BVH8_CWBVH&);
uint32_t CWBVH_primitives_count(const BVH8_CWBVH&);

}

#endif
