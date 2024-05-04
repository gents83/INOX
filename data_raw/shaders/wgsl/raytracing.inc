const HIT_DATA_TLAS = 0u;
const HIT_DATA_BLAS = 1u;
const HIT_DATA_TRIANGLE = 2u;
const HIT_DATA_MISS = 4u;

struct Result {
    distance: f32,
    visibility_id: u32,
    instance_id: u32,
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
fn intersect_triangle(origin: vec3<f32>, direction: vec3<f32>, max_distance: f32, v0: vec3<f32>, v1: vec3<f32>, v2: vec3<f32>) -> f32
{
    var result = max_distance;
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
    
    result = select(result, t, (t > MATH_EPSILON && t < result));
    result = select(result, max_distance, u < 0. || u > 1.);
    result = select(result, max_distance, v < 0. || u + v > 1.);
    return result;
}

//starting index = tlas_starting_index
fn traverse_bvh(world_ray_origin: vec3<f32>, world_ray_direction: vec3<f32>, tlas_starting_index: u32) -> Result {
    var origin = world_ray_origin;
    var direction = world_ray_direction;
    var max_distance = MAX_TRACING_DISTANCE;
    var node_index = i32(tlas_starting_index) + 1;
    var visibility_id = 0u;
    var instance_id = 0u;
    var hit_type = HIT_DATA_TLAS;
    var meshlets_offset = 0u;
    var vertices_position_offset = 0u;
    var tlas_sibling = 0;
    
    var node = bhv.data[u32(node_index)]; 
    while(node_index >= 0)
    {
        node = bhv.data[u32(node_index)]; 
        if(node.reference < 0) {
            node_index = node_index + 1;
            continue;
        }
        let intersection = intersect_aabb(origin, direction, max_distance, node.min, node.max);        
        if(intersection < max_distance) {
            let reference = u32(node.reference);  
            //leaf node
            if(hit_type == HIT_DATA_BLAS) {  
                let meshlet_id = meshlets_offset + reference;              
                let meshlet = meshlets.data[meshlet_id];
                let triangle_count = meshlet.indices_count / 3u;
                for(var i = 0u; i < triangle_count; i++)
                {
                    let index_offset = meshlet.indices_offset + (i * 3u);
                    //TODO: use instances and convert to real positions like:
                    //let size = instance.mesh_local_bb_max - instance.mesh_local_bb_min;
                    //let p = instance.mesh_local_bb_min + unpack_unorm_to_3_f32(vertex_position) * size;
                    //let v = transform_vector(p, instance.position, instance.orientation, instance.scale);

                    let v1 = unpack_unorm_to_3_f32(vertices_positions.data[vertices_position_offset + indices.data[index_offset]]);
                    let v2 = unpack_unorm_to_3_f32(vertices_positions.data[vertices_position_offset + indices.data[index_offset + 1u]]);
                    let v3 = unpack_unorm_to_3_f32(vertices_positions.data[vertices_position_offset + indices.data[index_offset + 2u]]);
                    let distance = intersect_triangle(world_ray_origin, world_ray_direction, max_distance, v1, v2, v3);
                    if(distance < max_distance) {
                        visibility_id = ((meshlet_id + 1u) << 8u)  | reference;
                        max_distance = distance;
                    }
                }
                node_index = node.miss;
                let should_go_to_tlas = node_index < 0;
                hit_type = select(hit_type, HIT_DATA_TLAS, should_go_to_tlas);
                node_index = select(node_index, tlas_sibling, should_go_to_tlas);
                origin = select(origin, world_ray_origin, should_go_to_tlas);
                direction = select(direction, world_ray_direction, should_go_to_tlas);
            }
            else {  
                hit_type = HIT_DATA_BLAS;
                tlas_sibling = node.miss;
                let mesh = &meshes.data[reference];  
                meshlets_offset = (*mesh).meshlets_offset;
                vertices_position_offset = (*mesh).vertices_position_offset;
                node_index = i32((*mesh).blas_index) + 1;  //We add 1 here to avoid checking same aabb 

                //TODO: use instances to retrieve matrix
                //let matrix = transform_matrix((*mesh).position, (*mesh).orientation, (*mesh).scale);    
                //let inverse_matrix = matrix_inverse(matrix);
                //let local_ray_origin = inverse_matrix * vec4<f32>(world_ray_origin, 1.);
                //let local_ray_direction = inverse_matrix * vec4<f32>(world_ray_direction, 0.);
                //origin = local_ray_origin.xyz;
                //direction = local_ray_direction.xyz;
            }
        }
        else {
            node_index = node.miss;
            let should_go_to_tlas = hit_type == HIT_DATA_BLAS && node_index < 0;
            hit_type = select(hit_type, HIT_DATA_TLAS, should_go_to_tlas);
            node_index = select(node_index, tlas_sibling, should_go_to_tlas);
            origin = select(origin, world_ray_origin, hit_type == HIT_DATA_TLAS);
            direction = select(direction, world_ray_direction, hit_type == HIT_DATA_TLAS);
        }         
    }
    return Result(max_distance, visibility_id, instance_id);
}
