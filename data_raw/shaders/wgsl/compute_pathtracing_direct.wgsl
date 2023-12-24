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
var ray_texture: texture_storage_2d<rgba32float, read_write>;
@group(1) @binding(5)
var visibility_texture: texture_2d<f32>;
@group(1) @binding(6)
var depth_texture: texture_depth_2d;

#import "texture_utils.inc"
#import "matrix_utils.inc"
#import "geom_utils.inc"
#import "material_utils.inc"
#import "pbr_utils.inc"
#import "visibility_utils.inc"
#import "raytracing.inc"
#import "pathtracing.inc"

fn execute_job(job_index: u32, pixel: vec2<u32>, dimensions: vec2<u32>) -> vec4<f32>  
{        
    let visibility_value = textureLoad(visibility_texture, pixel, 0);
    let visibility_id = pack4x8unorm(visibility_value);
    if (visibility_id == 0u || (visibility_id & 0xFFFFFFFFu) == 0xFF000000u) {
        return vec4<f32>(0.);
    }

    var seed = (pixel * dimensions) ^ vec2<u32>(constant_data.frame_index * 0xFFFFu);
    seed = get_random_numbers(seed); 
    
    let depth_value = textureLoad(depth_texture, pixel, 0);
    let hit_point = pixel_to_world(pixel, dimensions, depth_value);
    let radiance_data = compute_radiance_from_visibility(visibility_id, hit_point, vec2<f32>(seed), vec3<f32>(0.), vec3<f32>(1.));     
    
    let origin = hit_point + radiance_data.direction * HIT_EPSILON;
    let direction = normalize(radiance_data.direction);

    let rr = direction.x;
    let rg = direction.y;
    let rb = direction.z;
    let ra = depth_value;
    let out_rays_color = vec4<f32>(f32(rr), f32(rg), f32(rb), f32(ra));
    textureStore(ray_texture, pixel, out_rays_color);
        
    let r = pack2x16float(radiance_data.radiance.rg);
    let g = pack2x16float(vec2<f32>(radiance_data.radiance.b, radiance_data.throughput_weight.r));
    let b = pack2x16float(radiance_data.throughput_weight.gb);
    let a = visibility_id;
    
    return vec4<f32>(f32(r), f32(g), f32(b), f32(a));
}



const MAX_WORKGROUP_SIZE: u32 = 16u*16u;
var<workgroup> jobs_count: atomic<u32>;

@compute
@workgroup_size(16, 16, 1)
fn main(
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    let dimensions = textureDimensions(render_target);
    atomicStore(&jobs_count, MAX_WORKGROUP_SIZE);
    
    var job_index = 0u;
    while(job_index < MAX_WORKGROUP_SIZE)
    {
        let pixel = vec2<u32>(workgroup_id.x * 16u + job_index % 16u, 
                              workgroup_id.y * 16u + job_index / 16u);
        if (pixel.x >= dimensions.x || pixel.y >= dimensions.y) {
            job_index = MAX_WORKGROUP_SIZE - atomicSub(&jobs_count, 1u);
            continue;
        }    
        
        let index = u32(pixel.y * dimensions.x + pixel.x);

        var out_color = execute_job(index, pixel, dimensions);
        textureStore(render_target, pixel, out_color);
        job_index = MAX_WORKGROUP_SIZE - atomicSub(&jobs_count, 1u);
    }
}