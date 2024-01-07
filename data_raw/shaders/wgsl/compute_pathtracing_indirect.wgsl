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
var<storage, read_write> radiance_data_buffer: RadianceDataBuffer;
@group(1) @binding(1)
var<storage, read_write> atomic_counters: array<atomic<u32>>;
@group(1) @binding(2)
var<storage, read> runtime_vertices: RuntimeVertices;
@group(1) @binding(3)
var<storage, read> bhv: BHV;
@group(1) @binding(4)
var radiance_texture: texture_storage_2d<rgba8unorm, read_write>;
@group(1) @binding(5)
var debug_data_texture: texture_storage_2d<r32float, read_write>;

#import "texture_utils.inc"
#import "matrix_utils.inc"
#import "geom_utils.inc"
#import "material_utils.inc"
#import "pbr_utils.inc"
#import "visibility_utils.inc"
#import "raytracing.inc"
#import "pathtracing.inc"

fn read_value_from_debug_data_texture(index: u32) -> u32 {
    let dimensions = textureDimensions(debug_data_texture);
    let v = textureLoad(debug_data_texture, vec2<u32>(index % dimensions.x, index / dimensions.x));
    return u32(v.r);
}

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


const WORKGROUP_SIZE: u32 = 8u;

@compute
@workgroup_size(WORKGROUP_SIZE, WORKGROUP_SIZE, 1)
fn main(
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let dimensions = textureDimensions(radiance_texture);
    
    let tail = atomicLoad(&atomic_counters[0u]) - 1u;
    var head = atomicAdd(&atomic_counters[1u], 1u);
    var debug_index = 0u;
    var num_bounces = 0u;
    while (head < tail) 
    {
        var buffer_data = radiance_data_buffer.data[head]; 
        var origin = buffer_data.origin;
        var direction = buffer_data.direction;
        var radiance = buffer_data.radiance;
        var throughput_weight = buffer_data.throughput_weight; 
        var seed = vec2<u32>(buffer_data.seed_x, buffer_data.seed_y);  
        let pixel_x = (buffer_data.pixel & 0xFFFF0000u) >> 16u; 
        let pixel_y = buffer_data.pixel & 0x0000FFFFu;      

        var is_pixel_to_debug = (constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_PATHTRACE) != 0;
        if (is_pixel_to_debug) {
            let debug_pixel = vec2<u32>(constant_data.debug_uv_coords * vec2<f32>(dimensions));
            is_pixel_to_debug &= debug_pixel.x == pixel_x && debug_pixel.y == pixel_y;
        }     

        let result = traverse_bvh(origin, direction, constant_data.tlas_starting_index);  
        var should_skip =  result.visibility_id == 0u || (result.visibility_id & 0xFFFFFFFFu) == 0xFF000000u;
        if (!should_skip) {
            let visibility_id = result.visibility_id;      
            let hit_point = origin + (direction * result.distance);
                
            let radiance_data = compute_radiance_from_visibility(visibility_id, hit_point, get_random_numbers(&seed), radiance, throughput_weight); 
            radiance += radiance_data.radiance;
            throughput_weight *= radiance_data.throughput_weight;
            
            if (is_pixel_to_debug) {
                debug_index = write_value_on_debug_data_texture(debug_index, f32(visibility_id));
                debug_index = write_vec3_on_debug_data_texture(debug_index, origin);
                debug_index = write_vec3_on_debug_data_texture(debug_index, direction);
            }

            buffer_data.origin = hit_point + radiance_data.direction * HIT_EPSILON;
            buffer_data.direction = radiance_data.direction;
            buffer_data.radiance = radiance;
            buffer_data.throughput_weight = throughput_weight;
            buffer_data.seed_x = seed.x;
            buffer_data.seed_y = seed.y;
            
            radiance_data_buffer.data[head] = buffer_data;
            num_bounces += 1u;
            should_skip = num_bounces >= constant_data.indirect_light_num_bounces;
        } 
        if (should_skip) {
            head = atomicAdd(&atomic_counters[1u], 1u);
            num_bounces = 0u;
        }
        
        if (is_pixel_to_debug) {
            write_value_on_debug_data_texture(0u, f32(debug_index));
        }   
    }
    
}