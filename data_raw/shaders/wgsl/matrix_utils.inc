
fn extract_scale(m: mat4x4<f32>) -> vec3<f32> 
{
    let s = mat3x3<f32>(m[0].xyz, m[1].xyz, m[2].xyz);
    let sx = length(s[0]);
    let sy = length(s[1]);
    let det = determinant(s);
    var sz = length(s[2]);
    if (det < 0.) 
    {
        sz = -sz;
    }
    return vec3<f32>(sx, sy, sz);
}

fn matrix_row(m: mat4x4<f32>, row: u32) -> vec4<f32> 
{
    if (row == 1u) {
        return vec4<f32>(m[0].y, m[1].y, m[2].y, m[3].y);
    } else if (row == 2u) {
        return vec4<f32>(m[0].z, m[1].z, m[2].z, m[3].z);
    } else if (row == 3u) {
        return vec4<f32>(m[0].w, m[1].w, m[2].w, m[3].w);
    } else {        
        return vec4<f32>(m[0].x, m[1].x, m[2].x, m[3].x);
    }
}

fn normalize_plane(plane: vec4<f32>) -> vec4<f32> 
{
    return (plane / length(plane.xyz));
}

fn rotate_vector(v: vec3<f32>, orientation: vec4<f32>) -> vec3<f32> 
{
    return v + 2. * cross(orientation.xyz, cross(orientation.xyz, v) + orientation.w * v);
}

fn transform_vector(v: vec3<f32>, position: vec3<f32>, orientation: vec4<f32>, scale: vec3<f32>) -> vec3<f32> 
{
    return rotate_vector(v, orientation) * scale + position;
}
