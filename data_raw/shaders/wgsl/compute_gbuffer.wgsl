#import "common.inc"
#import "utils.inc"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> indices: Indices;
@group(0) @binding(2)
var<storage, read> vertices_positions: VerticesPositions;
@group(0) @binding(3)
var<storage, read> vertices_attributes: VerticesAttributes;
@group(0) @binding(4)
var<storage, read> instances: Instances;
@group(0) @binding(5)
var<storage, read> transforms: Transforms;
@group(0) @binding(6)
var<storage, read> meshes: Meshes;
@group(0) @binding(7)
var<storage, read> meshlets: Meshlets;

@group(1) @binding(0)
var visibility_texture: texture_2d<u32>;
@group(1) @binding(1)
var depth_texture: texture_depth_2d;
@group(1) @binding(2)
var<storage, read_write> data_buffer_0: array<RayPackedData>;
@group(1) @binding(3)
var<storage, read_write> data_buffer_1: array<RadiancePackedData>;
@group(1) @binding(4)
var<storage, read_write> data_buffer_2: array<ThroughputPackedData>;

#import "matrix_utils.inc"
#import "geom_utils.inc"
#import "visibility_utils.inc"
#import "pathtracing.inc"

const WORKGROUP_SIZE: u32 = 8u;

@compute
@workgroup_size(WORKGROUP_SIZE, WORKGROUP_SIZE, 1)
fn main(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let dimensions = vec2<u32>(DEFAULT_WIDTH, DEFAULT_HEIGHT);
    let pixel = vec2<u32>(global_invocation_id.x, global_invocation_id.y);
    if (pixel.x >= dimensions.x || pixel.y >= dimensions.y) {
        return;
    }

    let data_index = global_invocation_id.y * dimensions.x + global_invocation_id.x;

    // Default: clear G-buffer
    data_buffer_0[data_index].origin_tmin = vec4<f32>(0.);
    data_buffer_0[data_index].direction_tmax = vec4<f32>(0.);
    data_buffer_1[data_index].data = vec4<f32>(0.);
    data_buffer_2[data_index].data = vec4<f32>(0.);

    let visibility_dimensions = textureDimensions(visibility_texture);
    let visibility_scale = vec2<f32>(visibility_dimensions) / vec2<f32>(dimensions);
    let visibility_pixel = vec2<u32>(vec2<f32>(pixel) * visibility_scale);
    let visibility_value = textureLoad(visibility_texture, visibility_pixel, 0);
    let visibility_id = visibility_value.r;

    if (visibility_id == 0u || (visibility_id & 0xFFFFFFFFu) == 0xFF000000u) {
        return;
    }

    let depth_dimensions = textureDimensions(depth_texture);
    let depth_scale = vec2<f32>(depth_dimensions) / vec2<f32>(dimensions);
    let depth_pixel = vec2<u32>(vec2<f32>(pixel) * depth_scale);
    let depth = textureLoad(depth_texture, depth_pixel, 0);
    let hit_point = pixel_to_world(depth_pixel, depth_dimensions, depth);

    let pixel_data = visibility_to_gbuffer(visibility_id, hit_point);

    write_gbuffer(
        data_index,
        pixel_data.world_pos,
        pixel_data.normal,
        pixel_data.material_id,
        pixel_data.tangent,
        pixel_data.uv_set[0].xy,
        pixel_data.instance_id
    );
}
