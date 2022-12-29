let MAX_FLOAT: f32 = 3.402823466e+38;
let HIT_EPSILON: f32 = 0.0001;
let NEG_HIT_EPSILON: f32 = -0.0001;
let ONE_PLUS_HIT_EPSILON: f32 = 1.0001;
let INVALID_NODE: i32 = -1;

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
    
    let origin = unproject(clip_coords.xy, 0.);
    let far = unproject(clip_coords.xy, 1.);
    let direction = normalize(far - origin);
    
    let ray = Ray(origin, direction);
    return ray;
}

fn intersect_oobb(r: Ray, oobb_min: vec3<f32>, oobb_max: vec3<f32>) -> f32 
{     
    let inverse_dir = 1. / r.direction;
    let t_bottom = (oobb_min - r.origin) * inverse_dir;
    let t_top = (oobb_max - r.origin) * inverse_dir;

    let t_min = min(t_top, t_bottom);
    let t_max = max(t_top, t_bottom);

    let smallest_max = min(min(t_max.x, t_max.y), min(t_max.x, t_max.z));
    let largest_min = max(max(t_min.x, t_min.y), max(t_min.x, t_min.z));

    if (smallest_max < largest_min || smallest_max < 0.) 
    { 
        return MAX_FLOAT; 
    }
    if (largest_min > 0.) {
        return largest_min;
    } else {
        return smallest_max;
    } 
}

fn ray_triangle_intersection_point(r: Ray, v0: vec3<f32>, v1: vec3<f32>, v2: vec3<f32>) -> vec3<f32>
{
    let v1v0 = v1 - v0;
    let v2v0 = v2 - v0;
    let rov0 = r.origin - v0;
    let  n = cross( v1v0, v2v0 );
    let  q = cross( rov0, r.direction );
    let d = 1.0/dot( r.direction, n );
    let u = d*dot( -q, v2v0 );
    let v = d*dot(  q, v1v0 );
    var t = d*dot( -n, rov0 );
    if( u<0.0 || v<0.0 || (u+v)>1.0 ) { t = -1.0; }
    return vec3<f32>( t, u, v );
}

fn ray_triangle_intersection_point_distance(r: Ray, v0: vec3<f32>, v1: vec3<f32>, v2: vec3<f32>) -> f32
{
    let intersection = ray_triangle_intersection_point(r, v0, v1, v2);
    if(intersection.x < 0.)
    {
        return MAX_FLOAT;
    }
    return length(intersection - r.origin);
}

fn traverse_bhv(ray: Ray, mesh_id: u32) -> Result {
    let mesh = &meshes.data[mesh_id];    
    var bhv_index = i32((*mesh).bhv_index);    
    let mesh_bhv_index = bhv_index;
    let nodes_count = i32(arrayLength(&bhv.data));
    var nearest = MAX_FLOAT;  
    var visibility_id = 0u;

    while (bhv_index >= 0 && bhv_index < nodes_count)
    {
        let node = &bhv.data[u32(bhv_index)];    
        let oobb_min = vec4<f32>(transform_vector((*node).min, (*mesh).position, (*mesh).orientation, (*mesh).scale), 1.);
        let oobb_max = vec4<f32>(transform_vector((*node).max, (*mesh).position, (*mesh).orientation, (*mesh).scale), 1.);
        let intersection = intersect_oobb(ray, oobb_min.xyz, oobb_max.xyz);
        if (intersection < nearest) {
            if ((*node).reference >= 0) {
                //leaf node
                let reference = u32((*node).reference);
                let meshlet_id = (*mesh).meshlets_offset + (reference >> 8u) - 1u; 
                let primitive_id = reference & 255u;
                let meshlet = &meshlets.data[u32(meshlet_id)];
                    
                let index_offset = (*mesh).indices_offset + (*meshlet).indices_offset + primitive_id * 3u;
                let i1 = indices.data[index_offset];
                let i2 = indices.data[index_offset + 1u];
                let i3 = indices.data[index_offset + 2u];

                let v1 = &vertices.data[(*mesh).vertex_offset + i1];
                let v2 = &vertices.data[(*mesh).vertex_offset + i2];
                let v3 = &vertices.data[(*mesh).vertex_offset + i3];
                
                let oobb_size = oobb_max.xyz - oobb_min.xyz;
                
                let p1 = oobb_min.xyz + decode_as_vec3(positions.data[(*v1).position_and_color_offset]) * oobb_size;
                let p2 = oobb_min.xyz + decode_as_vec3(positions.data[(*v2).position_and_color_offset]) * oobb_size;
                let p3 = oobb_min.xyz + decode_as_vec3(positions.data[(*v3).position_and_color_offset]) * oobb_size;

                let hit_distance = ray_triangle_intersection_point_distance(ray, p1.xyz, p2.xyz, p3.xyz);
                if (hit_distance < nearest) {
                    visibility_id = (meshlet_id + 1u) << 8u | primitive_id;
                    nearest = hit_distance;
                }
            } else {
                //inner node
                bhv_index = bhv_index + 1;
                continue;                
            }
        }        
        bhv_index = (*node).miss;
        if bhv_index >= 0 {
            bhv_index += mesh_bhv_index;
        }
    }
    return Result(nearest, visibility_id);
}