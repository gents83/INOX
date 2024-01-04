struct RadianceData {
    direction: vec3<f32>,
    radiance: vec3<f32>, 
    throughput_weight: vec3<f32>, 
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

// Need constant_data, meshlets, meshes, indices, runtime_vertices, vertices_attributes
fn compute_radiance_from_visibility(visibility_id: u32, hit_point: vec3<f32>, seed: vec2<f32>, radiance: vec3<f32>, throughput_weight: vec3<f32>) -> RadianceData { 
    var pixel_data = visibility_to_gbuffer(visibility_id, hit_point);
    let material_info = compute_color_from_material(pixel_data.material_id, &pixel_data);

    let direction = sample_hemisphere(seed, pixel_data.normal);

    //let rad = select(radiance + throughput_weight * material_info.emissive_color, material_info.base_color.rgb, length(radiance) == 0.); 
    let rad = radiance + throughput_weight * material_info.emissive_color; 
    let d = dot(pixel_data.normal, direction);
    let weight = throughput_weight * (material_info.base_color.rgb * 2. * d);

    return RadianceData(direction, rad, weight);
}