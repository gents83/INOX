#import "common.inc"
#import "utils.inc"
#import "matrix_utils.inc"
#import "geom_utils.inc"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> scene_data: array<u32>;

@group(1) @binding(0)
var visibility_texture: texture_2d<u32>;
@group(1) @binding(1)
var depth_texture: texture_depth_2d;
@group(1) @binding(2)
var<storage, read_write> data_buffer_0: array<f32>;
@group(1) @binding(3)
var<storage, read_write> data_buffer_1: array<f32>;
@group(1) @binding(4)
var<storage, read_write> data_buffer_2: array<f32>;

fn table_offset(kind: u32) -> u32 { return scene_data[4u + kind * 4u]; }
fn load_f32(offset: u32) -> f32 { return bitcast<f32>(scene_data[offset]); }
fn load_vec3(offset: u32) -> vec3<f32> { return vec3<f32>(load_f32(offset), load_f32(offset + 1u), load_f32(offset + 2u)); }
fn surface_pixel_to_world(pixel: vec2<u32>, dimensions: vec2<u32>, depth: f32) -> vec3<f32> {
    let uv = (vec2<f32>(pixel) + vec2<f32>(0.5)) / vec2<f32>(dimensions);
    let clip = vec4<f32>(uv.x * 2.0 - 1.0, (1.0 - uv.y) * 2.0 - 1.0, depth, 1.0);
    let world = constant_data.inverse_view_proj * clip;
    return world.xyz / max(world.w, MATH_EPSILON);
}
fn write_empty(pixel: u32) {
    let o = pixel * SIZE_OF_DATA_BUFFER_ELEMENT;
    data_buffer_0[o] = 0.0; data_buffer_0[o + 1u] = 0.0; data_buffer_0[o + 2u] = 0.0; data_buffer_0[o + 3u] = 0.0;
    data_buffer_1[o] = 0.0; data_buffer_1[o + 1u] = 0.0; data_buffer_1[o + 2u] = 1.0; data_buffer_1[o + 3u] = 0.0;
    data_buffer_2[o] = 0.0; data_buffer_2[o + 1u] = 0.0; data_buffer_2[o + 2u] = 0.0; data_buffer_2[o + 3u] = 0.0;
}

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dimensions = vec2<u32>(u32(constant_data.screen_width), u32(constant_data.screen_height));
    if (id.x >= dimensions.x || id.y >= dimensions.y) { return; }
    let pixel = id.y * dimensions.x + id.x;
    let visibility = textureLoad(visibility_texture, id.xy, 0).r;
    if (visibility == 0u || (visibility & 0xFF000000u) == 0xFF000000u) { write_empty(pixel); return; }

    let depth = textureLoad(depth_texture, id.xy, 0);
    let hit_point = surface_pixel_to_world(id.xy, dimensions, depth);
    let instance_id = (visibility >> 8u) - 1u;
    let primitive_id = visibility & 255u;
    let instance_base = table_offset(3u) + instance_id * 4u;
    let transform_base = table_offset(4u) + scene_data[instance_base] * 16u;
    let mesh_index = scene_data[instance_base + 1u];
    let mesh_base = table_offset(5u) + mesh_index * 16u;
    let meshlet_index = scene_data[instance_base + 2u];
    let meshlet_base = table_offset(6u) + meshlet_index * 20u;
    let index_base = table_offset(0u) + scene_data[meshlet_base + 1u] + primitive_id * 3u;
    let i0 = scene_data[index_base]; let i1 = scene_data[index_base + 1u]; let i2 = scene_data[index_base + 2u];
    let vertex_offset = scene_data[mesh_base];
    let bb_min = load_vec3(transform_base + 8u);
    let bb_max = load_vec3(transform_base + 12u);
    let size = abs(bb_max - bb_min);
    let p0 = bb_min + unpack_unorm_to_3_f32(scene_data[table_offset(1u) + vertex_offset + i0]) * size;
    let p1 = bb_min + unpack_unorm_to_3_f32(scene_data[table_offset(1u) + vertex_offset + i1]) * size;
    let p2 = bb_min + unpack_unorm_to_3_f32(scene_data[table_offset(1u) + vertex_offset + i2]) * size;
    let orientation = vec4<f32>(load_f32(transform_base), load_f32(transform_base + 1u), load_f32(transform_base + 2u), load_f32(transform_base + 3u));
    let position_scale = load_vec3(transform_base + 4u);
    let scale = vec3<f32>(load_f32(transform_base + 7u), load_f32(transform_base + 11u), load_f32(transform_base + 15u));
    let w0 = transform_vector(p0, position_scale, orientation, scale);
    let w1 = transform_vector(p1, position_scale, orientation, scale);
    let w2 = transform_vector(p2, position_scale, orientation, scale);
    let normal = normalize(cross(w1 - w0, w2 - w0));
    let material_id = bitcast<u32>(bitcast<i32>(scene_data[mesh_base + 3u]));
    let o = pixel * SIZE_OF_DATA_BUFFER_ELEMENT;
    data_buffer_0[o] = hit_point.x; data_buffer_0[o + 1u] = hit_point.y; data_buffer_0[o + 2u] = hit_point.z;
    data_buffer_0[o + 3u] = f32(pack2x16float(octahedral_mapping(normal)));
    data_buffer_1[o] = f32(material_id); data_buffer_1[o + 1u] = 1.0; data_buffer_1[o + 2u] = 1.0; data_buffer_1[o + 3u] = 0.0;
    data_buffer_2[o] = 0.0; data_buffer_2[o + 1u] = f32(instance_id); data_buffer_2[o + 2u] = 0.0; data_buffer_2[o + 3u] = 0.0;
}
