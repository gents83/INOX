#include "raytracing_structs.inc"

@group(0) @binding(0) var<uniform> constant_data: ConstantData;
@group(0) @binding(1) var<storage, read> geometry_buffer: array<u32>;
@group(0) @binding(2) var<storage, read> scene_buffer: array<u32>;

// Helper functions that use these bindings... (moved from common)
// Copy-paste helper functions here or in another include.
// To keep it simple, I'll put helpers in `raytracing_helpers.inc` which includes bindings.

const MESH_STRIDE: u32 = 23u;
const MESHLET_STRIDE: u32 = 20u;
const INSTANCE_STRIDE: u32 = 4u;
const TRANSFORM_STRIDE: u32 = 16u;

// ... Helpers (unpack_position etc) ...
