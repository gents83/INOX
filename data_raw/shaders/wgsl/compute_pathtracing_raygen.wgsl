#import "common.inc"
#import "utils.inc"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> indices: Indices;
@group(0) @binding(2)
var<storage, read> vertices_positions: VerticesPositions;
@group(0) @binding(3)
var<storage, read> instances: Instances;
@group(0) @binding(4)
var<storage, read> meshlets: Meshlets;

@group(1) @binding(0)
var visibility_texture: texture_2d<u32>;
@group(1) @binding(1)
var depth_texture: texture_depth_2d;
@group(1) @binding(2)
var<storage, read_write> rays: array<f32>;
@group(1) @binding(3)
var<storage, read_write> hits: array<f32>;
@group(1) @binding(4)
var<storage, read_write> radiance: array<f32>;
@group(1) @binding(5)
var<storage, read_write> throughput: array<f32>;

#import "matrix_utils.inc"
#import "visibility_utils.inc"

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

    let ray_index = (global_invocation_id.y * dimensions.x + global_invocation_id.x) * SIZE_OF_DATA_BUFFER_ELEMENT;

    // Clear buffers
    rays[ray_index] = 0.;
    rays[ray_index + 1u] = 0.;
    rays[ray_index + 2u] = 0.;
    rays[ray_index + 3u] = 0.;
    hits[ray_index] = 0.; // VisibilityID
    hits[ray_index + 1u] = 0.; // t

    throughput[ray_index] = 1.;
    throughput[ray_index + 1u] = 1.;
    throughput[ray_index + 2u] = 1.;
    throughput[ray_index + 3u] = 1.;

    if (constant_data.frame_index == 0u) {
        radiance[ray_index] = 0.;
        radiance[ray_index + 1u] = 0.;
        radiance[ray_index + 2u] = 0.;
        radiance[ray_index + 3u] = 0.;
    }

    let visibility_dimensions = textureDimensions(visibility_texture);
    let visibility_scale = vec2<f32>(visibility_dimensions) / vec2<f32>(dimensions);
    let visibility_pixel = vec2<u32>(vec2<f32>(pixel) * visibility_scale);
    let visibility_value = textureLoad(visibility_texture, visibility_pixel, 0);
    let visibility_id = visibility_value.r;

    if (visibility_id != 0u && (visibility_id & 0xFFFFFFFFu) != 0xFF000000u) {
        let depth_dimensions = textureDimensions(depth_texture);
        let depth_scale = vec2<f32>(depth_dimensions) / vec2<f32>(dimensions);
        let depth_pixel = vec2<u32>(vec2<f32>(pixel) * depth_scale);
        let depth = textureLoad(depth_texture, depth_pixel, 0);

        let hit_point = pixel_to_world(depth_pixel, depth_dimensions, depth);

        // Primary ray origin is camera pos.
        let cam_pos = (constant_data.inv_view * vec4<f32>(0., 0., 0., 1.)).xyz;
        let direction = normalize(hit_point - cam_pos);

        // Store Ray
        rays[ray_index] = cam_pos.x;
        rays[ray_index + 1u] = cam_pos.y;
        rays[ray_index + 2u] = cam_pos.z;
        let packed_direction = f32(pack2x16float(octahedral_mapping(direction)));
        rays[ray_index + 3u] = packed_direction;

        // Store Hit
        hits[ray_index] = f32(visibility_id);
        hits[ray_index + 1u] = length(hit_point - cam_pos);
    }
}
