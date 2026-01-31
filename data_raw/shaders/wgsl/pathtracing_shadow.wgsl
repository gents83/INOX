#import "common.inc"
#import "pathtracing_common.inc"

@group(0) @binding(0) var<uniform> constant_data: ConstantData;
@group(0) @binding(1) var<storage, read> indices: Indices;
@group(0) @binding(2) var<storage, read> vertices_positions: VerticesPositions;
@group(0) @binding(3) var<storage, read> vertices_attributes: VerticesAttributes;
@group(0) @binding(4) var<storage, read> instances: Instances;
@group(0) @binding(5) var<storage, read> transforms: Transforms;
@group(0) @binding(6) var<storage, read> meshes: Meshes;
@group(0) @binding(7) var<storage, read> meshlets: Meshlets;
@group(0) @binding(8) var<storage, read> bvh: BVH;

@group(1) @binding(0) var<storage, read> shadow_rays: array<ShadowRay>;
@group(1) @binding(1) var<storage, read_write> counters: PathTracingCounters;
@group(1) @binding(2) var diffuse_texture: texture_storage_2d<rgba32float, read_write>;
@group(1) @binding(3) var specular_texture: texture_storage_2d<rgba32float, read_write>;
@group(1) @binding(4) var shadow_texture: texture_storage_2d<r32float, read_write>;
@group(1) @binding(5) var ao_texture: texture_storage_2d<r32float, read_write>;
@group(1) @binding(6) var<storage, read_write> data_buffer_1: array<f32>;

#import "raytracing_optimized.inc"

const WORKGROUP_SIZE: u32 = 8u;

@compute
@workgroup_size(WORKGROUP_SIZE, WORKGROUP_SIZE, 1)
fn main(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let index = global_invocation_id.y * u32(constant_data.screen_width) + global_invocation_id.x;
    let shadow_ray = shadow_rays[index];

    if (shadow_ray.t_max <= 0.0) {
        return;
    }

    let result = traverse_bvh_optimized(shadow_ray.origin, shadow_ray.direction, constant_data.tlas_starting_index);

    var visibility = 1.0;
    if (result.distance < shadow_ray.t_max) {
        visibility = 0.0;
    }

    let final_radiance = shadow_ray.contribution * visibility;

    let width = u32(constant_data.screen_width);
    let pixel = vec2<i32>(i32(shadow_ray.pixel_index % width), i32(shadow_ray.pixel_index / width));

    let current = textureLoad(diffuse_texture, pixel);
    textureStore(diffuse_texture, pixel, current + vec4<f32>(final_radiance, 0.0));

    let current_shadow = textureLoad(shadow_texture, pixel);
    textureStore(shadow_texture, pixel, current_shadow + vec4<f32>(visibility, 0., 0., 0.));

    let data_index = shadow_ray.pixel_index * 4u;
    let r = data_buffer_1[data_index];
    let g = data_buffer_1[data_index + 1u];
    let b = data_buffer_1[data_index + 2u];

    data_buffer_1[data_index] = r + final_radiance.x;
    data_buffer_1[data_index + 1u] = g + final_radiance.y;
    data_buffer_1[data_index + 2u] = b + final_radiance.z;
}
