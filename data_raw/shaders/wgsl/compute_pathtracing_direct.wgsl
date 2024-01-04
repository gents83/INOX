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
var<storage, read> culling_result: array<u32>;
@group(1) @binding(2)
var<storage, read> bhv: BHV;

@group(1) @binding(3)
var render_target: texture_storage_2d<rgba32float, read_write>;
@group(1) @binding(4)
var visibility_texture: texture_2d<f32>;
@group(1) @binding(5)
var depth_texture: texture_depth_2d;

#import "texture_utils.inc"
#import "matrix_utils.inc"
#import "geom_utils.inc"
#import "material_utils.inc"
#import "pbr_utils.inc"
#import "visibility_utils.inc"
#import "pathtracing.inc"

fn execute_job(source_pixel: vec2<u32>, source_dimensions: vec2<u32>) -> vec4<f32>  
{        
    let visibility_value = textureLoad(visibility_texture, source_pixel, 0);
    let visibility_id = pack4x8unorm(visibility_value);
    if (visibility_id == 0u || (visibility_id & 0xFFFFFFFFu) == 0xFF000000u) {
        return vec4<f32>(0.);
    }
    
    let depth = textureLoad(depth_texture, source_pixel, 0);
    let hit_point = pixel_to_world(source_pixel, source_dimensions, depth); 
    var pixel_data = visibility_to_gbuffer(visibility_id, hit_point);
    let material_info = compute_color_from_material(pixel_data.material_id, &pixel_data);   
    
    let out_gbuffer = vec4<f32>(material_info.base_color.rgb, 
                                f32(visibility_id));
    
    return vec4<f32>(out_gbuffer);
}



const WORKGROUP_SIZE: u32 = 4u;

@compute
@workgroup_size(WORKGROUP_SIZE, WORKGROUP_SIZE, 1)
fn main(
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    let target_dimensions = textureDimensions(render_target);
    let source_dimensions = textureDimensions(visibility_texture);
    let scale = vec2<f32>(source_dimensions) / vec2<f32>(target_dimensions);

    let target_pixel = vec2<u32>(workgroup_id.x * WORKGROUP_SIZE + local_invocation_id.x, 
                                 workgroup_id.y * WORKGROUP_SIZE + local_invocation_id.y);
    if (target_pixel.x > target_dimensions.x || target_pixel.y > target_dimensions.y) {
        return;
    }    
    
    let source_pixel = vec2<u32>(vec2<f32>(target_pixel) * scale);
    var out_color = execute_job(source_pixel, source_dimensions);
    textureStore(render_target, target_pixel, out_color);
    
}