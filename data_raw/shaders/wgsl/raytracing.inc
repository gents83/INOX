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

    let b_min = min(v_min, v_max);
    let b_max = max(v_min, v_max);

    let t_near = max(max(b_min.x, b_min.y), max(b_min.x, b_min.z));
    let t_far = min(min(b_max.x, b_max.y), min(b_max.x, b_max.z));
    
    let r = select(t_near, r_max, t_near > t_far || t_far < 0.);
    return r; 
}

fn intersect_triangle(ray: ptr<function, Ray>, v0: vec3<f32>, v1: vec3<f32>, v2: vec3<f32>) -> f32
{
    let origin = (*ray).origin;
    let dir = (*ray).direction;
    let far = (*ray).t_max;

    let e1 = v1 - v0;
    let e2 = v2 - v0;

    let p = cross(dir, e2);
    let det = dot(e1, p);
    
    if (abs(det) < HIT_EPSILON) { return far; }

    // Computes Barycentric coordinates.
    let inv_det = 1. / det;
    let t1 = origin - v0;    
    let u = dot(t1, p) * inv_det;
    if (u < 0. || u > 1.) { return far; }
    
    let q = cross(t1, e1);
    let v = dot(dir, q) * inv_det;
    if (v < 0. || u + v > 1.) { return far; }

    let t2 = dot(e2, q) * inv_det;
    return select(t2, far, t2 < 0.);
}

fn intersect_meshlet_primitive(ray: ptr<function, Ray>, position_offset: u32, meshlet_id: u32, index_offset: u32) -> f32 {
    let vert_indices = vec3<u32>(indices.data[index_offset], indices.data[index_offset + 1u], indices.data[index_offset + 2u]);
    let pos_indices = vert_indices + vec3<u32>(position_offset, position_offset, position_offset);
        
    return intersect_triangle(ray, runtime_vertices.data[pos_indices.x].world_pos, runtime_vertices.data[pos_indices.y].world_pos, runtime_vertices.data[pos_indices.z].world_pos);
}

fn intersect_meshlet(ray: ptr<function, Ray>, position_offset: u32, meshlet_id: u32, far_plane: f32) -> Result {
    var nearest = far_plane;  
    var visibility_id = 0u;
      
    let meshlet = &meshlets.data[meshlet_id];
    let index_count = (*meshlet).indices_count;
    let index_offset = (*meshlet).indices_offset;
    for(var index = 0u; index < index_count; index = index + 3u)
    {       
        let hit = intersect_meshlet_primitive(ray, position_offset, meshlet_id, index_offset + index);
        visibility_id = select(visibility_id, ((meshlet_id + 1u) << 8u) | (index / 3u), hit < nearest);
        nearest = min(nearest, hit);
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
        if (intersection > nearest) {
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


