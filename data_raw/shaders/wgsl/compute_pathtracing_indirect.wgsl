#import "common.inc"
#import "utils.inc"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> indices: Indices;
@group(0) @binding(2)
var<storage, read> vertices_attributes: VerticesAttributes;
@group(0) @binding(3)
var<storage, read> meshes: Meshes;
@group(0) @binding(4)
var<storage, read> meshlets: Meshlets;
@group(0) @binding(5)
var<storage, read> materials: Materials;
@group(0) @binding(6)
var<storage, read> textures: Textures;
@group(0) @binding(7)
var<storage, read> lights: Lights;

@group(1) @binding(0)
var<storage, read> runtime_vertices: RuntimeVertices;
@group(1) @binding(1)
var<storage, read> bhv: BHV;
@group(1) @binding(2)
var<storage, read_write> data_buffer_0: array<f32>;
@group(1) @binding(3)
var<storage, read_write> data_buffer_1: array<f32>;
@group(1) @binding(4)
var<storage, read_write> data_buffer_2: array<f32>;
@group(1) @binding(5)
var<storage, read_write> data_buffer_debug: array<f32>;

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
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let dimensions = vec2<u32>(DEFAULT_WIDTH, DEFAULT_HEIGHT);
    
    let pixel = vec2<u32>(global_invocation_id.x, global_invocation_id.y);
    if (pixel.x >= dimensions.x || pixel.y >= dimensions.y) {
        return;
    } 

    var debug_index = 1u;
    var is_pixel_to_debug = (constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_PATHTRACE) != 0;
    if (is_pixel_to_debug) {
        let debug_pixel = vec2<u32>(constant_data.debug_uv_coords * vec2<f32>(dimensions));
        is_pixel_to_debug &= debug_pixel.x == pixel.x && debug_pixel.y == pixel.y;
    }  

    let data_index = (global_invocation_id.y * dimensions.x + global_invocation_id.x) * SIZE_OF_DATA_BUFFER_ELEMENT;

    var origin = vec3<f32>(data_buffer_0[data_index], data_buffer_0[data_index + 1u], data_buffer_0[data_index + 2u]);
    let packed_direction = data_buffer_0[data_index + 3u];
    var direction = octahedral_unmapping(unpack2x16float(u32(packed_direction)));

    var radiance = vec3<f32>(data_buffer_1[data_index], data_buffer_1[data_index + 1u], data_buffer_1[data_index + 2u]);
    var visibility_id = u32(data_buffer_1[data_index + 3u]);
    
    var throughput_weight = vec3<f32>(data_buffer_2[data_index], data_buffer_2[data_index + 1u], data_buffer_2[data_index + 2u]);

    if (is_pixel_to_debug) {
        data_buffer_debug[debug_index] = f32(visibility_id);
        data_buffer_debug[debug_index + 1u] = origin.x;
        data_buffer_debug[debug_index + 2u] = origin.y;
        data_buffer_debug[debug_index + 3u] = origin.z;
        data_buffer_debug[debug_index + 4u] = direction.x;
        data_buffer_debug[debug_index + 5u] = direction.y;
        data_buffer_debug[debug_index + 6u] = direction.z;
        debug_index = debug_index + 7u;
    }

    var should_skip = visibility_id == 0u;
    var seed = (pixel * dimensions) ^ vec2<u32>(constant_data.frame_index << 16u);
    var bounce = 0u;
    for(bounce = 0u; !should_skip && bounce < constant_data.indirect_light_num_bounces; bounce++)
    {
        let result = traverse_bvh(origin, direction, constant_data.tlas_starting_index);  
        should_skip = result.visibility_id == 0u || (result.visibility_id & 0xFFFFFFFFu) == 0xFF000000u;
        if (should_skip) { 
            //hit the sky
            //radiance += vec3<f32>(0.03);
            break;
        }  
        let hit_point = origin + (direction * result.distance);
            
        let radiance_data = compute_radiance_from_visibility(result.visibility_id, hit_point, get_random_numbers(&seed), radiance, throughput_weight); 
        origin = hit_point + radiance_data.direction * HIT_EPSILON;
        direction = radiance_data.direction;
        radiance += radiance_data.radiance;
        throughput_weight *= radiance_data.throughput_weight;

        if (is_pixel_to_debug) {
            data_buffer_debug[debug_index] = f32(result.visibility_id);
            data_buffer_debug[debug_index + 1u] = origin.x;
            data_buffer_debug[debug_index + 2u] = origin.y;
            data_buffer_debug[debug_index + 3u] = origin.z;
            data_buffer_debug[debug_index + 4u] = direction.x;
            data_buffer_debug[debug_index + 5u] = direction.y;
            data_buffer_debug[debug_index + 6u] = direction.z;
            debug_index = debug_index + 7u;
        }
    }
    if (is_pixel_to_debug) {
        data_buffer_debug[0u] = f32(debug_index);
    } 
    if(bounce > 0u) {
        data_buffer_1[data_index] = radiance.x;
        data_buffer_1[data_index + 1u] = radiance.y;
        data_buffer_1[data_index + 2u] = radiance.z;
    }
}