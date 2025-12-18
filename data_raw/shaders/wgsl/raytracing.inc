#import "matrix_utils.inc"

const HIT_DATA_TLAS = 0u;
const HIT_DATA_BLAS = 1u;
const HIT_DATA_TRIANGLE = 2u;
const HIT_DATA_MISS = 4u;

struct Result {
    distance: f32,
    instance_id: u32,
    primitive_index: u32,
    u: f32,
    v: f32,
    steps: u32,
}

struct IntersectTriangleResult {
    t: f32,
    u: f32,
    v: f32,
}

fn intersect_aabb(origin: vec3<f32>, direction: vec3<f32>, max_distance: f32, aabb_min: vec3<f32>, aabb_max: vec3<f32>) -> f32 {
    let size = aabb_max - aabb_min;
    let inverse_dir = 1. / direction;
    let n = origin * inverse_dir;
    let k = abs(inverse_dir) * size;
    let t_min = -n - k;
    let t_max = -n + k;

    let t_near = max(max(t_min.x, t_min.y), t_min.z);
    let t_far = min(min(t_max.x, t_max.y), t_max.z);

    return select(t_far, t_near, (t_near < max_distance && t_far > 0.));
}

// Mäller-Trumbore algorithm
// https://en.wikipedia.org/wiki/Möller–Trumbore_intersection_algorithm
fn intersect_triangle(origin: vec3<f32>, direction: vec3<f32>, max_distance: f32, v0: vec3<f32>, v1: vec3<f32>, v2: vec3<f32>) -> IntersectTriangleResult
{
    var result = IntersectTriangleResult(max_distance, 0., 0.);
    let e1 = v1 - v0;
    let e2 = v2 - v0;
    let n = cross(e1, e2);
    // if (dot(n, direction) > 0.) {
    //    return result;
    // }

    let s = origin - v0;
    let q = cross(s, e1);
    let h = cross(direction, e2);
    let det = dot(e1, h);

    let inv_det = 1. / det;
    let u = inv_det * dot(s, h);
    let v = inv_det * dot(direction, q);
    let t = inv_det * dot(e2, q);
    
    if (t > MATH_EPSILON && t < max_distance && u >= 0. && u <= 1. && v >= 0. && u + v <= 1.) {
        result.t = t;
        result.u = u;
        result.v = v;
    }
    return result;
}

//starting index = tlas_starting_index
//starting index = tlas_starting_index
// traverse_blas removed as it was replaced by traverse_blas_stack

// Rewriting above because I need World Ray for triangle intersection.

fn traverse_blas_stack(
    world_ray_origin: vec3<f32>,
    world_ray_direction: vec3<f32>,
    local_ray_origin: vec3<f32>, 
    local_ray_direction: vec3<f32>, 
    blas_index: i32, 
    min_dist: f32,
    instance_id: u32,
    hit_size: vec3<f32>,
    position: vec3<f32>,
    orientation: vec4<f32>,
    scale: vec3<f32>,
    meshlets_offset: u32,
    vertices_position_offset: u32,
    bb_min: vec3<f32>
) -> Result {
    var max_distance = min_dist;
    var primitive_index = 0u;
    var result_instance_id = 0u;
    var best_u = 0.;
    var best_v = 0.;

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
        // Use LOCAL RAY for AABB
        let aabb_min = bb_min + node.min * hit_size - vec3<f32>(0.01);
        let aabb_max = bb_min + node.max * hit_size + vec3<f32>(0.01);

        let intersection = intersect_aabb(local_ray_origin, local_ray_direction, max_distance, aabb_min, aabb_max);
        
        if(intersection < max_distance) {
            if(node.reference < 0) {
                let left_index = node_index + 1;
                let left_node = bvh.data[u32(left_index)];
                let right_index = left_node.miss;

                // Stack Check
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
                    
                    // Use WORLD RAY for Triangle
                    let res = intersect_triangle(world_ray_origin, world_ray_direction, max_distance, v1, v2, v3);
                    if(res.t < max_distance) {
                        primitive_index = meshlet.indices_offset / 3u + i;
                        result_instance_id = instance_id;
                        max_distance = res.t;
                        best_u = res.u;
                        best_v = res.v;
                    }
                }
            }
        }
    }
    return Result(max_distance, result_instance_id, primitive_index, best_u, best_v, loop_count);
}


fn traverse_bvh(world_ray_origin: vec3<f32>, world_ray_direction: vec3<f32>, tlas_starting_index: u32) -> Result {
    var max_distance = MAX_TRACING_DISTANCE;
    var primitive_index = 0u;
    var instance_id = 0u;
    var u_coord = 0.;
    var v_coord = 0.;

    var stack: array<i32, 64>;
    var stack_ptr = 0;
    
    stack[stack_ptr] = i32(tlas_starting_index);
    stack_ptr++;

    var loop_count = 0u;

    while(stack_ptr > 0 && loop_count < 2048u) {
        loop_count++;
        stack_ptr--;
        let node_index = stack[stack_ptr];
        
        // Skip invalid nodes
        if(node_index < 0) { continue; }

        let node = bvh.data[u32(node_index)];
        let intersection = intersect_aabb(world_ray_origin, world_ray_direction, max_distance, node.min, node.max);
        
        if(intersection < max_distance) {
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
                // Instance Hit
                let current_instance_id = u32(node.reference);
                let instance = instances.data[current_instance_id];
                let mesh = &meshes.data[instance.mesh_id];
                let transform = transforms.data[instance.transform_id];
                
                let position = transform.position_scale_x.xyz;
                let scale = vec3<f32>(transform.position_scale_x.w, transform.bb_min_scale_y.w, transform.bb_max_scale_z.w);
                let orientation = transform.orientation;
                let matrix = transform_matrix(position, orientation, scale);    
                let inverse_matrix = matrix_inverse(matrix);
                
                let local_ray_origin = (inverse_matrix * vec4<f32>(world_ray_origin, 1.)).xyz;
                let local_ray_direction = (inverse_matrix * vec4<f32>(world_ray_direction, 0.)).xyz;
                
                let bb_min = transform.bb_min_scale_y.xyz;
                let bb_max = transform.bb_max_scale_z.xyz;
                let hit_size = bb_max - bb_min;

                let blas_index = i32((*mesh).blas_index);

                let blas_result = traverse_blas_stack(
                    world_ray_origin, world_ray_direction,
                    local_ray_origin, local_ray_direction,
                    blas_index, max_distance, current_instance_id,
                    hit_size, position, orientation, scale,
                    (*mesh).meshlets_offset, (*mesh).vertices_position_offset, bb_min
                );

                // Add BLAS steps to total
                loop_count += blas_result.steps;

                if(blas_result.distance < max_distance) {
                    max_distance = blas_result.distance;
                    primitive_index = blas_result.primitive_index;
                    instance_id = blas_result.instance_id;
                    u_coord = blas_result.u;
                    v_coord = blas_result.v;
                }
            }
        }
    }
    return Result(max_distance, instance_id, primitive_index, u_coord, v_coord, loop_count);
}
