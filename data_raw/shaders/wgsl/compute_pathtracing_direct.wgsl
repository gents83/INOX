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
var visibility_texture: texture_multisampled_2d<u32>;
@group(1) @binding(3)
var depth_texture: texture_depth_multisampled_2d;
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

    var seed = (pixel * dimensions) ^ vec2<u32>(constant_data.frame_index << 16u);

    var num_samples = 0u;
    var origin: vec3<f32>;
    var direction: vec3<f32>;
    var radiance: vec3<f32>;
    var throughput_weight: vec3<f32>;
    var first_visibility_id = 0u;

    for(var i = 0; i < 8; i++) {
        let visibility_dimensions = textureDimensions(visibility_texture);
        let visibility_scale = vec2<f32>(visibility_dimensions) / vec2<f32>(dimensions);
        let visibility_pixel = vec2<u32>(vec2<f32>(pixel) * visibility_scale);
        let visibility_value = textureLoad(visibility_texture, visibility_pixel, i);
        let visibility_id = visibility_value.r;
        if (visibility_id != 0u && (visibility_id & 0xFFFFFFFFu) != 0xFF000000u) {
            let depth_dimensions = textureDimensions(depth_texture);
            let depth_scale = vec2<f32>(depth_dimensions) / vec2<f32>(dimensions);
            let depth_pixel = vec2<u32>(vec2<f32>(pixel) * depth_scale);
            let depth = textureLoad(depth_texture, depth_pixel, i);
            let hit_point = pixel_to_world(depth_pixel, depth_dimensions, depth); 

            var pixel_data = visibility_to_gbuffer(visibility_id, hit_point);
            let material_info = compute_color_from_material(pixel_data.material_id, &pixel_data);   
            let radiance_data = compute_radiance_from_visibility(visibility_id, hit_point, get_random_numbers(&seed), vec3<f32>(0.), vec3<f32>(1.)); 
            
            origin += hit_point + radiance_data.direction * HIT_EPSILON;
            direction += radiance_data.direction;
            radiance += material_info.f_color.rgb;
            throughput_weight += radiance_data.throughput_weight;
                
            if num_samples == 0u {
                first_visibility_id = visibility_id;
            }
            num_samples = num_samples + 1u;
        }
    }
    
    if num_samples > 0u {
        let tot_samples = 1. / f32(num_samples);
        origin = origin * tot_samples;
        direction = direction  * tot_samples;
        radiance = radiance  * tot_samples;
        throughput_weight = throughput_weight  * tot_samples;
    }

    let data_index = (global_invocation_id.y * dimensions.x + global_invocation_id.x) * SIZE_OF_DATA_BUFFER_ELEMENT;
    
    data_buffer_0[data_index] = origin.x;
    data_buffer_0[data_index + 1u] = origin.y;
    data_buffer_0[data_index + 2u] = origin.z;
    let packed_direction = f32(pack2x16float(octahedral_mapping(direction)));
    data_buffer_0[data_index + 3u] = packed_direction;
    
    data_buffer_1[data_index] = radiance.x;
    data_buffer_1[data_index + 1u] = radiance.y;
    data_buffer_1[data_index + 2u] = radiance.z;
    data_buffer_1[data_index + 3u] = f32(first_visibility_id);

    data_buffer_2[data_index] = throughput_weight.x;
    data_buffer_2[data_index + 1u] = throughput_weight.y;
    data_buffer_2[data_index + 2u] = throughput_weight.z;
}