struct Ray {
    origin: vec3<f32>,
    min: f32,
    direction: vec3<f32>,
    max: f32,
}

struct Intersection {
    position: vec3<f32>,
    t: f32,
    normal: vec3<f32>,
    visibility_id: u32,
}


// Compute the intersection of a ray with a triangle
// using Möller–Trumbore algorithm
fn intersect_triangle(r: Ray, v0: vec3<f32>, v1: vec3<f32>, v2: vec3<f32>) -> bool
{        
    let EPSILON: f32 = 0.0001;

    let e1 = v1 - v0;
    let e2 = v2 - v0;
    let h = cross(r.direction, e2);
    let a = dot(e1, h);

    if (a < EPSILON && a > EPSILON) {
        return false;
    }

    let f = 1. / a;
    let s = r.origin - v0;
    let u = f * dot(s, h);

    if (u < 0. || u > 1.) {
        return false;
    }

    let q = cross(s, e1);
    let v = f * dot(r.direction, q);
    if (v < 0. || u + v > 1.) {
        return false;
    }
    
    let t = f * dot(e2, q);
    if (t < EPSILON)
    {
        return false;
    }

    return true;
}