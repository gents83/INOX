#import "common.inc"
#import "utils.inc"
#import "wavefront_records.inc"

@group(0) @binding(0) var<uniform> constant_data: ConstantData;
@group(0) @binding(1) var<storage, read> scene_data: array<u32>;
@group(1) @binding(0) var<storage, read> gbuffer_world: array<f32>;
@group(1) @binding(1) var<storage, read> gbuffer_material: array<f32>;
@group(1) @binding(2) var<storage, read> ray_queue: array<WavefrontRay>;
@group(1) @binding(3) var<storage, read> hit_queue_header: array<WavefrontQueueHeader>;
@group(1) @binding(4) var<storage, read> hit_queue: array<WavefrontHit>;
@group(1) @binding(5) var<storage, read_write> next_queue_header: array<WavefrontQueueHeader>;
@group(1) @binding(6) var<storage, read_write> next_queue: array<WavefrontRay>;

fn append_next(ray: WavefrontRay) {
    let slot = atomicAdd(&next_queue_header[0].count, 1u);
    if (slot < next_queue_header[0].capacity) { next_queue[slot] = ray; }
    else { atomicStore(&next_queue_header[0].overflowed, 1u); }
}
fn hash_u32(v: u32) -> u32 { var x=v; x ^= x>>16u; x*=0x7feb352du; x ^= x>>15u; x*=0x846ca68bu; return x^(x>>16u); }
fn table_offset(kind: u32) -> u32 { return scene_data[4u + kind * 4u]; }
fn load_material(material_id: u32, word: u32) -> f32 { return bitcast<f32>(scene_data[table_offset(8u) + material_id * 48u + word]); }

@compute
@workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;
    let count = min(atomicLoad(&hit_queue_header[0].count), hit_queue_header[0].capacity);
    if (index >= count) { return; }
    let hit = hit_queue[index];
    let ray = ray_queue[hit.ray_index];
    if (hit.visibility_id == 0u || ray.bounce >= constant_data.indirect_light_num_bounces) { return; }
    let offset = ray.pixel_index * SIZE_OF_DATA_BUFFER_ELEMENT;
    let normal = octahedral_unmapping(unpack2x16float(bitcast<u32>(gbuffer_world[offset + 3u])));
    let material_id = max(bitcast<i32>(bitcast<u32>(gbuffer_material[offset])), 0);
    let roughness = clamp(load_material(u32(material_id), 16u), 0.045, 1.0);
    let metallic = clamp(load_material(u32(material_id), 17u), 0.0, 1.0);
    let transmission = clamp(load_material(u32(material_id), 19u), 0.0, 1.0);
    let ior = max(load_material(u32(material_id), 18u), 1.001);
    let view = normalize(-ray.direction_and_t_max.xyz);
    let seed = hash_u32(ray.pixel_index ^ constant_data.frame_index ^ ray.bounce * 31u);
    let phi = f32(seed & 65535u) / 65535.0 * 6.2831853;
    let tangent = normalize(select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), abs(normal.x) > 0.5));
    let bitangent = normalize(cross(normal, tangent));
    let diffuse_direction = normalize(tangent * cos(phi) * 0.7 + bitangent * sin(phi) * 0.7 + normal * 0.7);
    let reflection_direction = normalize(reflect(-view, normal));
    let refraction_direction = normalize(refract(-view, normal, 1.0 / ior));
    let direction = select(
        select(diffuse_direction, reflection_direction, metallic > 0.5 || roughness < 0.2),
        refraction_direction,
        transmission > 0.5 && length(refraction_direction) > 0.0
    );
    let origin = vec3<f32>(gbuffer_world[offset], gbuffer_world[offset + 1u], gbuffer_world[offset + 2u]) + normal * HIT_EPSILON;
    append_next(WavefrontRay(vec4<f32>(origin, HIT_EPSILON), vec4<f32>(direction, MAX_TRACING_DISTANCE), ray.throughput * 0.8, ray.pixel_index, ray.bounce + 1u, ray.flags, ray.sample_index));
}
