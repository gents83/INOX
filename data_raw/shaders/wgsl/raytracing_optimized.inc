const HIT_DATA_TLAS = 0u;
const HIT_DATA_BLAS = 1u;
const HIT_DATA_TRIANGLE = 2u;
const HIT_DATA_MISS = 4u;

struct Result {
    distance: f32,
    visibility_id: u32,
    instance_id: u32,
    steps: u32,
    barycentrics: vec2<f32>,
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
// Returns vec3(t, u, v)
fn intersect_triangle_barycentrics(origin: vec3<f32>, direction: vec3<f32>, max_distance: f32, v0: vec3<f32>, v1: vec3<f32>, v2: vec3<f32>) -> vec3<f32>
{
    var result = vec3<f32>(max_distance, 0., 0.);
    let e1 = v1 - v0;
    let e2 = v2 - v0;
    let n = cross(e1, e2);
    if (dot(n, direction) > 0.) {
        return result;
    }

    let s = origin - v0;
    let q = cross(s, e1);
    let h = cross(direction, e2);
    let det = dot(e1, h);

    let inv_det = 1. / det;
    let u = inv_det * dot(s, h);
    let v = inv_det * dot(direction, q);
    let t = inv_det * dot(e2, q);

    if (t > MATH_EPSILON && t < max_distance && u >= 0. && u <= 1. && v >= 0. && u + v <= 1.) {
        result = vec3<f32>(t, u, v);
    }
    return result;
}

fn traverse_blas_local(
    local_ray_origin: vec3<f32>,
    local_ray_direction: vec3<f32>,
    blas_index: i32,
    min_dist: f32,
    instance_id: u32,
    hit_size: vec3<f32>,
    meshlets_offset: u32,
    vertices_position_offset: u32,
    bb_min: vec3<f32>
) -> Result {
    var max_distance = min_dist;
    var visibility_id = 0u;
    var result_instance_id = 0u;
    var barycentrics = vec2<f32>(0.);

    var stack: array<i32, 64>;
    var stack_ptr = 0;

    stack[stack_ptr] = blas_index;
    stack_ptr++;

    var loop_count = 0u;

    while(stack_ptr > 0 && loop_count < 256u) {
        loop_count++;
        stack_ptr--;
        let node_index = stack[stack_ptr];

        let node = bvh.data[u32(node_index)];
        // Use LOCAL RAY for AABB
        let intersection = intersect_aabb(local_ray_origin, local_ray_direction, max_distance, node.min, node.max);

        if(intersection < max_distance) {
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

                    // Use LOCAL RAY for Triangle (Optimized)
                    let hit = intersect_triangle_barycentrics(local_ray_origin, local_ray_direction, max_distance, p1, p2, p3);

                    if(hit.x < max_distance) {
                        max_distance = hit.x;
                        visibility_id = ((instance_id + 1u) << 8u) | i;
                        result_instance_id = instance_id;
                        barycentrics = hit.yz;
                    }
                }
            }
        }
    }
    return Result(max_distance, visibility_id, result_instance_id, loop_count, barycentrics);
}


fn traverse_bvh_optimized(world_ray_origin: vec3<f32>, world_ray_direction: vec3<f32>, tlas_starting_index: u32) -> Result {
    var max_distance = MAX_TRACING_DISTANCE;
    var visibility_id = 0u;
    var instance_id = 0u;
    var barycentrics = vec2<f32>(0.);

    var stack: array<i32, 64>;
    var stack_ptr = 0;

    stack[stack_ptr] = i32(tlas_starting_index);
    stack_ptr++;

    var loop_count = 0u;

    while(stack_ptr > 0 && loop_count < 256u) {
        loop_count++;
        stack_ptr--;
        let node_index = stack[stack_ptr];

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

                let blas_result = traverse_blas_local(
                    local_ray_origin, local_ray_direction,
                    blas_index, max_distance, current_instance_id,
                    hit_size,
                    (*mesh).meshlets_offset, (*mesh).vertices_position_offset, bb_min
                );

                loop_count += blas_result.steps;

                if(blas_result.distance < max_distance) {
                    max_distance = blas_result.distance;
                    visibility_id = blas_result.visibility_id;
                    instance_id = blas_result.instance_id;
                    barycentrics = blas_result.barycentrics;
                }
            }
        }
    }
    return Result(max_distance, visibility_id, instance_id, loop_count, barycentrics);
}
