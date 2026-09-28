#import "common.inc"
#import "utils.inc"
#import "wavefront_records.inc"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;

@group(1) @binding(0)
var<storage, read> gbuffer_world: array<f32>;
@group(1) @binding(1)
var<storage, read> gbuffer_material: array<f32>;
@group(1) @binding(2)
var<storage, read> gbuffer_auxiliary: array<f32>;
@group(1) @binding(3)
var<storage, read_write> ray_queue_header: array<WavefrontQueueHeader>;
@group(1) @binding(4)
var<storage, read_write> ray_queue: array<WavefrontRay>;

fn append_primary_ray(ray: WavefrontRay) {
    let slot = atomicAdd(&ray_queue_header[0].count, 1u);
    if (slot < ray_queue_header[0].capacity) {
        ray_queue[slot] = ray;
    } else {
        atomicStore(&ray_queue_header[0].overflowed, 1u);
        atomicAdd(&ray_queue_header[0].dropped_records, 1u);
    }
}

const WORKGROUP_SIZE: u32 = 8u;

fn decode_normal(data_index: u32) -> vec3<f32> {
    return octahedral_unmapping(unpack2x16float(u32(gbuffer_world[data_index + 3u])));
}

fn sample_cosine_hemisphere(random_numbers: vec2<f32>, normal: vec3<f32>) -> vec3<f32> {
    let phi = 2.0 * MATH_PI * random_numbers.x;
    let r = sqrt(random_numbers.y);
    let local = vec3<f32>(r * cos(phi), r * sin(phi), sqrt(1.0 - random_numbers.y));
    let tangent = normalize(select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), abs(normal.x) > 0.5));
    let bitangent = normalize(cross(normal, tangent));
    return normalize(tangent * local.x + bitangent * local.y + normal * local.z);
}

@compute
@workgroup_size(WORKGROUP_SIZE, WORKGROUP_SIZE, 1)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let dimensions = vec2<u32>(u32(constant_data.screen_width), u32(constant_data.screen_height));
    let pixel = global_invocation_id.xy;
    if (pixel.x >= dimensions.x || pixel.y >= dimensions.y) {
        return;
    }

    let pixel_index = pixel.y * dimensions.x + pixel.x;
    let data_index = pixel_index * SIZE_OF_DATA_BUFFER_ELEMENT;
    let normal = decode_normal(data_index);
    if (length(normal) < MATH_EPSILON) {
        return;
    }

    let origin = vec3<f32>(
        gbuffer_world[data_index],
        gbuffer_world[data_index + 1u],
        gbuffer_world[data_index + 2u]
    ) + normal * HIT_EPSILON;
    let seed = vec2<u32>(pixel.x ^ constant_data.frame_index, pixel.y + 17u);
    var random_seed = seed;
    let direction = sample_cosine_hemisphere(get_random_numbers(&random_seed), normal);

    let ray = WavefrontRay(
        vec4<f32>(origin, HIT_EPSILON),
        vec4<f32>(direction, MAX_TRACING_DISTANCE),
        vec4<f32>(1.0),
        pixel_index,
        1u,
        1u,
        constant_data.frame_index
    );
    append_primary_ray(ray);
}
