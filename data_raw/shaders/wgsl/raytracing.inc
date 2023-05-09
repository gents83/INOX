const HIT_EPSILON: f32 = 0.0001;
const INVALID_NODE: i32 = -1;

struct Result {
    distance: f32,
    visibility_id: u32,
}


fn intersect_aabb(ray: ptr<function, Ray>, aabb_min: vec3<f32>, aabb_max: vec3<f32>) -> f32 
{     
    let inverse_dir = 1. / (*ray).direction;
    let v_min = (aabb_min - (*ray).origin) * inverse_dir;
    let v_max = (aabb_max - (*ray).origin) * inverse_dir;

    let t_min = min(v_min, v_max);
    let t_max = max(v_min, v_max);

    let t_near = max(max(t_min.x, t_min.y), max(t_min.x, t_min.z));
    let t_far = min(min(t_max.x, t_max.y), min(t_max.x, t_max.z));

    if (t_near > t_far || t_far < 0.) { 
        return MAX_FLOAT; 
    }
    if (t_near > (*ray).t_min) {
        (*ray).t_max = t_far;
        return t_near;
    } else if (t_far < (*ray).t_max) {
        (*ray).t_max = t_far;
        return t_far;
    } 
    return MAX_FLOAT; 
}


fn intersect_triangle(ray: ptr<function, Ray>, v0: vec3<f32>, v1: vec3<f32>, v2: vec3<f32>) -> f32
{
    let e1 = v1 - v0;
    let e2 = v2 - v0;

    let p = cross((*ray).direction, e2);
    let det = dot(e1, p);
    
    if (abs(det) < HIT_EPSILON) { return MAX_FLOAT; }

    // Computes Barycentric coordinates.
    let inv_det = 1. / det;
    let t1 = (*ray).origin - v0;    
    let u = dot(t1, p) * inv_det;
    if (u < 0. || u > 1.) { return MAX_FLOAT; }
    
    let q = cross(t1, e1);
    let v = dot((*ray).direction, q) * inv_det;
    if (v < 0. || u + v > 1.) { return MAX_FLOAT; }

    let t2 = dot(e2, q) * inv_det;
    if (t2 < 0.) { return MAX_FLOAT; }
    return t2;
}

fn intersect_meshlet_primitive(ray: ptr<function, Ray>, mesh_id: u32, meshlet_id: u32, primitive_id: u32) -> f32 {
    let mesh = &meshes.data[mesh_id];    
    let node = &bhv.data[u32((*mesh).blas_index)];  
    
    let meshlet = &meshlets.data[meshlet_id];
    
    let index_offset = (*meshlet).indices_offset + (primitive_id * 3u);
    let i1 = indices.data[index_offset];
    let i2 = indices.data[index_offset + 1u];
    let i3 = indices.data[index_offset + 2u];

    let o1 = vertices.data[(*mesh).vertices_position_offset + i1].position_and_color_offset;
    let o2 = vertices.data[(*mesh).vertices_position_offset + i2].position_and_color_offset;
    let o3 = vertices.data[(*mesh).vertices_position_offset + i3].position_and_color_offset;

    let v1 = positions.data[o1];
    let v2 = positions.data[o2];
    let v3 = positions.data[o3];
    
    let oobb_size = (*node).max - (*node).min;
    
    let p1 = (*node).min + decode_as_vec3(v1) * oobb_size;
    let p2 = (*node).min + decode_as_vec3(v2) * oobb_size;
    let p3 = (*node).min + decode_as_vec3(v3) * oobb_size;
    
    return intersect_triangle(ray, p1, p2, p3);
}

fn is_meshlet_visible(meshlet_id: u32) -> bool
{
    let bits = atomicLoad(&culling_result[meshlet_id / 32u]);
    return (bits & ((1u << meshlet_id) % 32u)) > 0u;
}

fn intersect_meshlet(ray: ptr<function, Ray>, mesh_id: u32, meshlet_id: u32, far_plane: f32) -> Result {
    var nearest = far_plane;  
    var visibility_id = 0u;
      
    let meshlet = &meshlets.data[meshlet_id];
    let primitive_count = (*meshlet).indices_count / 3u;
    for(var primitive_id = 0u; primitive_id < primitive_count; primitive_id = primitive_id + 1u)
    {       
        let hit = intersect_meshlet_primitive(ray, mesh_id, meshlet_id, primitive_id);
        visibility_id = select(visibility_id, ((meshlet_id + 1u) << 8u) | primitive_id, hit < nearest);
        nearest = min(nearest, hit);
    }
    return Result(nearest, visibility_id);
}

fn traverse_bhv_of_meshlets(ray: ptr<function, Ray>, mesh_id: u32, far_plane: f32) -> Result {
    let mesh = &meshes.data[mesh_id];    
    var bhv_index = i32((*mesh).blas_index);    
    let mesh_bhv_index = bhv_index;
    var nearest = far_plane;  
    var visibility_id = 0u;

    while (bhv_index >= 0)
    { 
        let node = &bhv.data[u32(bhv_index)];   
        let intersection = intersect_aabb(ray, (*node).min, (*node).max);
        if (intersection >= nearest) {
            bhv_index = select((*node).miss, (*node).miss + mesh_bhv_index, (*node).miss >= 0);
            continue;
        }
        if ((*node).reference < 0) {
            //inner node
            bhv_index = bhv_index + 1;
            continue;  
        }
        //leaf node
        let meshlet_id = (*mesh).meshlets_offset + u32((*node).reference);             
        if (!is_meshlet_visible(meshlet_id)) {
            bhv_index = select((*node).miss, (*node).miss + mesh_bhv_index, (*node).miss >= 0);
            continue;
        }
        let hit = intersect_meshlet(ray, mesh_id, meshlet_id, nearest);
        visibility_id = select(visibility_id, hit.visibility_id, hit.distance < nearest);
        nearest = hit.distance;
        bhv_index = select((*node).miss, (*node).miss + mesh_bhv_index, (*node).miss >= 0);
    }
    return Result(nearest, visibility_id);
}


