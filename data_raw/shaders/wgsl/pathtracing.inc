struct RadianceResult {
    direction: vec3<f32>,
    radiance: vec3<f32>, 
    throughput_weight: vec3<f32>, 
}

fn pixarOnb(n: vec3f) -> mat3x3f {
    // https://www.jcgt.org/published/0006/01/01/paper-lowres.pdf
    let s = select(-1f, 1f, n.z >= 0f);
    let a = -1f / (s + n.z);
    let b = n.x * n.y * a;
    let u = vec3(1f + s * n.x * n.x * a, s * b, -s * n.x);
    let v = vec3(b, s + n.y * n.y * a, -n.y);

    return mat3x3(u, v, n);
}

fn sample_sphere(random_numbers: vec2<f32>) -> vec3<f32> {
    let z = 2. * random_numbers.y - 1.;
    let phi = 2. * MATH_PI * random_numbers.x;
    let s = sqrt(1. - z * z);
    let x = cos(phi) * s;
    let y = sin(phi) * s;
    return normalize(vec3<f32>(x, y, z));
}

fn sample_hemisphere(random_numbers: vec2<f32>, n: vec3<f32>) -> vec3<f32> {
    let v = sample_sphere(random_numbers);
    return v * sign(dot(v, n));
}

fn sample_cosine_wheighted_hemisphere(random_numbers: vec2<f32>, n: vec3<f32>) -> vec3<f32> {
    let phi = 2. * MATH_PI * random_numbers.y;
    let sin_theta = sqrt(1. - random_numbers.x);

    let x = cos(phi) * sin_theta;
    let y = sin(phi) * sin_theta;
    let z = sqrt(random_numbers.x);

    let v = vec3<f32>(x, y, z);
    let onb = pixarOnb(n);
    return normalize(onb * v);
}

fn sample_in_cone(random_numbers: vec2<f32>, cos_max: f32, n: vec3<f32>) -> vec3<f32> {
    let cos_theta = 1. - random_numbers.x * (1. - cos_max);
    let sin_theta = sqrt(1. - cos_theta * cos_theta);
    let phi = 2. * MATH_PI * random_numbers.y;

    let x = cos(phi) * sin_theta;
    let y = sin(phi) * sin_theta;
    let z = cos_theta;
    
    let v = vec3<f32>(x, y, z);
    let onb = pixarOnb(n);
    return normalize(onb * v);
}

// Need constant_data, meshlets, meshes, indices, runtime_vertices, vertices_attributes
fn compute_radiance_from_visibility(visibility_id: u32, hit_point: vec3<f32>, seed: vec2<f32>, old_radiance: vec3<f32>, old_throughput_weight: vec3<f32>) -> RadianceResult { 
    var pixel_data = visibility_to_gbuffer(visibility_id, hit_point);
    let material_info = compute_color_from_material(pixel_data.material_id, &pixel_data);

    let direction = sample_in_cone(seed, MATH_PI * 0.25, pixel_data.normal);
    //let direction = sample_hemisphere(seed, pixel_data.normal);

    let radiance = old_throughput_weight * material_info.f_emissive.rgb; 
    let d = dot(pixel_data.normal, direction);
    let throughput_weight = (material_info.f_color.rgb * 2. * d);

    return RadianceResult(direction, radiance, throughput_weight);
}