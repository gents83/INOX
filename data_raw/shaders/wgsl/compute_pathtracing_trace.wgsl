#import "common.inc"
#import "utils.inc"

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
var<storage, read> rays: array<f32>;
@group(1) @binding(1)
var<storage, read_write> hits: array<f32>;

#import "matrix_utils.inc"
#import "geom_utils.inc"
#import "raytracing.inc"

const WORKGROUP_SIZE: u32 = 8u;

@compute
@workgroup_size(WORKGROUP_SIZE, WORKGROUP_SIZE, 1)
fn main(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let dimensions = vec2<u32>(DEFAULT_WIDTH, DEFAULT_HEIGHT);
    let pixel = vec2<u32>(global_invocation_id.x, global_invocation_id.y);
    if (pixel.x >= dimensions.x || pixel.y >= dimensions.y) {
        return;
    }

    let ray_index = (global_invocation_id.y * dimensions.x + global_invocation_id.x) * SIZE_OF_DATA_BUFFER_ELEMENT;

    let origin = vec3<f32>(rays[ray_index], rays[ray_index + 1u], rays[ray_index + 2u]);
    let packed_direction = rays[ray_index + 3u];

    if (packed_direction == 0.) {
        hits[ray_index] = 0.;
        hits[ray_index + 1u] = 0.;
        return;
    }

    let direction = octahedral_unmapping(unpack2x16float(u32(packed_direction)));

    let result = traverse_bvh(origin, direction, constant_data.tlas_starting_index);

    hits[ray_index] = f32(result.visibility_id);
    hits[ray_index + 1u] = result.distance;
}
