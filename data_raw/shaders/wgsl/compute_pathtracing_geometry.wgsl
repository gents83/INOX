#import "common.inc"
#import "utils.inc"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> indices: Indices;
@group(0) @binding(2)
var<storage, read> vertices_attributes: VerticesAttributes;
@group(0) @binding(3)
var<storage, read> instances: Instances;
@group(0) @binding(4)
var<storage, read> meshlets: Meshlets;

@group(1) @binding(0)
var<storage, read> rays: array<f32>;
@group(1) @binding(1)
var<storage, read> hits: array<f32>;
@group(1) @binding(2)
var<storage, read_write> surface: array<f32>;

#import "matrix_utils.inc"
#import "geom_utils.inc"
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

    let index_4 = (global_invocation_id.y * dimensions.x + global_invocation_id.x) * SIZE_OF_DATA_BUFFER_ELEMENT;
    let index_16 = (global_invocation_id.y * dimensions.x + global_invocation_id.x) * SIZE_OF_DATA_BUFFER_ELEMENT * 4u;

    let visibility_id = u32(hits[index_4]);
    let t = hits[index_4 + 1u];

    if (visibility_id == 0u) {
        surface[index_16] = 0.;
        return;
    }

    let origin = vec3<f32>(rays[index_4], rays[index_4 + 1u], rays[index_4 + 2u]);
    let packed_direction = rays[index_4 + 3u];
    let direction = octahedral_unmapping(unpack2x16float(u32(packed_direction)));

    let hit_point = origin + direction * t;

    let pixel_data = visibility_to_gbuffer(visibility_id, hit_point);

    surface[index_16] = pixel_data.normal.x;
    surface[index_16 + 1u] = pixel_data.normal.y;
    surface[index_16 + 2u] = pixel_data.normal.z;
    surface[index_16 + 3u] = pixel_data.tangent.x;

    surface[index_16 + 4u] = pixel_data.tangent.y;
    surface[index_16 + 5u] = pixel_data.tangent.z;
    surface[index_16 + 6u] = pixel_data.tangent.w;
    surface[index_16 + 7u] = pixel_data.uv_set[0].x;

    surface[index_16 + 8u] = pixel_data.uv_set[0].y;
    surface[index_16 + 9u] = f32(pixel_data.material_id);
    surface[index_16 + 10u] = hit_point.x;
    surface[index_16 + 11u] = hit_point.y;
    surface[index_16 + 12u] = hit_point.z;
}
