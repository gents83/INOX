let MAX_FLOAT: f32 = 3.402823466e+38;
let HIT_EPSILON: f32 = 0.0001;
let INVALID_NODE: i32 = -1;
let MAX_STEPS: u32 = 1024u;

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

struct Result {
    distance: f32,
    visibility_id: u32,
}

fn compute_ray(image_pixel: vec2<u32>, image_size: vec2<u32>) -> Ray {
    var clip_coords = 2. * (vec2<f32>(image_pixel) / vec2<f32>(image_size)) - vec2<f32>(1., 1.);
    clip_coords.y = -clip_coords.y;
    
    let origin = unproject(clip_coords.xy, -1.);
    let far = unproject(clip_coords.xy, 1.);
    let direction = normalize(far - origin);
    
    let ray = Ray(origin, direction);
    return ray;
}

fn intersect_aabb(r: Ray, aabb_min: vec3<f32>, aabb_max: vec3<f32>) -> f32 
{     
    let inverse_dir = 1. / r.direction;
    let v_min = (aabb_min - r.origin) * inverse_dir;
    let v_max = (aabb_max - r.origin) * inverse_dir;

    let t_min = min(v_min, v_max);
    let t_max = max(v_min, v_max);

    let t_near = max(max(t_min.x, t_min.y), max(t_min.x, t_min.z));
    let t_far = min(min(t_max.x, t_max.y), min(t_max.x, t_max.z));

    if (t_near > t_far || t_far < 0.) 
    { 
        return MAX_FLOAT; 
    }
    if (t_near > 0.) {
        return t_near;
    } else {
        return t_far;
    } 
}


fn intersect_triangle(r: Ray, v0: vec3<f32>, v1: vec3<f32>, v2: vec3<f32>) -> f32
{
    let e1 = v1 - v0;
    let e2 = v2 - v0;

    let p = cross(r.direction, e2);
    let det = dot(e1, p);
    
    if (abs(det) < HIT_EPSILON) { return MAX_FLOAT; }

    // Computes Barycentric coordinates.
    let inv_det = 1. / det;
    let t = r.origin - v0;    
    let u = dot(t, p) * inv_det;
    if (u < 0. || u > 1.) { return MAX_FLOAT; }
    
    let q = cross(t, e1);
    let v = dot(r.direction, q) * inv_det;
    if (v < 0. || u + v > 1.) { return MAX_FLOAT; }

    let t = dot(e2, q) * inv_det;
    if (t < 0.) { return MAX_FLOAT; }
    return t;
}

fn intersect_meshlet(r: Ray, mesh_id: u32, meshlet_id: u32) -> Result {
    let mesh = &meshes.data[mesh_id];    
    let node = &bhv.data[u32((*mesh).bhv_index)];  
    var nearest = MAX_FLOAT;  
    var visibility_id = 0u;
    
    let meshlet = &meshlets.data[meshlet_id];
    let primitive_count = (*meshlet).indices_count / 3u;
    for(var primitive_id = 0u; primitive_id < primitive_count; primitive_id = primitive_id + 1u)
    {       
        let index_offset = (*mesh).indices_offset + (*meshlet).indices_offset + (primitive_id * 3u);
        let i1 = indices.data[index_offset];
        let i2 = indices.data[index_offset + 1u];
        let i3 = indices.data[index_offset + 2u];

        let v1 = &vertices.data[(*mesh).vertex_offset + i1];
        let v2 = &vertices.data[(*mesh).vertex_offset + i2];
        let v3 = &vertices.data[(*mesh).vertex_offset + i3];
        
        let oobb_size = (*node).max - (*node).min;
        
        let p1 = (*node).min + decode_as_vec3(positions.data[(*v1).position_and_color_offset]) * oobb_size;
        let p2 = (*node).min + decode_as_vec3(positions.data[(*v2).position_and_color_offset]) * oobb_size;
        let p3 = (*node).min + decode_as_vec3(positions.data[(*v3).position_and_color_offset]) * oobb_size;
        
        let hit_distance = intersect_triangle(r, p1, p2, p3);
        if (hit_distance < nearest) {
            visibility_id = (meshlet_id + 1u) << 8u | primitive_id;
            nearest = hit_distance;
        }
    }
    return Result(nearest, visibility_id);
}

fn traverse_meshlets(r: Ray, mesh_id: u32) -> Result {
    let mesh = &meshes.data[mesh_id];    
    var nearest = MAX_FLOAT;  
    var visibility_id = 0u;
    for(var i = 0u; i < (*mesh).meshlets_count; i = i + 1u)
    {
        let meshlet_id = (*mesh).meshlets_offset + i;
        let hit = intersect_meshlet(r, mesh_id, meshlet_id);
        if (hit.distance < nearest) {
            visibility_id = hit.visibility_id;
            nearest = hit.distance;
        }
    }
    return Result(nearest, visibility_id);
}

fn traverse_bhv(r: Ray, mesh_id: u32) -> Result {
    let mesh = &meshes.data[mesh_id];    
    var bhv_index = i32((*mesh).bhv_index);    
    let mesh_bhv_index = bhv_index;
    let nodes_count = i32(arrayLength(&bhv.data));
    var nearest = MAX_FLOAT;  
    var visibility_id = 0u;

    var steps = 0u;
    while (bhv_index >= 0 && steps < MAX_STEPS)
    { 
        let node = &bhv.data[u32(bhv_index)];   
        let intersection = intersect_aabb(r, (*node).min, (*node).max);
        if (intersection < nearest) {
            if ((*node).reference >= 0) {
                //leaf node
                let meshlet_id = (*mesh).meshlets_offset + u32((*node).reference);
                let hit = intersect_meshlet(r, mesh_id, meshlet_id);
                if (hit.distance < nearest) {
                    visibility_id = hit.visibility_id;
                    nearest = hit.distance;
                }
            } else {
                //inner node
                bhv_index = bhv_index + 1;
                steps = steps + 1u;
                continue;                
            }
        }        
        bhv_index = (*node).miss;
        if bhv_index >= 0 {
            bhv_index += mesh_bhv_index;
        }
        steps = steps + 1u;
    }
    return Result(nearest, visibility_id);
}