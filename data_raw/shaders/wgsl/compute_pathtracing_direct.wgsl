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
var<storage, read_write> radiance_data_buffer: RadianceDataBuffer;
@group(1) @binding(2)
var<storage, read_write> indirect_dispatch_size: DispatchCommandSize;
@group(1) @binding(3)
var<storage, read> bhv: BHV;

@group(1) @binding(4)
var visibility_texture: texture_storage_2d<rgba8unorm, read_write>;
@group(1) @binding(5)
var depth_texture: texture_depth_2d;

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
    if(global_invocation_id.x == 0u && global_invocation_id.y == 0u) {
        atomicStore(&indirect_dispatch_size.x, 0u);
        atomicStore(&indirect_dispatch_size.y, 1u);
        atomicStore(&indirect_dispatch_size.z, 1u);
    }
    let dimensions = textureDimensions(visibility_texture);

    let pixel = vec2<u32>(global_invocation_id.x, global_invocation_id.y);
    if (pixel.x >= dimensions.x || pixel.y >= dimensions.y) {
        return;
    }     

    var seed = (pixel * dimensions) ^ vec2<u32>(constant_data.frame_index << 16u);
    var index = 0u;

    let visibility_value = textureLoad(visibility_texture, pixel);
    let visibility_id = pack4x8unorm(visibility_value);
    if (visibility_id != 0u && (visibility_id & 0xFFFFFFFFu) != 0xFF000000u) {
        let depth_dimensions = textureDimensions(depth_texture);
        let depth_scale = vec2<f32>(depth_dimensions) / vec2<f32>(dimensions);
        let depth_pixel = vec2<u32>(vec2<f32>(pixel) * depth_scale);
        let depth = textureLoad(depth_texture, depth_pixel, 0);
        let hit_point = pixel_to_world(depth_pixel, depth_dimensions, depth); 

        var pixel_data = visibility_to_gbuffer(visibility_id, hit_point);
        let material_info = compute_color_from_material(pixel_data.material_id, &pixel_data);   
        let radiance_data = compute_radiance_from_visibility(visibility_id, hit_point, get_random_numbers(&seed), vec3<f32>(0.), vec3<f32>(1.)); 
        
        index = atomicAdd(&indirect_dispatch_size.x, 1u);
        radiance_data_buffer.data[index].origin = hit_point + radiance_data.direction * HIT_EPSILON;
        radiance_data_buffer.data[index].direction = radiance_data.direction;
        radiance_data_buffer.data[index].radiance = material_info.base_color.rgb + radiance_data.radiance;
        radiance_data_buffer.data[index].throughput_weight = radiance_data.throughput_weight;
        radiance_data_buffer.data[index].seed_x = seed.x;
        radiance_data_buffer.data[index].seed_y = seed.y;
        radiance_data_buffer.data[index].pixel = ((pixel.x & 0x0000FFFFu) << 16u) | pixel.y;
        radiance_data_buffer.data[index].visibility_id = visibility_id;    
        index += 1u;
    }
    textureStore(visibility_texture, pixel, unpack4x8unorm(index));
}