const MAX_PATH_BOUNCES: u32 = 8u;
const NUM_SAMPLES_PER_PIXEL: u32 = 4u;
const ADDITIONAL_EMISSIVE: f32 = 0.2;


struct RadianceData {
    ray: Ray,
    seed: vec2<u32>,
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
    if (d < 0.) {
        direction -= 2. * d * normal;
    }
    return direction;
}

// Need constant_data, meshlets, meshes, indices, runtime_vertices, vertices_attributes
fn compute_radiance_from_visibility(visibility_id: u32, screen_uv: vec2<f32>, data: RadianceData) -> RadianceData { 
    var pixel_data = visibility_to_gbuffer(visibility_id, screen_uv);
    
    let material = &materials.data[pixel_data.material_id];
    var emissive_color = (*material).emissive_color + vec3<f32>(ADDITIONAL_EMISSIVE);
    if (has_texture(pixel_data.material_id, TEXTURE_TYPE_EMISSIVE)) {
        let t = material_texture_uv(&pixel_data, TEXTURE_TYPE_EMISSIVE);
        emissive_color *= t.rgb;
    }
    pixel_data.color *= (*material).base_color;
    if (has_texture(pixel_data.material_id, TEXTURE_TYPE_BASE_COLOR)) {  
        let uv = material_texture_uv(&pixel_data, TEXTURE_TYPE_BASE_COLOR);
        let texture_color = sample_texture(uv);
        pixel_data.color *= texture_color * (*material).diffuse_color;
    }
    pixel_data.normal = compute_normal(&pixel_data);

    let seed = get_random_numbers(data.seed);    
    let rnd = vec2<f32>(seed) * 2.32830643654e-10;
    let direction = sample_hemisphere(rnd, pixel_data.normal);

    let rad = data.radiance + data.throughput_weight * emissive_color;
    let d = dot(pixel_data.normal, direction);
    let weight = data.throughput_weight * (pixel_data.color.rgb * 2. * d);
    let origin = data.ray.origin + (data.ray.direction * data.ray.t_max) + direction * HIT_EPSILON;
    let ray = Ray(origin, 0., direction, MAX_FLOAT);
    return RadianceData(ray, seed, rad, weight);
}