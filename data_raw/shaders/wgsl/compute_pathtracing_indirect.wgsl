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
var radiance_texture: texture_storage_2d<rgba32float, read_write>;
@group(1) @binding(4)
var ray_texture: texture_storage_2d<rgba32float, read>;
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

fn execute_job(job_index: u32, pixel: vec2<u32>, dimensions: vec2<u32>) -> vec4<f32>  
{    
    let radiance_value = textureLoad(radiance_texture, pixel);
    
    let radiance_rg = unpack2x16float(u32(radiance_value.r));
    let radiance_b_throughput_weight_r = unpack2x16float(u32(radiance_value.g));
    let throughput_weight_gb = unpack2x16float(u32(radiance_value.b));
    var throughput_weight = vec3<f32>(radiance_b_throughput_weight_r.y, throughput_weight_gb.x, throughput_weight_gb.y); 
    var radiance = vec3<f32>(radiance_rg.x, radiance_rg.y, radiance_b_throughput_weight_r.x) * throughput_weight;

    let visibility_id = u32(radiance_value.a);
    if (visibility_id == 0u || (visibility_id & 0xFFFFFFFFu) == 0xFF000000u) {
        return vec4<f32>(radiance, 1.);
    }  
 
    let rays_data = textureLoad(ray_texture, pixel);
    let direction = rays_data.rgb;
    let depth = rays_data.a;
    let origin = pixel_to_world(pixel, dimensions, depth) + direction * HIT_EPSILON;
    var ray = Ray(origin, 0., direction, MAX_TRACING_DISTANCE);

    var seed = (pixel * dimensions) ^ vec2<u32>(constant_data.frame_index << 16u);
    
    for(var i = 0u; i < constant_data.indirect_light_num_bounces; i++) {
        let result = traverse_bvh(ray, constant_data.tlas_starting_index);  
        if (result.visibility_id == 0u || (result.visibility_id & 0xFFFFFFFFu) == 0xFF000000u) {
            break;
        }
        
        let hit_point = ray.origin + (ray.direction * result.distance);
        let radiance_data = compute_radiance_from_visibility(result.visibility_id, hit_point, get_random_numbers(&seed), radiance, throughput_weight); 
        
        ray = Ray(hit_point + radiance_data.direction * HIT_EPSILON, 0., radiance_data.direction, MAX_TRACING_DISTANCE);
        radiance = radiance_data.radiance;
        throughput_weight = radiance_data.throughput_weight;
    }
    return vec4<f32>(radiance, 1.);
}



const MAX_WORKGROUP_SIZE: u32 = 16u*16u;
var<workgroup> jobs_count: atomic<u32>;

@compute
@workgroup_size(16, 16, 1)
fn main(
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    let dimensions = textureDimensions(radiance_texture);
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
        textureStore(radiance_texture, pixel, out_color);

        job_index = MAX_WORKGROUP_SIZE - atomicSub(&jobs_count, 1u);
    }
}