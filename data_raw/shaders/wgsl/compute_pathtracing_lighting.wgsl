#import "common.inc"
#import "utils.inc"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<uniform> materials: Materials;
@group(0) @binding(2)
var<uniform> textures: Textures;
@group(0) @binding(3)
var<uniform> lights: Lights;

@group(1) @binding(0)
var<storage, read> surface: array<f32>;
@group(1) @binding(1)
var<storage, read> rays: array<f32>;
@group(1) @binding(2)
var<storage, read_write> radiance: array<f32>;
@group(1) @binding(3)
var<storage, read_write> throughput: array<f32>;
@group(1) @binding(4)
var<storage, read_write> shadow_data: array<f32>;
@group(1) @binding(5)
var<storage, read_write> next_rays: array<f32>;

#import "texture_utils.inc"
#import "material_utils.inc"
#import "pbr_utils.inc"
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

    let index_4 = (global_invocation_id.y * dimensions.x + global_invocation_id.x) * SIZE_OF_DATA_BUFFER_ELEMENT;
    let index_16 = (global_invocation_id.y * dimensions.x + global_invocation_id.x) * SIZE_OF_DATA_BUFFER_ELEMENT * 4u;

    let normal = vec3<f32>(surface[index_16], surface[index_16 + 1u], surface[index_16 + 2u]);

    if (length(normal) == 0.) {
        next_rays[index_4 + 3u] = 0.;
        return;
    }

    var pixel_data: PixelData;
    pixel_data.normal = normal;
    pixel_data.tangent = vec4<f32>(surface[index_16 + 3u], surface[index_16 + 4u], surface[index_16 + 5u], surface[index_16 + 6u]);
    pixel_data.uv_set[0] = vec4<f32>(surface[index_16 + 7u], surface[index_16 + 8u], 0., 0.);
    pixel_data.material_id = u32(surface[index_16 + 9u]);
    let hit_point = vec3<f32>(surface[index_16 + 10u], surface[index_16 + 11u], surface[index_16 + 12u]);
    pixel_data.world_pos = hit_point;

    let material_info = compute_color_from_material(pixel_data.material_id, &pixel_data);

    var current_throughput = vec3<f32>(throughput[index_4], throughput[index_4 + 1u], throughput[index_4 + 2u]);

    radiance[index_4] += current_throughput.x * material_info.f_emissive.x;
    radiance[index_4 + 1u] += current_throughput.y * material_info.f_emissive.y;
    radiance[index_4 + 2u] += current_throughput.z * material_info.f_emissive.z;

    var seed = (pixel * dimensions) ^ vec2<u32>(constant_data.frame_index << 16u);
    seed = seed ^ vec2<u32>(u32(current_throughput.x * 1000.), u32(current_throughput.y * 1000.));

    let direction = sample_hemisphere(seed, pixel_data.normal);

    let d = dot(pixel_data.normal, direction);
    let new_throughput = current_throughput * (material_info.f_color.rgb * 2. * d);

    throughput[index_4] = new_throughput.x;
    throughput[index_4 + 1u] = new_throughput.y;
    throughput[index_4 + 2u] = new_throughput.z;

    let origin = hit_point + direction * HIT_EPSILON;
    next_rays[index_4] = origin.x;
    next_rays[index_4 + 1u] = origin.y;
    next_rays[index_4 + 2u] = origin.z;
    let packed_direction = f32(pack2x16float(octahedral_mapping(direction)));
    next_rays[index_4 + 3u] = packed_direction;
}
