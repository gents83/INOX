#import "common.inc"
#import "utils.inc"
#import "ray_data.inc"
#import "ray_types.inc"
#import "raytracing.inc"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;

// Geometry
@group(1) @binding(0)
var<storage, read> indices: Indices;
@group(1) @binding(1)
var<storage, read> vertices_positions: VerticesPositions;
@group(1) @binding(2)
var<storage, read> vertices_attributes: VerticesAttributes;
@group(1) @binding(3)
var<storage, read> instances: Instances;
@group(1) @binding(4)
var<storage, read> transforms: Transforms;
@group(1) @binding(5)
var<storage, read> meshes: Meshes;
@group(1) @binding(6)
var<storage, read> meshlets: Meshlets;

// BVH
@group(2) @binding(0)
var<storage, read> bvh: BVH;

// Ray Data
@group(3) @binding(0)
var<storage, read> rays: Rays;
@group(3) @binding(1)
var<storage, read_write> intersections: Intersections;

const WORKGROUP_SIZE: u32 = 64u;

@compute
@workgroup_size(WORKGROUP_SIZE, 1, 1)
fn main(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let ray_index = global_invocation_id.x;
    if (ray_index >= arrayLength(&rays.data)) {
        return;
    }

    let ray = rays.data[ray_index];
    
    // Skip inactive or terminated rays
    if ((ray.flags & RAY_FLAG_ACTIVE) == 0u || (ray.flags & RAY_FLAG_TERMINATED) != 0u) {
        intersections.data[ray_index].instance_id = -1;
        return;
    }
    
    // Skip rays with invalid t_max
    if (ray.t_max < 0.0) {
        intersections.data[ray_index].instance_id = -1;
        return;
    }

    // Perform BVH traversal (TLAS starts at index 0)
    let result = traverse_bvh(ray.origin, ray.direction, 0u);
    
    // Map Result to Intersection
    intersections.data[ray_index].t = result.distance;
    intersections.data[ray_index].u = result.u;
    intersections.data[ray_index].v = result.v;
    intersections.data[ray_index].instance_id = i32(result.instance_id);
    intersections.data[ray_index].primitive_index = i32(result.primitive_index);
    intersections.data[ray_index].padding = result.steps;
}
