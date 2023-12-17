const MAX_PATH_BOUNCES: u32 = 4u;

struct RadianceData {
    direction: vec3<f32>,
    radiance: vec3<f32>, 
    throughput_weight: vec3<f32>, 
}

fn sample_sphere(random_numbers: vec2<f32>) -> vec3<f32> {
    let z = 2. * random_numbers.y - 1.;
    let phi = 2. * PI * random_numbers.x;
    let s = sqrt(1. - z * z);
    let x = cos(phi) * s;
    let y = sin(phi) * s;
    return vec3<f32>(x, y, z);
}

fn sample_hemisphere(random_numbers: vec2<f32>, normal: vec3<f32>) -> vec3<f32> {
    var direction = sample_sphere(random_numbers);
    let d = dot(normal, direction);
    return select(direction, direction - 2. * d * normal, d < 0.);
}

// Need constant_data, meshlets, meshes, indices, runtime_vertices, vertices_attributes
fn compute_radiance_from_visibility(visibility_id: u32, clip_coords: vec2<f32>, seed: vec2<u32>, data: RadianceData, mvp: mat4x4<f32>) -> RadianceData { 
    var pixel_data = visibility_to_gbuffer(visibility_id, clip_coords, mvp);
    let pbr_data = compute_color(pixel_data.material_id, &pixel_data);

    let rnd = vec2<f32>(seed) * 2.32830643654e-10;
    let direction = sample_hemisphere(rnd, pixel_data.normal);

    let rad = data.radiance + data.throughput_weight * pbr_data.emissive_color;
    let d = dot(pixel_data.normal, direction);
    let weight = data.throughput_weight * (pbr_data.diffuse_color.rgb * 2. * d);

    return RadianceData(direction, rad, weight);
}