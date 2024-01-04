const HIT_DATA_TLAS = 0u;
const HIT_DATA_BLAS = 1u;
const HIT_DATA_TRIANGLE = 2u;
const HIT_DATA_MISS = 4u;

struct Result {
    distance: f32,
    visibility_id: u32,
}

fn intersect_aabb(ray: ptr<function, Ray>, aabb_min: vec3<f32>, aabb_max: vec3<f32>) -> f32 {
    let size = aabb_max - aabb_min;
    let inverse_dir = 1. / (*ray).direction;
    let n = (*ray).origin * inverse_dir;
    let k = abs(inverse_dir) * size;
    let t_min = -n - k;
    let t_max = -n + k;

    let t_near = max(max(t_min.x, t_min.y), t_min.z);
    let t_far = min(min(t_max.x, t_max.y), t_max.z);

    if t_far >= 0. {
        return t_near;
    }
    return t_far;
}

fn intersect_triangle(ray: Ray, v0: vec3<f32>, v1: vec3<f32>, v2: vec3<f32>) -> f32
{
    let e1 = v1 - v0;
    let e2 = v2 - v0;

    let h = cross(ray.direction, e2);
    let a = dot(e1, h);    
    let inv_a = 1.0 / a;    
    let s = ray.origin - v0;    
    let u = inv_a * dot(s, h);    
    let mask_u = u >= 0.0 && u <= 1.0;
    let q = cross(s, e1);    
    let v = inv_a * dot(ray.direction, q);    
    let mask_v = v >= 0.0 && u + v <= 1.0;
    let t = inv_a * dot(e2, q);    
    let mask_t = t > 0.0 && t < ray.t_max;

    return select(ray.t_max, t, (mask_u && mask_v && mask_t));
}

//starting index = tlas_starting_index
fn traverse_bvh(world_ray: Ray, tlas_starting_index: u32, is_pixel_to_debug: bool, debug_starting_index: u32) -> Result {
    var ray = world_ray;
    var node_index = i32(tlas_starting_index) + 1;
    var visibility_id = 0u;
    var hit_type = HIT_DATA_TLAS;
    var meshlets_offset = 0u;
    var meshlet_indices_offset = 0u;
    var vertices_position_offset = 0u;
    var meshlet_id = 0u;
    var blas_sibling = 0;
    var tlas_sibling = 0;
    var debug_index = 1u + debug_starting_index;

    while(node_index >= 0)
    {
        let node = bhv.data[u32(node_index)]; 
        let intersection = intersect_aabb(&ray, node.min, node.max);        
        if(intersection < ray.t_max) {
            if (is_pixel_to_debug) {
                debug_index = write_vec3_on_debug_data_texture(debug_index, node.min);
                debug_index = write_vec3_on_debug_data_texture(debug_index, node.max);
            }
            if(node.reference < 0) {
                node_index = node_index + 1;
                continue;
            }
            //leaf node
            if(hit_type == HIT_DATA_TRIANGLE) {     
                let triangle_id = u32(node.reference);
                let index_offset = meshlet_indices_offset + (triangle_id * 3u);
                let v1 = runtime_vertices.data[vertices_position_offset + indices.data[index_offset]].world_pos;
                let v2 = runtime_vertices.data[vertices_position_offset + indices.data[index_offset + 1u]].world_pos;
                let v3 = runtime_vertices.data[vertices_position_offset + indices.data[index_offset + 2u]].world_pos;
                let distance = intersect_triangle(world_ray, v1, v2, v3);
                if(distance < ray.t_max) {
                    visibility_id = ((meshlet_id + 1u) << 8u)  | triangle_id;
                    ray.t_max = distance;
                }
                node_index = node.miss;
                if(node_index < 0) {
                    node_index = blas_sibling;
                    hit_type = HIT_DATA_BLAS;
                    if(node_index < 0) {
                        node_index = tlas_sibling;
                        hit_type = HIT_DATA_TLAS;
                        ray.origin = world_ray.origin;
                        ray.direction = world_ray.direction;
                    }
                }     
            }
            else if(hit_type == HIT_DATA_BLAS) {  
                hit_type = HIT_DATA_TRIANGLE;
                blas_sibling = node.miss;
                meshlet_id = meshlets_offset + u32(node.reference);  
            
                let meshlet = meshlets.data[meshlet_id];
                meshlet_indices_offset = meshlet.indices_offset;
                node_index = i32(meshlet.triangles_bhv_index) + 1; //We add 1 here to avoid checking same aabb 
            }
            else {  
                hit_type = HIT_DATA_BLAS;
                tlas_sibling = node.miss;
                let mesh_id = u32(node.reference);
                let mesh = meshes.data[mesh_id];  
                meshlets_offset = mesh.meshlets_offset;
                vertices_position_offset = mesh.vertices_position_offset;
                node_index = i32(mesh.blas_index) + 1;  //We add 1 here to avoid checking same aabb 

                let matrix = transform_matrix(mesh.position, mesh.orientation, mesh.scale);    
                let inverse_matrix = matrix_inverse(matrix);
                let local_ray_origin = inverse_matrix * vec4<f32>(world_ray.origin, 1.);
                let local_ray_direction = inverse_matrix * vec4<f32>(world_ray.direction, 0.);
                ray.origin = local_ray_origin.xyz;
                ray.direction = local_ray_direction.xyz;
            }
        }
        else {
            node_index = node.miss;
            if(hit_type == HIT_DATA_TRIANGLE && node_index < 0) {
                node_index = blas_sibling;
                hit_type = HIT_DATA_BLAS;
            }
            if(hit_type == HIT_DATA_BLAS && node_index < 0) {
                node_index = tlas_sibling;
                hit_type = HIT_DATA_TLAS;
                ray.origin = world_ray.origin;
                ray.direction = world_ray.direction;
            }
        }         
    }
    if (is_pixel_to_debug) {
        write_value_on_debug_data_texture(debug_starting_index, f32(debug_index));
    }
    return Result(ray.t_max, visibility_id);
}
