#import "common.inc"
#import "utils.inc"

struct CullingData {
    view: mat4x4<f32>,
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

fn is_box_inside_frustum(aabb_min: vec3<f32>, aabb_max: vec3<f32>, view_proj: mat4x4<f32>) -> bool {
    // Calculate AABB center and half-extents
    let center = (aabb_min + aabb_max) * 0.5;
    let extents = (aabb_max - aabb_min) * 0.5;

    // Transform the center point into clip space
    var clip_center = view_proj * vec4(center, 1.0);

    // Early out: if the center is inside the frustum, we consider the AABB visible
    if (all(abs(clip_center.xyz) <= clip_center.www)) {
        return true;
    }

    // Handle the reversed depth in WebGPU
    clip_center.z = -clip_center.z;

    // Calculate the sum of the absolute values of the extents projected onto each axis
    let abs_extents_x = abs(extents.x * view_proj[0][0]) + abs(extents.y * view_proj[1][0]) + abs(extents.z * view_proj[2][0]);
    let abs_extents_y = abs(extents.x * view_proj[0][1]) + abs(extents.y * view_proj[1][1]) + abs(extents.z * view_proj[2][1]);
    let abs_extents_z = abs(extents.x * view_proj[0][2]) + abs(extents.y * view_proj[1][2]) + abs(extents.z * view_proj[2][2]);

    // Check against each pair of symmetric planes
    if (abs(clip_center.x) > abs_extents_x + clip_center.w) {
        return false; // Outside left/right planes
    }
    if (abs(clip_center.y) > abs_extents_y + clip_center.w) {
        return false; // Outside top/bottom planes
    }
    if (abs(clip_center.z) > abs_extents_z + clip_center.w) {
        return false; // Outside near/far planes
    }

    return true; // Inside or intersecting all planes
}

fn is_sphere_inside_frustum(min: vec3<f32>, max: vec3<f32>, position: vec3<f32>, orientation: vec4<f32>, scale: vec3<f32>, frustum: vec4<f32>, znear: f32, zfar: f32) -> bool {
    var center = (min + max) * 0.5;
    center = (culling_data.view * vec4<f32>(center, 1.0)).xyz;
    let radius = length(max - min) * 0.5;
    let v = -(center.z + radius * 0.5);

    // Frustum plane checks
    var visible = true;
    visible &= v * frustum.y < -abs(center.x) * frustum.x;
    visible &= v * frustum.w < -abs(center.y) * frustum.z;
    visible &= v > znear && v < zfar;
    
    return visible;
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

fn is_lod_visible(meshlet: Meshlet, position: vec3<f32>, orientation: vec4<f32>, scale: vec3<f32>, fov: f32) -> bool {
    if (constant_data.forced_lod_level >= 0) {
        let desired_lod_level = MAX_LOD_LEVELS - 1u - u32(constant_data.forced_lod_level);
        return meshlet.lod_level == desired_lod_level;
    }
    let model_transform = transform_matrix(position, orientation, scale);
    let model_view = culling_data.view * model_transform;
    var projected_bounds = vec4<f32>(meshlet.bounding_sphere.xyz, max(meshlet.cluster_error, MAX_PROJECTED_ERROR));
    projected_bounds = transform_sphere(projected_bounds, model_view);

    var parent_projected_bounds  = vec4<f32>(meshlet.parent_bounding_sphere.xyz, max(meshlet.parent_error, MAX_PROJECTED_ERROR));
    parent_projected_bounds = transform_sphere(parent_projected_bounds, model_view);

    let cluster_error = project_error_to_screen(projected_bounds, fov);
    let parent_error = project_error_to_screen(parent_projected_bounds, fov);
    let render = cluster_error <= LOD_ERROR_THRESHOLD && parent_error > LOD_ERROR_THRESHOLD;
    return render;
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
     
    let view_proj = constant_data.proj * culling_data.view;
    let instance = active_instances.data[instance_id];
    let transform = transforms.data[instance.transform_id];
    let meshlet_id = instance.meshlet_id;
    let meshlet = meshlets.data[meshlet_id];
    
    let position = transform.position_scale_x.xyz;
    let scale = vec3<f32>(transform.position_scale_x.w, transform.bb_min_scale_y.w, transform.bb_min_scale_y.w);

    let bb_min = transform_vector(meshlet.aabb_min, position, transform.orientation, scale);
    let bb_max = transform_vector(meshlet.aabb_max, position, transform.orientation, scale);
    
    if !is_lod_visible(meshlet, position, transform.orientation, scale, constant_data.camera_fov) {
        return;
    }
    if(!is_box_inside_frustum(bb_min, bb_max, view_proj)) {
        return;
    }
    if is_occluded(bb_min, bb_max, view_proj) {
        return;    
    }

    active_instances.data[instance_id].command_id = 0;
    
    let commands_count = arrayLength(&commands_data);
    for(var i = meshlet_id; i < commands_count; i++) {
        atomicAdd(&meshlet_counts[i], 1u);
    }
}