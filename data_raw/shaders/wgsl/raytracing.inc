struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

fn swap_f32(ptr_a: ptr<function, f32>, ptr_b: ptr<function, f32>) {
    let c = *ptr_a;
    *ptr_a = *ptr_b;
    *ptr_b = c;
}

fn intersect_oobb(r: Ray, min: vec3<f32>, max: vec3<f32>) -> bool 
{     
    var tmin = (min.x - r.origin.x) / r.direction.x; 
    var tmax = (max.x - r.origin.x) / r.direction.x; 
 
    if (tmin > tmax) {
        swap_f32(&tmin, &tmax); 
    } 
 
    var tymin = (min.y - r.origin.y) / r.direction.y; 
    var tymax = (max.y - r.origin.y) / r.direction.y; 
 
    if (tymin > tymax) {
        swap_f32(&tymin, &tymax); 
    }  
    if ((tmin > tymax) || (tymin > tmax)) {
        return false; 
    } 
    if (tymin > tmin) {
        tmin = tymin; 
    } 
    if (tymax < tmax) {
        tmax = tymax; 
    }
 
    var tzmin = (min.z - r.origin.z) / r.direction.z; 
    var tzmax = (max.z - r.origin.z) / r.direction.z; 
 
    if (tzmin > tzmax) {
        swap_f32(&tzmin, &tzmax);
    }  
    if ((tmin > tzmax) || (tzmin > tmax)) {
        return false; 
    } 
    if (tzmin > tmin) {
        tmin = tzmin; 
    }  
    if (tzmax < tmax) {
        tmax = tzmax; 
    } 
    return true; 
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