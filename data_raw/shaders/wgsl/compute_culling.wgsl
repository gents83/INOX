#import "common.inc"
#import "utils.inc"

struct CullingData {
    view: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
    mesh_flags: u32,
    lod0_meshlets_count: u32,
    _padding1: u32,
    _padding2: u32,
};

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<uniform> culling_data: CullingData;
@group(0) @binding(2)
var<storage, read> meshlets: Meshlets;
@group(0) @binding(3)
var<storage, read> bvh: BVH; //unused
@group(0) @binding(4)
var<storage, read> transforms: Transforms;
@group(0) @binding(5)
var<storage, read_write> instances: Instances;
@group(0) @binding(6)
var<storage, read_write> commands_data: array<atomic<i32>>;
@group(0) @binding(7)
var<storage, read_write> commands: DrawIndexedCommands;

@group(1) @binding(0)
var<storage, read_write> active_instances: Instances;
@group(1) @binding(1)
var<storage, read_write> meshlet_counts: array<atomic<u32>>;
@group(1) @binding(2) 
var texture_hzb: texture_2d<f32>;
@group(1) @binding(3)
var default_sampler: sampler;

#import "matrix_utils.inc"
#import "geom_utils.inc"

fn obb_vs_frustum(obb_min: vec3<f32>, obb_max: vec3<f32>, proj: mat4x4<f32>, view: mat4x4<f32>) -> bool {
    let planes = extract_frustum_planes(proj, view);
    for (var i = 0u; i < 6u; i++) {
        let plane = planes[i];
        let normal = plane.xyz;
        let distance = plane.w;

        // Optimized positive vertex calculation
        let positive_x = select(obb_max.x, obb_min.x, normal.x < 0.0);
        let positive_y = select(obb_max.y, obb_min.y, normal.y < 0.0);
        let positive_z = select(obb_max.z, obb_min.z, normal.z < 0.0);
        let positive_vertex = vec3<f32>(positive_x, positive_y, positive_z);

        if (dot(positive_vertex, normal) + distance < 0.0) {
            return false;
        }
    }
    return true;
}

fn transform_sphere(sphere: vec4<f32>, transform: mat4x4<f32>) -> vec4<f32> {
    let center = transform * vec4<f32>(sphere.xyz, 1.);
    let p = center.xyz / center.w;
    let v = transform * vec4<f32>(sphere.w, 0., 0., 0.);
    let l = length(v.xyz);
    return vec4<f32>(p, l);
}

fn is_infinite(value: f32) -> bool {
    let BIG_VALUE: f32 = 1e30f; // Adjust as needed
    return abs(value) > BIG_VALUE;
}

fn project_error_to_screen(sphere: vec4<f32>, fov: f32) -> f32 {
    // https://stackoverflow.com/questions/21648630/radius-of-projected-sphere-in-screen-space
    if (is_infinite(sphere.w)) {
        return sphere.w;
    }
    let cot_half_fov = 1. / tan(fov * 0.5);
    let d2 = dot(sphere.xyz, sphere.xyz);
    let r = sphere.w;
    let projected_radius = (constant_data.screen_height * cot_half_fov * r / sqrt(d2 - r * r));
    return projected_radius;
}

// https://github.com/zeux/meshoptimizer/blob/1e48e96c7e8059321de492865165e9ef071bffba/demo/nanite.cpp#L115
fn compute_bounds_error(sphere: vec4<f32>, error: f32, position: vec3<f32>, orientation: vec4<f32>, scale: vec3<f32>) -> f32 {
    let model_transform = transform_matrix(position, orientation, scale);
    let world_scale = max(scale.x, max(scale.y, scale.z));
    let sphere_world_space = (model_transform * vec4(sphere.xyz, 1.0)).xyz;
    let radius_world_space = world_scale * sphere.w;

    var view_pos = culling_data.inverse_view_proj * vec4<f32>(0., 0., 0.,1.);
    view_pos /= view_pos.w;
    let dir = sphere_world_space - view_pos.xyz;
    let distance = length(dir) - radius_world_space;
    let distance_clamped_to_znear = max(distance, constant_data.camera_near);
	let proj = 1.f / tan(constant_data.camera_fov * 0.5f);
    
    return (error / distance_clamped_to_znear) * proj * 0.5 * constant_data.screen_height;
}

fn is_lod_visible(meshlet: Meshlet, position: vec3<f32>, orientation: vec4<f32>, scale: vec3<f32>, fov: f32) -> bool {
    if (constant_data.forced_lod_level >= 0) {
        let desired_lod_level = MAX_LOD_LEVELS - 1u - u32(constant_data.forced_lod_level);
        return meshlet.lod_level == desired_lod_level;
    }
    let lod_error = compute_bounds_error(meshlet.bounding_sphere, meshlet.group_error, position, orientation, scale);
    let parent_error = compute_bounds_error(meshlet.parent_bounding_sphere, meshlet.parent_error, position, orientation, scale);
    if (lod_error <= 1. && parent_error > 1.) {
        return true;
    }
    return false;
}

fn is_occluded(aabb_min: vec3<f32>, aabb_max: vec3<f32>, view_proj: mat4x4<f32>) -> bool {
    let proj_min = view_proj * vec4<f32>(aabb_min, 1.0);
    let proj_max = view_proj * vec4<f32>(aabb_max, 1.0);
    var ndc_min = proj_min.xyz / proj_min.w;
    var ndc_max = proj_max.xyz / proj_max.w;
    ndc_min = min(ndc_min, ndc_max);
    ndc_max = max(ndc_min, ndc_max);
    if (ndc_max.z < 0.0 || ndc_min.z > 1.0 || 
        any(ndc_max.xy < vec2<f32>(-1.0)) || any(ndc_min.xy > vec2<f32>(1.0))) {
        return false; // Outside the view frustum
    }

    let clip_min = clip_to_normalized(ndc_min.xy);
    let clip_max = clip_to_normalized(ndc_max.xy);
    let width = (clip_max.x - clip_min.x) * constant_data.screen_width;
    let height = (clip_max.y - clip_min.y) * constant_data.screen_height;
    let mip_level = floor(0.5 * log2(max(width, height)));
    var depth_values = vec4<f32>(0.);
    depth_values.x = textureSampleLevel(texture_hzb, default_sampler, vec2<f32>(clip_min.x, clip_min.y), mip_level).r;
    depth_values.y = textureSampleLevel(texture_hzb, default_sampler, vec2<f32>(clip_min.x, clip_max.y), mip_level).r;
    depth_values.z = textureSampleLevel(texture_hzb, default_sampler, vec2<f32>(clip_max.x, clip_min.y), mip_level).r;
    depth_values.w = textureSampleLevel(texture_hzb, default_sampler, vec2<f32>(clip_max.x, clip_max.y), mip_level).r;
    let depth = max(max(depth_values.x, depth_values.y), max(depth_values.z, depth_values.w));
    if (ndc_min.z > depth + 0.1) {
        return true;
    }
    return false;
}

@compute
@workgroup_size(256, 1, 1)
fn main(
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, 
    @builtin(local_invocation_index) local_invocation_index: u32, 
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    let instance_id = global_invocation_id.x;
    if (instance_id >= arrayLength(&active_instances.data)) {
        return;
    }
     
    let instance = active_instances.data[instance_id];
    let transform = transforms.data[instance.transform_id];
    let meshlet_id = instance.meshlet_id;
    let meshlet = meshlets.data[meshlet_id];
    
    let position = transform.position_scale_x.xyz;
    let scale = vec3<f32>(transform.position_scale_x.w, transform.bb_min_scale_y.w, transform.bb_min_scale_y.w);
    
    if !is_lod_visible(meshlet, position, transform.orientation, scale, constant_data.camera_fov) {
        return;
    }

    let bb_min = transform_vector(meshlet.aabb_min, position, transform.orientation, scale);
    let bb_max = transform_vector(meshlet.aabb_max, position, transform.orientation, scale);
    let min_obb = min(bb_min, bb_max);
    let max_obb = max(bb_min, bb_max);
    if(!obb_vs_frustum(min_obb, max_obb, constant_data.proj, culling_data.view)) {
        return;
    }

    let view_proj = constant_data.proj * culling_data.view;
    if is_occluded(min_obb, max_obb, view_proj) {
        return;    
    }

    active_instances.data[instance_id].command_id = 0;
    
    let commands_count = arrayLength(&commands_data);
    for(var i = meshlet_id; i < commands_count; i++) {
        atomicAdd(&meshlet_counts[i], 1u);
    }
}