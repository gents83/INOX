#import "common.inc"
#import "pathtracing_common.inc"

@group(0) @binding(0) var<uniform> constant_data: ConstantData;
@group(0) @binding(1) var<storage, read> indices: Indices;
@group(0) @binding(2) var<storage, read> vertices_positions: VerticesPositions;
@group(0) @binding(3) var<storage, read> vertices_attributes: VerticesAttributes;
@group(0) @binding(4) var<storage, read> instances: Instances;
@group(0) @binding(5) var<storage, read> transforms: Transforms;
@group(0) @binding(6) var<storage, read> meshes: Meshes;
@group(0) @binding(7) var<storage, read> meshlets: Meshlets;
@group(0) @binding(8) var<storage, read> bvh: BVH;

@group(1) @binding(0) var<storage, read_write> rays: array<Ray>;
@group(1) @binding(1) var<storage, read_write> hits: array<RayHit>;
@group(1) @binding(2) var<storage, read_write> counters: PathTracingCounters;

#import "utils.inc"
#import "matrix_utils.inc"
#import "geom_utils.inc"
#import "raytracing_optimized.inc"

const WORKGROUP_SIZE: u32 = 8u;

@compute
@workgroup_size(WORKGROUP_SIZE, WORKGROUP_SIZE, 1)
fn main(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let index = global_invocation_id.y * u32(constant_data.screen_width) + global_invocation_id.x;

    let ray = rays[index];
    if (ray.t_max <= 0.0) {
        return;
    }

    let tlas_index = constant_data.tlas_starting_index;
    let result = traverse_bvh_optimized(ray.origin, ray.direction, tlas_index);

    let hit_index = atomicAdd(&counters.hit_count, 1u);

    var hit: RayHit;
    hit.pixel_index = ray.pixel_index;
    hit.direction = ray.direction;
    hit.throughput = ray.throughput;

    if (result.distance < ray.t_max) {
        hit.instance_id = result.instance_id;
        hit.primitive_index = result.visibility_id & 255u;
        hit.barycentrics = result.barycentrics;
        hit.t = result.distance;
    } else {
        hit.instance_id = 0xFFFFFFFFu; // Sentinel for miss
        hit.t = -1.0;
    }

    hits[hit_index] = hit;
}
