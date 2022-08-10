
struct Derivatives {
    dx: vec3<f32>,
    dy: vec3<f32>,
}

fn compute_barycentrics(a: vec2<f32>, b: vec2<f32>, c: vec2<f32>, p: vec2<f32>) -> vec3<f32> {
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
    
    var deriv: Derivatives;
    deriv.dx = vec3<f32>(v1.y - v2.y, v2.y - v0.y, v0.y - v1.y) * d;
    deriv.dy = vec3<f32>(v2.x - v1.x, v0.x - v2.x, v1.x - v0.x) * d;
    return deriv;
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