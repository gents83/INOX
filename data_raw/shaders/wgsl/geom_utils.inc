
struct Derivatives {
    dx: vec3<f32>,
    dy: vec3<f32>,
}

fn pixel_to_normalized(image_pixel: vec2<u32>, image_size: vec2<u32>) -> vec2<f32> {
    return ((vec2<f32>(0.5) + vec2<f32>(image_pixel)) / vec2<f32>(image_size));
}
fn clip_to_normalized(clip_coords: vec2<f32>) -> vec2<f32> {
    return (clip_coords + vec2<f32>(1.)) * vec2<f32>(0.5);
}

fn pixel_to_clip(image_pixel: vec2<u32>, image_size: vec2<u32>) -> vec2<f32> {
    var clip_coords = 2. * pixel_to_normalized(image_pixel, image_size) - vec2<f32>(1.);
    clip_coords.y = -clip_coords.y;
    return clip_coords;
}

fn pixel_to_world(image_pixel: vec2<u32>, image_size: vec2<u32>, depth: f32) -> vec3<f32> {
    let clip_coords = pixel_to_clip(image_pixel, image_size);
    let world_pos = clip_to_world(clip_coords, depth);
    return world_pos;
}

fn clip_to_world(clip_coords: vec2<f32>, depth: f32) -> vec3<f32> {    
    var world_pos = constant_data.inverse_view_proj * vec4<f32>(clip_coords, depth, 1.);
    world_pos /= world_pos.w;
    return world_pos.xyz;
}

fn world_to_clip(world_pos: vec3<f32>) -> vec3<f32> {    
	let ndc_pos: vec4<f32> = constant_data.view_proj * vec4<f32>(world_pos, 1.);
	return ndc_pos.xyz / ndc_pos.w;
}

fn view_pos() -> vec3<f32> {    
    var world_pos = constant_data.inverse_view_proj * vec4<f32>(0.,0.,0., 1.);
    world_pos /= world_pos.w;
    return world_pos.xyz;
}

fn compute_barycentrics_3d(p1: vec3<f32>, p2: vec3<f32>, p3: vec3<f32>, p: vec3<f32>) -> vec3<f32> {
    let v1 = p - p1;
    let v2 = p - p2;
    let v3 = p - p3;
    
    let area = length(cross(v1 - v2, v1 - v3)); 
    return vec3<f32>(length(cross(v2, v3)) / area, length(cross(v3, v1)) / area, length(cross(v1, v2)) / area); 
}

fn compute_barycentrics_2d(a: vec2<f32>, b: vec2<f32>, c: vec2<f32>, p: vec2<f32>) -> vec3<f32> {
    let v0 = b - a;
    let v1 = c - a;
    let v2 = p - a;
    
    let d00 = dot(v0, v0);    
    let d01 = dot(v0, v1);    
    let d11 = dot(v1, v1);
    let d20 = dot(v2, v0);
    let d21 = dot(v2, v1);
    
    let inv_denom = 1. / (d00 * d11 - d01 * d01);    
    let v = (d11 * d20 - d01 * d21) * inv_denom;    
    let w = (d00 * d21 - d01 * d20) * inv_denom;    
    let u = 1. - v - w;

    return vec3 (u,v,w);
}

// Engel's barycentric coord partial derivs function. Follows equation from [Schied][Dachsbacher]
// Computes the partial derivatives of point's barycentric coordinates from the projected screen space vertices
fn compute_partial_derivatives(v0: vec2<f32>, v1: vec2<f32>, v2: vec2<f32>) -> Derivatives
{
    let d = 1. / determinant(mat2x2<f32>(v2-v1, v0-v1));
    
    return Derivatives(vec3<f32>(v1.y - v2.y, v2.y - v0.y, v0.y - v1.y) * d, vec3<f32>(v2.x - v1.x, v0.x - v2.x, v1.x - v0.x) * d);
}

// Interpolate 2D attributes using the partial derivatives and generates dx and dy for texture sampling.
fn interpolate_2d_attribute(a0: vec2<f32>, a1: vec2<f32>, a2: vec2<f32>, deriv: Derivatives, delta: vec2<f32>) -> vec2<f32>
{
	let attr0 = vec3<f32>(a0.x, a1.x, a2.x);
	let attr1 = vec3<f32>(a0.y, a1.y, a2.y);
	let attribute_x = vec2<f32>(dot(deriv.dx, attr0), dot(deriv.dx, attr1));
	let attribute_y = vec2<f32>(dot(deriv.dy, attr0), dot(deriv.dy, attr1));
	let attribute_s = a0;
	
	return (attribute_s + delta.x * attribute_x + delta.y * attribute_y);
}

// Interpolate vertex attributes at point 'd' using the partial derivatives
fn interpolate_3d_attribute(a0: vec3<f32>, a1: vec3<f32>, a2: vec3<f32>, deriv: Derivatives, delta: vec2<f32>) -> vec3<f32>
{
	let attr0 = vec3<f32>(a0.x, a1.x, a2.x);
	let attr1 = vec3<f32>(a0.y, a1.y, a2.y);
	let attr2 = vec3<f32>(a0.z, a1.z, a2.z);
    let attributes = mat3x3<f32>(a0, a1, a2);
	let attribute_x = attributes * deriv.dx;
	let attribute_y = attributes * deriv.dy;
	let attribute_s = a0;
	
	return (attribute_s + delta.x * attribute_x + delta.y * attribute_y);
}