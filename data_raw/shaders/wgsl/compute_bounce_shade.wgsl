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
var<uniform> materials: Materials;
@group(1) @binding(1)
var<uniform> textures: Textures;
@group(1) @binding(2)
var<uniform> lights: Lights;
@group(1) @binding(3)
var<storage, read> bvh: BVH;
@group(1) @binding(4)
var<storage, read_write> data_buffer_0: array<f32>;
@group(1) @binding(5)
var<storage, read_write> data_buffer_1: array<f32>;
@group(1) @binding(6)
var<storage, read_write> data_buffer_2: array<f32>;

#import "texture_utils.inc"
#import "matrix_utils.inc"
#import "geom_utils.inc"
#import "material_utils.inc"
#import "pbr_utils.inc"
#import "visibility_utils.inc"
#import "raytracing.inc"
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

    let data_index = (global_invocation_id.y * dimensions.x + global_invocation_id.x) * SIZE_OF_DATA_BUFFER_ELEMENT;

    // Check if this ray is still active
    if (!read_ray_active(data_index)) {
        return;
    }

    // Read hit results from traversal
    let hit_point = vec3<f32>(
        data_buffer_0[data_index],
        data_buffer_0[data_index + 1u],
        data_buffer_0[data_index + 2u]
    );
    let hit_or_vis = data_buffer_0[data_index + 3u];

    // Read current accumulated radiance
    var radiance = read_radiance(data_index);
    var throughput = read_throughput(data_index);

    var seed = (pixel * dimensions) ^ vec2<u32>((constant_data.frame_index + 7u) << 16u);

    // Miss — sample sky / environment
    if (hit_or_vis < 0.) {
        // Read the direction stored by traversal for env map sampling
        let packed_dir = data_buffer_1[data_index + 3u];
        // Sky contribution — simple ambient for now
        // TODO: sample environment map using constant_data.environment_map_texture_index
        let sky_color = vec3<f32>(0.03);
        radiance += throughput * sky_color;

        // Terminate ray
        write_radiance(data_index, radiance, 0u);
        write_throughput(data_index, vec3<f32>(0.), false);
        return;
    }

    // Hit — shade the intersection
    let visibility_id = u32(hit_or_vis);
    if (visibility_id == 0u || (visibility_id & 0xFFFFFFFFu) == 0xFF000000u) {
        write_throughput(data_index, vec3<f32>(0.), false);
        return;
    }

    // Reconstruct G-buffer at hit point
    var pixel_data = visibility_to_gbuffer(visibility_id, hit_point);
    
    // Evaluate full PBR material at hit
    var material_info = compute_color_from_material(pixel_data.material_id, &pixel_data, 0xFFFFFFFFu);

    // Add emissive contribution
    radiance += throughput * material_info.f_emissive;

    // Add direct lighting at hit (diffuse + specular from PBR evaluation)
    let direct = material_info.f_diffuse + material_info.f_specular;
    radiance += throughput * direct;

    // The incoming ray direction was stored by compute_ray_traversal.wgsl
    // The view direction for this bounce is the negated incoming ray
    let incoming_dir = octahedral_unmapping(unpack2x16float(u32(data_buffer_1[data_index + 3u])));
    let view = -incoming_dir;

    // Sample next bounce direction
    let brdf_sample = sample_brdf(
        pixel_data.normal, view,
        material_info.perceptual_roughness,
        material_info.metallic,
        &seed
    );

    let NdotL = max(dot(pixel_data.normal, brdf_sample.direction), 0.);
    let brdf_eval = eval_brdf(&material_info, pixel_data.normal, view, brdf_sample.direction, brdf_sample.is_specular);
    var new_throughput = vec3<f32>(0.);
    if (brdf_sample.pdf > 0.) {
        new_throughput = throughput * brdf_eval * NdotL / brdf_sample.pdf;
    }

    // Russian roulette (after first 2 bounces — controlled by frame logic)
    var is_active = NdotL > 0. && length(new_throughput) > MATH_EPSILON;
    if (is_active && length(new_throughput) < 0.5) {
        if (russian_roulette(new_throughput, &seed)) {
            is_active = false;
        } else {
            new_throughput = russian_roulette_weight(new_throughput);
        }
    }

    // Write next bounce ray
    let next_origin = hit_point + brdf_sample.direction * HIT_EPSILON;
    write_ray_data(data_index, next_origin, brdf_sample.direction);

    // Write updated radiance
    write_radiance(data_index, radiance, visibility_id);

    // Write updated throughput
    write_throughput(data_index, new_throughput, is_active);
}
