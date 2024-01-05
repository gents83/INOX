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

    return select(t_far, t_near, (t_near < (*ray).t_max && t_far > 0.));
}

// Mäller-Trumbore algorithm
// https://en.wikipedia.org/wiki/Möller–Trumbore_intersection_algorithm
fn intersect_triangle(ray: ptr<function, Ray>, v0: vec3<f32>, v1: vec3<f32>, v2: vec3<f32>) -> f32
{
    let o = (*ray).origin;
    let dir = (*ray).direction;
    let miss = (*ray).t_max;
    var result = miss;

    let e1 = v1 - v0;
    let e2 = v2 - v0;

    let h = cross(dir, e2);
    let det = dot(e1, h);

    let inv_det = 1. / det;
    let s = o - v0;
    let u = inv_det * dot(s, h);

    let q = cross(s, e1);
    let v = inv_det * dot(dir, q);

    let t = inv_det * dot(e2, q);
    result = select(result, t, (t > MATH_EPSILON && t < result));
    result = select(result, miss, u < 0. || u > 1.);
    result = select(result, miss, v < 0. || u + v > 1.);
    return result;
}

//starting index = tlas_starting_index
fn traverse_bvh(world_ray: Ray, tlas_starting_index: u32) -> Result {
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
    
    var node = bhv.data[u32(node_index)]; 
    while(node_index >= 0)
    {
        node = bhv.data[u32(node_index)]; 
        if(node.reference < 0) {
            node_index = node_index + 1;
            continue;
        }
        let intersection = intersect_aabb(&ray, node.min, node.max);        
        if(intersection < ray.t_max) {
            //leaf node
            if(hit_type == HIT_DATA_TRIANGLE) {     
                let triangle_id = u32(node.reference);
                let index_offset = meshlet_indices_offset + (triangle_id * 3u);
                let v1 = runtime_vertices.data[vertices_position_offset + indices.data[index_offset]].world_pos;
                let v2 = runtime_vertices.data[vertices_position_offset + indices.data[index_offset + 1u]].world_pos;
                let v3 = runtime_vertices.data[vertices_position_offset + indices.data[index_offset + 2u]].world_pos;
                ray.origin = world_ray.origin;
                ray.direction = world_ray.direction;
                let distance = intersect_triangle(&ray, v1, v2, v3);
                if(distance < ray.t_max) {
                    visibility_id = ((meshlet_id + 1u) << 8u)  | triangle_id;
                    ray.t_max = distance;
                }
                node_index = node.miss;
                let should_go_to_blas = node_index < 0;
                hit_type = select(hit_type, HIT_DATA_BLAS, should_go_to_blas);
                node_index = select(node_index, blas_sibling, should_go_to_blas);
                let should_go_to_tlas = node_index < 0;
                hit_type = select(hit_type, HIT_DATA_TLAS, should_go_to_tlas);
                node_index = select(node_index, tlas_sibling, should_go_to_tlas);
                ray.origin = select(ray.origin, world_ray.origin, should_go_to_tlas);
                ray.direction = select(ray.direction, world_ray.direction, should_go_to_tlas);
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
            let should_go_to_blas = hit_type == HIT_DATA_TRIANGLE && node_index < 0;
            hit_type = select(hit_type, HIT_DATA_BLAS, should_go_to_blas);
            node_index = select(node_index, blas_sibling, should_go_to_blas);
            let should_go_to_tlas = hit_type == HIT_DATA_BLAS && node_index < 0;
            hit_type = select(hit_type, HIT_DATA_TLAS, should_go_to_tlas);
            node_index = select(node_index, tlas_sibling, should_go_to_tlas);
            ray.origin = select(ray.origin, world_ray.origin, hit_type == HIT_DATA_TLAS);
            ray.direction = select(ray.direction, world_ray.direction, hit_type == HIT_DATA_TLAS);
        }         
    }
    return Result(ray.t_max, visibility_id);
}
