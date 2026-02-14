#import "common.inc"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> bvh: BVH;
@group(0) @binding(2)
var<storage, read> indices: Indices;
@group(0) @binding(3)
var<storage, read> vertices_positions: VerticesPositions;
@group(0) @binding(4)
var<storage, read> instances: Instances;
@group(0) @binding(5)
var<storage, read> meshlets: Meshlets;

@group(1) @binding(0)
var<storage, read> shadow_data: array<f32>;
@group(1) @binding(1)
var<storage, read_write> radiance: array<f32>;

const WORKGROUP_SIZE: u32 = 8u;

@compute
@workgroup_size(WORKGROUP_SIZE, WORKGROUP_SIZE, 1)
fn main() {
    // Placeholder
}
