fn extract_translation(mat: mat4x4<f32>) -> vec3<f32> {
    return mat[3].xyz;
}
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
fn extract_rotation(mat: mat4x4<f32>) -> mat4x4<f32> {
    let scale = extract_scale(mat);
    return mat4x4(
        vec4(vec3(mat[0].xyz / scale), 0.),
        vec4(vec3(mat[1].xyz / scale), 0.),
        vec4(vec3(mat[2].xyz / scale), 0.),
        vec4(0., 0., 0., 1.),
    );
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


fn extract_frustum_planes(proj: mat4x4<f32>, view: mat4x4<f32>) -> array<vec4<f32>, 6> {
    let view_proj = proj * view;
    var planes: array<vec4<f32>, 6>;

    // Left
    planes[0] = vec4<f32>(view_proj[0][3] + view_proj[0][0], view_proj[1][3] + view_proj[1][0], view_proj[2][3] + view_proj[2][0], view_proj[3][3] + view_proj[3][0]);
    // Right
    planes[1] = vec4<f32>(view_proj[0][3] - view_proj[0][0], view_proj[1][3] - view_proj[1][0], view_proj[2][3] - view_proj[2][0], view_proj[3][3] - view_proj[3][0]);
    // Bottom
    planes[2] = vec4<f32>(view_proj[0][3] + view_proj[0][1], view_proj[1][3] + view_proj[1][1], view_proj[2][3] + view_proj[2][1], view_proj[3][3] + view_proj[3][1]);
    // Top
    planes[3] = vec4<f32>(view_proj[0][3] - view_proj[0][1], view_proj[1][3] - view_proj[1][1], view_proj[2][3] - view_proj[2][1], view_proj[3][3] - view_proj[3][1]);
    // Near
    planes[4] = vec4<f32>(view_proj[0][2], view_proj[1][2], view_proj[2][2], view_proj[3][2]);
    // Far
    planes[5] = vec4<f32>(view_proj[0][3] - view_proj[0][2], view_proj[1][3] - view_proj[1][2], view_proj[2][3] - view_proj[2][2], view_proj[3][3] - view_proj[3][2]);

    // Normalize planes
    for (var i = 0u; i < 6u; i++) {
        let length = length(planes[i].xyz);
        planes[i] = planes[i] / vec4<f32>(length, length, length, length);
    }

    return planes;
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

fn matrix_from_translation(translation: vec3<f32>) -> mat4x4<f32> {
    return mat4x4<f32>(vec4<f32>(1.0, 0.0, 0.0, 0.0),
                      vec4<f32>(0.0, 1.0, 0.0, 0.0),
                      vec4<f32>(0.0, 0.0, 1.0, 0.0),
                      vec4<f32>(translation.x, translation.y, translation.z, 1.0));
}

fn matrix_from_scale(scale: vec3<f32>) -> mat4x4<f32> {
    return mat4x4<f32>(vec4<f32>(scale.x, 0.0, 0.0, 0.0),
                      vec4<f32>(0.0, scale.y, 0.0, 0.0),
                      vec4<f32>(0.0, 0.0, scale.z, 0.0),
                      vec4<f32>(0.0, 0.0, 0.0, 1.0));
}

fn matrix_from_orientation(q: vec4<f32>) -> mat4x4<f32> {
    let xx = q.x * q.x;
    let yy = q.y * q.y;
    let zz = q.z * q.z;
    let xy = q.x * q.y;
    let xz = q.x * q.z;
    let yz = q.y * q.z;
    let wx = q.w * q.x;
    let wy = q.w * q.y;
    let wz = q.w * q.z;

    let m00 = 1.0 - 2.0 * (yy + zz);
    let m01 = 2.0 * (xy - wz);
    let m02 = 2.0 * (xz + wy);

    let m10 = 2.0 * (xy + wz);
    let m11 = 1.0 - 2.0 * (xx + zz);
    let m12 = 2.0 * (yz - wx);

    let m20 = 2.0 * (xz - wy);
    let m21 = 2.0 * (yz + wx);
    let m22 = 1.0 - 2.0 * (xx + yy);

    // Utilizza la funzione mat4x4 per creare la matrice 4x4
    return mat4x4<f32>(
        vec4<f32>(m00, m01, m02, 0.0),
        vec4<f32>(m10, m11, m12, 0.0),
        vec4<f32>(m20, m21, m22, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0)
    );
}

fn transform_matrix(position: vec3<f32>, orientation: vec4<f32>, scale: vec3<f32>) -> mat4x4<f32> {
    let translation_matrix = matrix_from_translation(position);
    let rotation_matrix = matrix_from_orientation(orientation);
    let scale_matrix = matrix_from_scale(scale);    
    return translation_matrix * rotation_matrix * scale_matrix;
}

fn matrix_inverse(m: mat4x4<f32>) -> mat4x4<f32> {
    let a00 = m[0][0]; let a01 = m[0][1]; let a02 = m[0][2]; let a03 = m[0][3];
    let a10 = m[1][0]; let a11 = m[1][1]; let a12 = m[1][2]; let a13 = m[1][3];
    let a20 = m[2][0]; let a21 = m[2][1]; let a22 = m[2][2]; let a23 = m[2][3];
    let a30 = m[3][0]; let a31 = m[3][1]; let a32 = m[3][2]; let a33 = m[3][3];

    let b00 = a00 * a11 - a01 * a10;
    let b01 = a00 * a12 - a02 * a10;
    let b02 = a00 * a13 - a03 * a10;
    let b03 = a01 * a12 - a02 * a11;
    let b04 = a01 * a13 - a03 * a11;
    let b05 = a02 * a13 - a03 * a12;
    let b06 = a20 * a31 - a21 * a30;
    let b07 = a20 * a32 - a22 * a30;
    let b08 = a20 * a33 - a23 * a30;
    let b09 = a21 * a32 - a22 * a31;
    let b10 = a21 * a33 - a23 * a31;
    let b11 = a22 * a33 - a23 * a32;

    let det = b00 * b11 - b01 * b10 + b02 * b09 + b03 * b08 - b04 * b07 + b05 * b06;
    
    // Ottimizzazione: Calcola l'inverso del determinante una sola volta
    let invDet = 1.0 / det;

    return mat4x4<f32>(
        vec4<f32>((a11 * b11 - a12 * b10 + a13 * b09) * invDet, (a02 * b10 - a01 * b11 - a03 * b09) * invDet, (a31 * b05 - a32 * b04 + a33 * b03) * invDet, (a22 * b04 - a21 * b05 - a23 * b03) * invDet),
        vec4<f32>((a12 * b08 - a10 * b11 - a13 * b07) * invDet, (a00 * b11 - a02 * b08 + a03 * b07) * invDet, (a32 * b02 - a30 * b05 - a33 * b01) * invDet, (a20 * b05 - a22 * b02 + a23 * b01) * invDet),
        vec4<f32>((a10 * b10 - a11 * b08 + a13 * b06) * invDet, (a01 * b08 - a00 * b10 - a03 * b06) * invDet, (a30 * b04 - a31 * b02 + a33 * b00) * invDet, (a21 * b02 - a20 * b04 - a23 * b00) * invDet),
        vec4<f32>((a11 * b07 - a10 * b09 - a12 * b06) * invDet, (a00 * b09 - a01 * b07 + a02 * b06) * invDet, (a31 * b01 - a30 * b03 - a32 * b00) * invDet, (a20 * b03 - a21 * b01 + a22 * b00) * invDet)
    );
}

fn matrix_transpose(m: mat4x4<f32>) -> mat4x4<f32> {
    let a00 = m[0][0]; let a01 = m[0][1]; let a02 = m[0][2]; let a03 = m[0][3];
    let a10 = m[1][0]; let a11 = m[1][1]; let a12 = m[1][2]; let a13 = m[1][3];
    let a20 = m[2][0]; let a21 = m[2][1]; let a22 = m[2][2]; let a23 = m[2][3];
    let a30 = m[3][0]; let a31 = m[3][1]; let a32 = m[3][2]; let a33 = m[3][3];

    return mat4x4<f32>(
        vec4<f32>(a00, a10, a20, a30),
        vec4<f32>(a01, a11, a21, a31),
        vec4<f32>(a02, a12, a22, a32),
        vec4<f32>(a03, a13, a23, a33)
    );
}
