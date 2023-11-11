const HIT_EPSILON: f32 = 0.0001;
const INVALID_NODE: i32 = -1;

struct Result {
    distance: f32,
    visibility_id: u32,
}

fn intersect_aabb(ray: ptr<function, Ray>, aabb_min: vec3<f32>, aabb_max: vec3<f32>) -> f32 
{     
    let r_min = (*ray).t_min;
    let r_max = (*ray).t_max;
    let inverse_dir = 1. / (*ray).direction;
    let v_min = (aabb_min - (*ray).origin) * inverse_dir;
    let v_max = (aabb_max - (*ray).origin) * inverse_dir;

    let t_min = min(v_min, v_max);
    let t_max = max(v_min, v_max);

    let t_near = max(max(t_min.x, t_min.y), max(t_min.x, t_min.z));
    let t_far = min(min(t_max.x, t_max.y), min(t_max.x, t_max.z));
    
    var r = select(r_max, t_far, t_far <= r_max);
    r = select(r_max, t_near, t_near >= r_min);
    r = select(r, r_max, t_near > t_far || t_far < 0.);
    return r; 
}

fn intersect_triangle(ray: ptr<function, Ray>, v0: vec3<f32>, v1: vec3<f32>, v2: vec3<f32>) -> f32
{
    let e1 = v1 - v0;
    let e2 = v2 - v0;
    let far = (*ray).t_max;

    let p = cross((*ray).direction, e2);
    let det = dot(e1, p);
    
    if (abs(det) < HIT_EPSILON) { return far; }

    // Computes Barycentric coordinates.
    let inv_det = 1. / det;
    let t1 = (*ray).origin - v0;    
    let u = dot(t1, p) * inv_det;
    if (u < 0. || u > 1.) { return far; }
    
    let q = cross(t1, e1);
    let v = dot((*ray).direction, q) * inv_det;
    if (v < 0. || u + v > 1.) { return far; }

    var t2 = dot(e2, q) * inv_det;
    t2 = select(t2, far, t2 < 0.);
    return t2;
}

fn intersect_meshlet_primitive(ray: ptr<function, Ray>, position_offset: u32, meshlet_id: u32, index_offset: u32) -> f32 {
    let vert_indices = vec3<u32>(indices.data[index_offset], indices.data[index_offset + 1u], indices.data[index_offset + 2u]);
    let pos_indices = vert_indices + vec3<u32>(position_offset, position_offset, position_offset);
    
    let v1 = runtime_vertices.data[pos_indices.x].world_pos;
    let v2 = runtime_vertices.data[pos_indices.y].world_pos;
    let v3 = runtime_vertices.data[pos_indices.z].world_pos;
    
    return intersect_triangle(ray, v1, v2, v3);
}

fn intersect_meshlet(ray: ptr<function, Ray>, position_offset: u32, meshlet_id: u32, far_plane: f32) -> Result {
    var nearest = far_plane;  
    var visibility_id = 0u;
      
    let meshlet = &meshlets.data[meshlet_id];
    let primitive_count = (*meshlet).indices_count / 3u;
    var index_offset = (*meshlet).indices_offset;
    for(var primitive_id = 0u; primitive_id < primitive_count; primitive_id = primitive_id + 1u)
    {       
        let hit = intersect_meshlet_primitive(ray, position_offset, meshlet_id, index_offset);
        visibility_id = select(visibility_id, ((meshlet_id + 1u) << 8u) | primitive_id, hit < nearest);
        nearest = min(nearest, hit);
        index_offset += 3u;
    }
    return Result(nearest, visibility_id);
}

fn traverse_bhv_of_meshlets(world_ray: ptr<function, Ray>, local_ray: ptr<function, Ray>, mesh_id: u32, far_plane: f32) -> Result {
    let mesh = &meshes.data[mesh_id];    
    let position_offset = (*mesh).vertices_position_offset;
    var blas_index = i32((*mesh).blas_index);    
    let mesh_blas_index = blas_index;
    var nearest = far_plane;  
    var visibility_id = 0u;

    while (blas_index >= 0)
    { 
        let node = &bhv.data[u32(blas_index)];   
        let intersection = intersect_aabb(local_ray, (*node).min, (*node).max);
        if (intersection >= nearest) {
            blas_index = select((*node).miss, (*node).miss + mesh_blas_index, (*node).miss >= 0);
            continue;
        }
        if ((*node).reference < 0) {
            //inner node
            blas_index = blas_index + 1;
            continue;  
        }
        //leaf node
        let meshlet_id = (*mesh).meshlets_offset + u32((*node).reference);   
        
        let index = meshlet_id / 32u;
        let offset = meshlet_id - (index * 32u);
        let bits = culling_result[index];
        let is_meshlet_visible =  (bits & (1u << offset)) > 0u;   

        if (!is_meshlet_visible) {
            blas_index = select((*node).miss, (*node).miss + mesh_blas_index, (*node).miss >= 0);
            continue;
        }
        let hit = intersect_meshlet(world_ray, position_offset, meshlet_id, nearest);
        visibility_id = select(visibility_id, hit.visibility_id, hit.distance < nearest);
        nearest = min(nearest, hit.distance);
        blas_index = select((*node).miss, (*node).miss + mesh_blas_index, (*node).miss >= 0);
    }
    return Result(nearest, visibility_id);
}


