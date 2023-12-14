#import "common.inc"
#import "utils.inc"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> indices: Indices;
@group(0) @binding(2)
var<storage, read> runtime_vertices: RuntimeVertices;
@group(0) @binding(3)
var<storage, read> vertices_attributes: VerticesAttributes;
@group(0) @binding(4)
var<storage, read> meshes: Meshes;
@group(0) @binding(5)
var<storage, read> meshlets: Meshlets;
@group(0) @binding(6)
var<storage, read> culling_result: array<u32>;

@group(1) @binding(0)
var<storage, read> meshes_inverse_matrix: Matrices;
@group(1) @binding(1)
var<storage, read> materials: Materials;
@group(1) @binding(2)
var<storage, read> textures: Textures;
@group(1) @binding(3)
var<storage, read> bhv: BHV;
@group(1) @binding(4)
var<uniform> tlas_starting_index: u32;
@group(1) @binding(5)
var<storage, read_write> rays: Rays;
@group(1) @binding(6)
var render_target: texture_storage_2d<rgba8unorm, read_write>;

#import "texture_utils.inc"
#import "matrix_utils.inc"
#import "geom_utils.inc"
#import "material_utils.inc"
#import "visibility_utils.inc"
#import "raytracing.inc"
#import "pathtracing.inc"


fn compute_visibility_from_traversal(ray: ptr<function, Ray>) -> u32 
{
    var nearest = (*ray).t_max;  
    var visibility_id = 0u;        
    var tlas_index = 0;

    while (tlas_index >= 0)
    {
        let node = &bhv.data[tlas_starting_index + u32(tlas_index)];    
        let intersection = intersect_aabb(ray, (*node).min, (*node).max);
        if (intersection > nearest) {
            tlas_index = (*node).miss;
            continue;
        }
        if ((*node).reference < 0) {
            //inner node
            tlas_index = tlas_index + 1;
            continue;
        }
        //leaf node
        let mesh_id = u32((*node).reference);
        let inverse_matrix = &meshes_inverse_matrix.data[mesh_id];    
        let transformed_origin = (*inverse_matrix) * vec4<f32>((*ray).origin, 1.);
        let transformed_direction = (*inverse_matrix) * vec4<f32>((*ray).direction, 0.);
        var transformed_ray = Ray(transformed_origin.xyz, (*ray).t_min, transformed_direction.xyz, (*ray).t_max);
        let result = traverse_bhv_of_meshlets(ray, &transformed_ray, mesh_id, nearest);
        visibility_id = select(visibility_id, result.visibility_id, result.distance < nearest);
        nearest = min(nearest, result.distance);
        tlas_index = (*node).miss;
    } 
    (*ray).t_max = nearest;
    return visibility_id;
}

fn execute_job(job_index: u32, pixel: vec2<f32>, dimensions: vec2<f32>) -> vec4<f32>  
{    
    var ray = rays.data[job_index];
    var seed = vec2<u32>(pixel * dimensions) ^ vec2<u32>(constant_data.frame_index << 16u);
    var uv_coords = 2. * (pixel / dimensions) - vec2<f32>(1., 1.);
    uv_coords.y = -uv_coords.y;
    var pixel_color = vec3<f32>(0.);
    for (var sample = 0u; sample < NUM_SAMPLES_PER_PIXEL; sample++) {
        var radiance_data = RadianceData(ray, seed, vec3<f32>(0.), vec3<f32>(1.));
        for (var bounce = 0u; bounce < MAX_PATH_BOUNCES; bounce++) {
            let result = traverse_bvh(radiance_data.ray, i32(tlas_starting_index));
            if (result.visibility_id == 0u) {
                break;
            }
            radiance_data.ray.t_max = result.distance;
            radiance_data = compute_radiance_from_visibility(result.visibility_id, uv_coords, radiance_data);
            seed = radiance_data.seed;
        }
        pixel_color += radiance_data.radiance;
    }
    pixel_color /= f32(NUM_SAMPLES_PER_PIXEL);
    return vec4<f32>(pixel_color, 1.);
}


const MAX_WORKGROUP_SIZE: i32 = 16*16;
const MAX_SIZE: u32 = u32(MAX_WORKGROUP_SIZE);

var<workgroup> jobs_count: atomic<i32>;
//var<workgroup> rays_data: array<Ray, MAX_SIZE>;
//var<workgroup> visibility_data: array<u32, MAX_SIZE>;
//var<workgroup> radiance_data: array<RadianceData, MAX_SIZE>;

@compute
@workgroup_size(16, 16, 1)
fn main(
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    let max_jobs = 16 * 16;
    let group = vec2<i32>(i32(workgroup_id.x), i32(workgroup_id.y));
    let dimensions = vec2<i32>(textureDimensions(render_target));
    atomicStore(&jobs_count, max_jobs);
    
    var job_index = 0;
    while(job_index < MAX_WORKGROUP_SIZE)
    {
        let pixel = vec2<i32>(group.x * 16 + job_index % 16, 
                              group.y * 16 + job_index / 16);
        if (pixel.x >= dimensions.x || pixel.y >= dimensions.y) {
            job_index = max_jobs - atomicSub(&jobs_count, 1);
            continue;
        }    
        
        let index = u32(pixel.y * dimensions.x + pixel.x);

        var out_color = execute_job(index, vec2<f32>(pixel), vec2<f32>(dimensions));
        if(constant_data.frame_index > 0u) {
            var prev_value = textureLoad(render_target, pixel);
            let weight = 1. / f32(min(constant_data.frame_index, 4096u) + 1u);
            out_color = vec4<f32>((1. - weight) * prev_value.rgb + weight * out_color.rgb, 1.);
        } 
        textureStore(render_target, pixel, out_color);
        job_index = max_jobs - atomicSub(&jobs_count, 1);
    }
}