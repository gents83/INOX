#import "common.inc"
#import "utils.inc"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> indices: Indices;
@group(0) @binding(2)
var<storage, read> vertices_positions: VerticesPositions;
@group(0) @binding(3)
var<storage, read> instances: Instances;
@group(0) @binding(4)
var<storage, read> transforms: Transforms;
@group(0) @binding(5)
var<storage, read> meshes: Meshes;
@group(0) @binding(6)
var<storage, read> meshlets: Meshlets;
@group(0) @binding(7)
var<storage, read> bvh: BVH;

@group(1) @binding(0)
var<storage, read_write> data_buffer_0: array<f32>;
@group(1) @binding(1)
var<storage, read_write> data_buffer_1: array<f32>;
@group(1) @binding(2)
var<storage, read_write> data_buffer_2: array<f32>;

#import "matrix_utils.inc"
#import "geom_utils.inc"
#import "raytracing.inc"
#import "pathtracing.inc"

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

    let data_index = (global_invocation_id.y * dimensions.x + global_invocation_id.x) * SIZE_OF_DATA_BUFFER_ELEMENT;

    // Check if this ray is still active
    if (!read_ray_active(data_index)) {
        return;
    }

    // Read ray
    let origin = read_ray_origin(data_index);
    let direction = read_ray_direction(data_index);

    // Traverse BVH
    let result = traverse_bvh(origin, direction, constant_data.tlas_starting_index);

    let is_miss = result.visibility_id == 0u || (result.visibility_id & 0xFFFFFFFFu) == 0xFF000000u;

    if (is_miss) {
        // Write miss as hit data — bounce shade will handle sky contribution
        data_buffer_0[data_index] = origin.x;
        data_buffer_0[data_index + 1u] = origin.y;
        data_buffer_0[data_index + 2u] = origin.z;
        // Encode miss with special distance value
        data_buffer_0[data_index + 3u] = -1.0;
    } else {
        // Write hit data
        let hit_point = origin + direction * result.distance;
        data_buffer_0[data_index] = hit_point.x;
        data_buffer_0[data_index + 1u] = hit_point.y;
        data_buffer_0[data_index + 2u] = hit_point.z;
        data_buffer_0[data_index + 3u] = f32(result.visibility_id);
    }
    // Keep direction in buffer_1 for env map sampling and bounce brdf view evaluation
    data_buffer_1[data_index + 3u] = f32(pack2x16float(octahedral_mapping(direction)));
}
