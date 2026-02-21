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
var<storage, read_write> data_buffer_0: array<RayPackedData>;
@group(1) @binding(5)
var<storage, read_write> data_buffer_1: array<RadiancePackedData>;
@group(1) @binding(6)
var<storage, read_write> data_buffer_2: array<ThroughputPackedData>;

#import "texture_utils.inc"
#import "matrix_utils.inc"
#import "geom_utils.inc"
#import "material_utils.inc"
#import "pbr_utils.inc"
#import "visibility_utils.inc"
#import "raytracing.inc"
#import "pathtracing.inc"
#import "pathtracing_tracing.inc"

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

    // Check if this ray is still active
    if (!read_ray_active(data_index)) {
        return;
    }

    // Read hit results from traversal
    let hit_point = read_hit_position(data_index);
    let visibility_id = read_hit_visibility_id(data_index);

    // Read current accumulated radiance
    var radiance = read_radiance(data_index);
    var throughput = read_throughput(data_index);

    var seed = (pixel * dimensions) ^ vec2<u32>((constant_data.frame_index + 7u) << 16u);

    // Miss — sample sky / environment
    if (visibility_id == 0u || (visibility_id & 0xFFFFFFFFu) == 0xFF000000u) {
        // Read the direction stored by traversal for env map sampling
        let packed_dir = read_incoming_direction(data_index);

        // Sky contribution — simple ambient for now
        let sky_color = vec3<f32>(0.03);
        radiance += throughput * sky_color;

        // Terminate ray
        write_radiance(data_index, radiance, 0u);
        write_throughput(data_index, vec3<f32>(0.), false);
        return;
    }

    // Hit — shade the intersection

    // Reconstruct G-buffer at hit point
    var pixel_data = visibility_to_gbuffer(visibility_id, hit_point);
    
    // Evaluate full PBR material at hit with shadows

    // === Shadow rays ===
    var shadow_mask = 0u;
    let num_lights_to_process = min(constant_data.num_lights, 32u);
    for (var light_i = 0u; light_i < num_lights_to_process; light_i++) {
        let light = lights.data[light_i];
        if (light.light_type == LIGHT_TYPE_INVALID) {
            shadow_mask |= (1u << light_i);
            continue;
        }

        var to_light: vec3<f32>;
        var max_distance: f32;
        if (light.light_type == LIGHT_TYPE_DIRECTIONAL) {
            to_light = -light.direction;
            max_distance = MAX_TRACING_DISTANCE;
        } else {
            to_light = light.position - hit_point;
            max_distance = length(to_light);
        }

        let shadow = trace_shadow_ray(hit_point + pixel_data.normal * HIT_EPSILON, to_light, max_distance, constant_data.tlas_starting_index);
        if (shadow > 0.5) {
            shadow_mask |= (1u << light_i);
        }
    }
    // Fill remaining bits as lit
    for (var i = num_lights_to_process; i < 32u; i++) {
        shadow_mask |= (1u << i);
    }

    var material_info = compute_color_from_material(pixel_data.material_id, &pixel_data, shadow_mask);

    // Add emissive contribution
    radiance += throughput * material_info.f_emissive;

    // Add direct lighting at hit (diffuse + specular from PBR evaluation)
    let direct = material_info.f_diffuse + material_info.f_specular;
    radiance += throughput * direct;

    // The incoming ray direction was stored by compute_ray_traversal.wgsl
    // The view direction for this bounce is the negated incoming ray
    let incoming_dir = read_incoming_direction(data_index);
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
    // Here we just check throughput
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
