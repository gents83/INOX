let MAX_FLOAT: f32 = 3.402823466e+38;
let HIT_EPSILON: f32 = 0.0001;
let NEG_HIT_EPSILON: f32 = -0.0001;
let ONE_PLUS_HIT_EPSILON: f32 = 1.0001;

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
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


// Compute the intersection of a ray with a triangle
// using Möller–Trumbore algorithm
fn intersect_triangle(r: Ray, v0: vec3<f32>, v1: vec3<f32>, v2: vec3<f32>) -> f32
{        
    let e1 = v1 - v0;
    let e2 = v2 - v0;

    let p = cross(r.direction, e2);
    let det = dot(e1, p);

    if (det <= NEG_HIT_EPSILON || (det > NEG_HIT_EPSILON && det < HIT_EPSILON)) {
        return MAX_FLOAT;
    }

    let inv_det = 1. / det;
    let centered = r.origin - v0;

    let u = inv_det * dot(centered, p);
    if (u < HIT_EPSILON || u > ONE_PLUS_HIT_EPSILON) {
        return MAX_FLOAT;
    }

    let q = cross(centered, e1);
    let v = inv_det * dot(r.direction, q);
    if (v < HIT_EPSILON || u + v > ONE_PLUS_HIT_EPSILON) {
        return MAX_FLOAT;
    }
    
    let t = inv_det * dot(e2, q);
    if (t < HIT_EPSILON)
    {
        return MAX_FLOAT;
    }
    return t;
}