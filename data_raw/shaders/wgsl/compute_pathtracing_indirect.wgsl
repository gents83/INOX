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
var<storage, read> vertices_positions: VerticesPositions;
@group(1) @binding(1)
var<storage, read> runtime_vertices: RuntimeVertices;
@group(1) @binding(2)
var<storage, read> bhv: BHV;

@group(1) @binding(3)
var radiance_texture: texture_storage_2d<rgba32float, read_write>;
@group(1) @binding(4)
var visibility_texture: texture_2d<f32>;
@group(1) @binding(5)
var depth_texture: texture_depth_2d;
@group(1) @binding(6)
var debug_data_texture: texture_storage_2d<r32float, read_write>;

#import "texture_utils.inc"
#import "matrix_utils.inc"
#import "geom_utils.inc"
#import "material_utils.inc"
#import "pbr_utils.inc"
#import "visibility_utils.inc"
#import "raytracing.inc"
#import "pathtracing.inc"

fn write_value_on_debug_data_texture(index: u32, value: f32) -> u32 {
    let dimensions = textureDimensions(debug_data_texture);
    textureStore(debug_data_texture, vec2<u32>(index % dimensions.x, index / dimensions.x), vec4<f32>(value, 0., 0., 1.));
    return index + 1u;
}

fn write_vec3_on_debug_data_texture(index: u32, v: vec3<f32>) -> u32 {
    let dimensions = textureDimensions(debug_data_texture);
    var new_index = index;
    textureStore(debug_data_texture, vec2<u32>(new_index % dimensions.x, new_index / dimensions.x), vec4<f32>(v.x, 0., 0., 1.));
    new_index += 1u;
    textureStore(debug_data_texture, vec2<u32>(new_index % dimensions.x, new_index / dimensions.x), vec4<f32>(v.y, 0., 0., 1.));
    new_index += 1u;
    textureStore(debug_data_texture, vec2<u32>(new_index % dimensions.x, new_index / dimensions.x), vec4<f32>(v.z, 0., 0., 1.));
    new_index += 1u;
    return new_index;
}

fn execute_job(pixel: vec2<u32>, dimensions: vec2<u32>) -> vec4<f32>  
{    
    var is_pixel_to_debug = (constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_PATHTRACE) != 0;
    if (is_pixel_to_debug) {
        let debug_pixel = vec2<u32>(constant_data.debug_uv_coords * vec2<f32>(dimensions));
        is_pixel_to_debug &= debug_pixel.x == pixel.x && debug_pixel.y == pixel.y;
    } 

    var radiance = vec3<f32>(0.);

    let visibility_value = textureLoad(visibility_texture, pixel, 0);
    let visibility_id = pack4x8unorm(visibility_value);
    if (visibility_id == 0u || (visibility_id & 0xFFFFFFFFu) == 0xFF000000u) {
        
        if (is_pixel_to_debug) {
            write_value_on_debug_data_texture(0u, 0.);
        }
        return vec4<f32>(radiance, 1.);
    }       

    var throughput_weight = vec3<f32>(1.); 
    var seed = (pixel * dimensions) ^ vec2<u32>(constant_data.frame_index << 16u);

    let depth = textureLoad(depth_texture, pixel, 0);
    var hit_point = pixel_to_world(pixel, dimensions, depth);
    var radiance_data = compute_radiance_from_visibility(visibility_id, hit_point, get_random_numbers(&seed), radiance, throughput_weight);  

    var ray = Ray(hit_point + radiance_data.direction * HIT_EPSILON, 0., radiance_data.direction, MAX_TRACING_DISTANCE);

    var debug_index = 1u;
    if (is_pixel_to_debug) {
        debug_index = write_value_on_debug_data_texture(debug_index, f32(visibility_id));
        debug_index = write_vec3_on_debug_data_texture(debug_index, ray.origin);
        debug_index = write_vec3_on_debug_data_texture(debug_index, ray.direction);
    }
    
    for(var i = 0u; i < constant_data.indirect_light_num_bounces; i++) {
        let result = traverse_bvh(ray, constant_data.tlas_starting_index, is_pixel_to_debug, 100u);  
        if (result.visibility_id == 0u || (result.visibility_id & 0xFFFFFFFFu) == 0xFF000000u) {
            break;
        }
        
        hit_point = ray.origin + (ray.direction * result.distance);
        radiance_data = compute_radiance_from_visibility(result.visibility_id, hit_point, get_random_numbers(&seed), radiance, throughput_weight); 
        
        ray = Ray(hit_point + radiance_data.direction * HIT_EPSILON, 0., radiance_data.direction, MAX_TRACING_DISTANCE);
        radiance = radiance_data.radiance;
        throughput_weight = radiance_data.throughput_weight;
        
        if (is_pixel_to_debug) {
            debug_index = write_value_on_debug_data_texture(debug_index, f32(result.visibility_id));
            debug_index = write_vec3_on_debug_data_texture(debug_index, ray.origin);
            debug_index = write_vec3_on_debug_data_texture(debug_index, ray.direction);
        }
    }

    if (is_pixel_to_debug) {
        write_value_on_debug_data_texture(0u, f32(debug_index));
    }
    return vec4<f32>(radiance, 1.);
}



const WORKGROUP_SIZE: u32 = 4u;

@compute
@workgroup_size(WORKGROUP_SIZE, WORKGROUP_SIZE, 1)
fn main(
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    let dimensions = textureDimensions(radiance_texture);
    
    let pixel = vec2<u32>(workgroup_id.x * WORKGROUP_SIZE + local_invocation_id.x, 
                            workgroup_id.y * WORKGROUP_SIZE + local_invocation_id.y);
    if (pixel.x > dimensions.x || pixel.y > dimensions.y) {
        return;
    }    

    var out_color = execute_job(pixel, dimensions);
    textureStore(radiance_texture, pixel, out_color);   
}