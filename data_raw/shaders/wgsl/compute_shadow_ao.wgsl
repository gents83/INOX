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
var<storage, read> transforms: Transforms;
@group(0) @binding(5)
var<storage, read> meshes: Meshes;
@group(0) @binding(6)
var<storage, read> meshlets: Meshlets;
@group(0) @binding(7)
var<storage, read> bvh: BVH;

@group(1) @binding(0)
var<uniform> lights: Lights;
@group(1) @binding(1)
var<storage, read_write> data_buffer_0: array<RayPackedData>;
@group(1) @binding(2)
var<storage, read_write> data_buffer_1: array<RadiancePackedData>;
@group(1) @binding(3)
var<storage, read_write> data_buffer_2: array<ThroughputPackedData>;

#import "matrix_utils.inc"
#import "geom_utils.inc"
#import "raytracing.inc"
#import "pathtracing.inc"

#import "pathtracing_tracing.inc"

const WORKGROUP_SIZE: u32 = 8u;
const AO_NUM_RAYS: u32 = 4u;
const AO_RADIUS: f32 = 0.5;

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

    // Read G-buffer
    let world_pos = read_gbuffer_world_pos(data_index);
    let normal = read_gbuffer_normal(data_index);
    let material_id = read_gbuffer_material_id(data_index);

    // Skip empty pixels
    if (length(normal) < 0.01) {
        write_shadow_mask(data_index, 0xFFFFFFFFu); // all lit
        write_ao_factor(data_index, 1.0);
        return;
    }

    var seed = (pixel * dimensions) ^ vec2<u32>(constant_data.frame_index << 16u);

    // === Shadow rays ===
    var shadow_mask = 0u;
    let num_lights_to_process = min(constant_data.num_lights, 32u);
    for (var light_i = 0u; light_i < num_lights_to_process; light_i++) {
        let light = lights.data[light_i];
        if (light.light_type == LIGHT_TYPE_INVALID) {
            shadow_mask |= (1u << light_i); // mark invalid lights as "lit"
            continue;
        }

        var to_light: vec3<f32>;
        var max_distance: f32;
        if (light.light_type == LIGHT_TYPE_DIRECTIONAL) {
            to_light = -light.direction;
            max_distance = MAX_TRACING_DISTANCE;
        } else {
            to_light = light.position - world_pos;
            max_distance = length(to_light);
        }

        let shadow = trace_shadow_ray(world_pos + normal * HIT_EPSILON, to_light, max_distance, constant_data.tlas_starting_index);
        if (shadow > 0.5) {
            shadow_mask |= (1u << light_i);
        }
    }
    // Fill remaining bits as lit
    for (var i = num_lights_to_process; i < 32u; i++) {
        shadow_mask |= (1u << i);
    }

    write_shadow_mask(data_index, shadow_mask);

    // === AO rays ===
    var ao_value = 0.0;
    for (var ao_i = 0u; ao_i < AO_NUM_RAYS; ao_i++) {
        let random = get_random_numbers(&seed);
        let ao_dir = sample_cosine_wheighted_hemisphere(random, normal);
        ao_value += trace_ao_ray(world_pos + normal * HIT_EPSILON, ao_dir, AO_RADIUS, constant_data.tlas_starting_index);
    }
    ao_value /= f32(AO_NUM_RAYS);

    write_ao_factor(data_index, ao_value);
}
