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

// Ray Data (uses shadow_rays/shadow_intersections naming for compatibility with Rust bindings)
@group(3) @binding(0)
var<storage, read> shadow_rays: Rays;
@group(3) @binding(1)
var<storage, read_write> shadow_intersections: Intersections;

const WORKGROUP_SIZE: u32 = 64u;

// Optimized AO ray traversal - any-hit mode (similar to shadow rays)
fn traverse_bvh_ao(ray_origin: vec3<f32>, ray_direction: vec3<f32>, t_max: f32, tlas_starting_index: u32) -> bool {
    var stack: array<i32, 64>;
    var stack_ptr = 0;
    
    stack[stack_ptr] = i32(tlas_starting_index);
    stack_ptr++;

    var loop_count = 0u;

    while(stack_ptr > 0 && loop_count < 2048u) {
        loop_count++;
        stack_ptr--;
        let node_index = stack[stack_ptr];
        
        if(node_index < 0) { continue; }

        let node = bvh.data[u32(node_index)];
        let intersection = intersect_aabb(ray_origin, ray_direction, t_max, node.min, node.max);
        
        if(intersection < t_max) {
            if(node.reference < 0) {
                // Internal TLAS Node
                let left_index = node_index + 1;
                let left_node = bvh.data[u32(left_index)];
                let right_index = left_node.miss;

                if(stack_ptr < 62) {
                    if(right_index >= 0) {
                        stack[stack_ptr] = right_index;
                        stack_ptr++;
                    }
                    if(left_index >= 0) {
                        stack[stack_ptr] = left_index;
                        stack_ptr++;
                    }
                }
            } else {
                // Instance Hit - traverse BLAS
                let current_instance_id = u32(node.reference);
                let instance = instances.data[current_instance_id];
                let mesh = &meshes.data[instance.mesh_id];
                let transform = transforms.data[instance.transform_id];
                
                let position = transform.position_scale_x.xyz;
                let scale = vec3<f32>(transform.position_scale_x.w, transform.bb_min_scale_y.w, transform.bb_max_scale_z.w);
                let orientation = transform.orientation;
                let matrix = transform_matrix(position, orientation, scale);    
                let inverse_matrix = matrix_inverse(matrix);
                
                let local_ray_origin = (inverse_matrix * vec4<f32>(ray_origin, 1.)).xyz;
                let local_ray_direction = (inverse_matrix * vec4<f32>(ray_direction, 0.)).xyz;
                
                let bb_min = transform.bb_min_scale_y.xyz;
                let bb_max = transform.bb_max_scale_z.xyz;
                let hit_size = bb_max - bb_min;

                let blas_index = i32((*mesh).blas_index);

                // Simple BLAS traversal for AO - any hit
                if(traverse_blas_any_hit(
                    ray_origin, ray_direction,
                    local_ray_origin, local_ray_direction,
                    blas_index, t_max, current_instance_id,
                    hit_size, position, orientation, scale,
                    (*mesh).meshlets_offset, (*mesh).vertices_position_offset, bb_min
                )) {
                    return true; // Early exit - we hit something, AO is occluded
                }
            }
        }
    }
    return false; // No hit - AO ray is visible
}

// Simplified BLAS traversal for AO rays - any hit
fn traverse_blas_any_hit(
    world_ray_origin: vec3<f32>,
    world_ray_direction: vec3<f32>,
    local_ray_origin: vec3<f32>, 
    local_ray_direction: vec3<f32>, 
    blas_index: i32, 
    max_dist: f32,
    instance_id: u32,
    hit_size: vec3<f32>,
    position: vec3<f32>,
    orientation: vec4<f32>,
    scale: vec3<f32>,
    meshlets_offset: u32,
    vertices_position_offset: u32,
    bb_min: vec3<f32>
) -> bool {
    var stack: array<i32, 64>;
    var stack_ptr = 0;
    
    stack[stack_ptr] = blas_index;
    stack_ptr++;

    var loop_count = 0u;

    while(stack_ptr > 0 && loop_count < 2048u) {
        loop_count++;
        stack_ptr--;
        let node_index = stack[stack_ptr];
        
        let node = bvh.data[u32(node_index)];
        let aabb_min = bb_min + node.min * hit_size - vec3<f32>(0.01);
        let aabb_max = bb_min + node.max * hit_size + vec3<f32>(0.01);

        let intersection = intersect_aabb(local_ray_origin, local_ray_direction, max_dist, aabb_min, aabb_max);
        
        if(intersection < max_dist) {
            if(node.reference < 0) {
                let left_index = node_index + 1;
                let left_node = bvh.data[u32(left_index)];
                let right_index = left_node.miss;

                if(stack_ptr < 62) {
                    if(right_index >= 0) {
                        stack[stack_ptr] = right_index;
                        stack_ptr++;
                    }
                    if(left_index >= 0) {
                        stack[stack_ptr] = left_index;
                        stack_ptr++;
                    }
                }
            } else {
                let meshlet_id = meshlets_offset + u32(node.reference);
                let meshlet = meshlets.data[meshlet_id];
                let triangle_count = meshlet.indices_count / 3u;

                for(var i = 0u; i < triangle_count; i++) {
                    let index_offset = meshlet.indices_offset + (i * 3u);
                    
                    let p1 = bb_min + unpack_unorm_to_3_f32(vertices_positions.data[vertices_position_offset + indices.data[index_offset]]) * hit_size;
                    let p2 = bb_min + unpack_unorm_to_3_f32(vertices_positions.data[vertices_position_offset + indices.data[index_offset + 1u]]) * hit_size;
                    let p3 = bb_min + unpack_unorm_to_3_f32(vertices_positions.data[vertices_position_offset + indices.data[index_offset + 2u]]) * hit_size;

                    let v1 = transform_vector(p1, position, orientation, scale);
                    let v2 = transform_vector(p2, position, orientation, scale);
                    let v3 = transform_vector(p3, position, orientation, scale);
                    
                    let res = intersect_triangle(world_ray_origin, world_ray_direction, max_dist, v1, v2, v3);
                    if(res.t < max_dist) {
                        return true; // Any hit is sufficient for AO occlusion
                    }
                }
            }
        }
    }
    return false;
}

@compute
@workgroup_size(WORKGROUP_SIZE, 1, 1)
fn main(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let ray_index = global_invocation_id.x;
    if (ray_index >= arrayLength(&shadow_rays.data)) {
        return;
    }

    let ray = shadow_rays.data[ray_index];
    
    // Skip inactive or terminated rays
    if ((ray.flags & RAY_FLAG_ACTIVE) == 0u || (ray.flags & RAY_FLAG_TERMINATED) != 0u) {
        shadow_intersections.data[ray_index].instance_id = -1;
        shadow_intersections.data[ray_index].t = -1.0;
        return;
    }
    
    // Skip rays with invalid t_max
    if (ray.t_max < 0.0) {
        shadow_intersections.data[ray_index].instance_id = -1;
        shadow_intersections.data[ray_index].t = -1.0;
        return;
    }

    // Perform any-hit BVH traversal for AO ray
    let is_occluded = traverse_bvh_ao(ray.origin, ray.direction, ray.t_max, constant_data.tlas_starting_index);
    
    // Write result - for AO rays we only need binary occlusion
    if (is_occluded) {
        shadow_intersections.data[ray_index].instance_id = 1; // Occluded
        shadow_intersections.data[ray_index].t = 0.0;
    } else {
        shadow_intersections.data[ray_index].instance_id = -1; // Visible
        shadow_intersections.data[ray_index].t = -1.0;
    }
    shadow_intersections.data[ray_index].u = 0.0;
    shadow_intersections.data[ray_index].v = 0.0;
    shadow_intersections.data[ray_index].primitive_index = 0;
    shadow_intersections.data[ray_index].padding = 0u;
}
