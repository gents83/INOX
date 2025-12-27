fn sample_disk_uniform(u: vec2<f32>) -> vec2<f32> {
    let r = sqrt(u.x);
    let theta = 2.0 * 3.14159265 * u.y;
    return vec2<f32>(r * cos(theta), r * sin(theta));
}

fn sample_cosine_weighted_hemisphere(u: vec2<f32>) -> vec3<f32> {
    let d = sample_disk_uniform(u);
    let z = sqrt(max(0.0, 1.0 - d.x * d.x - d.y * d.y));
    return vec3<f32>(d.x, d.y, z);
}

fn sample_ggx_vndf(roughness: f32, uh: vec2<f32>, V: vec3<f32>) -> vec3<f32> {
    // Input Ve: view direction in local tangent space
    // Mapping 2D point to hemisphere with matching density (VNDF)
    
    let alpha = roughness * roughness;
    let alpha2 = alpha * alpha;

    // Rescale view vector
    let Vh = normalize(vec3<f32>(alpha * V.x, alpha * V.y, V.z));

    // Orthonormal basis for Vh
    let lensq = Vh.x * Vh.x + Vh.y * Vh.y;
    let T1 = select(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(-Vh.y, Vh.x, 0.0) * inverseSqrt(lensq), lensq > 0.0);
    let T2 = cross(Vh, T1);

    let r = sqrt(uh.x);
    let phi = 2.0 * 3.14159265 * uh.y;
    let t1 = r * cos(phi);
    let t2 = r * sin(phi);
    let s = 0.5 * (1.0 + Vh.z);
    let t2_ = (1.0 - s) * sqrt(1.0 - t1 * t1) + s * t2;

    let Nh = t1 * T1 + t2_ * T2 + sqrt(max(0.0, 1.0 - t1 * t1 - t2_ * t2_)) * Vh;

    // Transform back to ellipsoid
    let Ne = normalize(vec3<f32>(alpha * Nh.x, alpha * Nh.y, max(0.0, Nh.z)));

    return Ne;
}

fn importance_sample_ggx(roughness: f32, u: vec2<f32>) -> vec3<f32> {
    let a = roughness * roughness;
    let phi = 2. * 3.14159265 * u.x;
    let cos_theta = sqrt((1. - u.y) / (1. + (a * a - 1.) * u.y));
    let sin_theta = sqrt(1. - cos_theta * cos_theta);
    
    // Returns H in tangent space (aligned to Z+)
    let H = vec3<f32>(sin_theta * cos(phi), sin_theta * sin(phi), cos_theta);
    return H;
}
