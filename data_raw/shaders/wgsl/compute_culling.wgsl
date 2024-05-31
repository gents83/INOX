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

#import "matrix_utils.inc"
#import "geom_utils.inc"

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

fn transform_sphere(sphere: vec4<f32>, position: vec3<f32>, orientation: vec4<f32>, scale: vec3<f32>) -> vec4<f32> {
    let transform = culling_data.view * transform_matrix(position, orientation, scale);
    let center = transform * vec4<f32>(sphere.xyz, 1.);
    let p = center.xyz / center.w;
    let v = transform * vec4<f32>(sphere.w, 0., 0., 0.);
    return vec4<f32>(p, length(v.xyz));
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
    let cot_half_fov = 1.0 / tan(fov * 0.5);
    let d2 = dot(sphere.xyz, sphere.xyz);
    let r = sphere.w;
    let projected_radius = constant_data.screen_height * 4. * cot_half_fov * r / sqrt(d2 - r * r);
    return projected_radius;
}

fn is_lod_visible(meshlet: Meshlet, position: vec3<f32>, orientation: vec4<f32>, scale: vec3<f32>, fov: f32) -> bool {
    if (constant_data.forced_lod_level >= 0) {
        let desired_lod_level = MAX_LOD_LEVELS - 1u - u32(constant_data.forced_lod_level);
        return meshlet.lod_level == desired_lod_level;
    }
    var projected_bounds = vec4<f32>(meshlet.bounding_sphere.xyz, max(meshlet.cluster_error, MAX_PROJECTED_ERROR));
    projected_bounds = transform_sphere(projected_bounds, position, orientation, scale);

    var parent_projected_bounds  = vec4<f32>(meshlet.parent_bounding_sphere.xyz, max(meshlet.parent_error, MAX_PROJECTED_ERROR));
    parent_projected_bounds = transform_sphere(parent_projected_bounds, position, orientation, scale);

    let cluster_error: f32 = project_error_to_screen(projected_bounds, fov);
    let parent_error: f32 = project_error_to_screen(parent_projected_bounds, fov);
    let render: bool = cluster_error <= LOD_ERROR_THRESHOLD && parent_error > LOD_ERROR_THRESHOLD;
    return render;
}

@compute
@workgroup_size(32, 1, 1)
fn main(
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, 
    @builtin(local_invocation_index) local_invocation_index: u32, 
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    let instance_id = global_invocation_id.x;
    if (instance_id >= arrayLength(&instances.data)) {
        return;
    }
     
    let view_proj = constant_data.proj * culling_data.view;
    let instance = active_instances.data[instance_id];
    let transform = transforms.data[instance.transform_id];
    let meshlet_id = instance.meshlet_id;
    let meshlet = meshlets.data[meshlet_id];
    let meshlet_lod_level = meshlet.lod_level;
    
    let position = transform.position_scale_x.xyz;
    let scale = vec3<f32>(transform.position_scale_x.w, transform.bb_min_scale_y.w, transform.bb_min_scale_y.w);
    let bb_min = transform_vector(meshlet.aabb_min, position, transform.orientation, scale);
    let bb_max = transform_vector(meshlet.aabb_max, position, transform.orientation, scale);
    let aabb_min = min(bb_min, bb_max);
    let aabb_max = max(bb_min, bb_max);
    
    let perspective_t = matrix_transpose(constant_data.proj);
    // x + w < 0
    let frustum_x = normalize(perspective_t[3] + perspective_t[0]);
    // y + w < 0
    let frustum_y = normalize(perspective_t[3] + perspective_t[1]);
    let frustum = vec4<f32>(frustum_x.x, frustum_x.z, frustum_y.y, frustum_y.z);
    if(!is_sphere_inside_frustum(aabb_min, aabb_max, position, transform.orientation, scale, frustum, constant_data.camera_near, constant_data.camera_far)) {
        return;
    } 
    if !is_lod_visible(meshlet, position, transform.orientation, scale, constant_data.camera_fov) {
        return;
    }

    var command_id = -1;
    let result = atomicCompareExchangeWeak(&commands_data[meshlet_id], -1, 0);
    if(result.old_value == -1) {
        command_id = i32(atomicAdd(&commands.count, 1u));
        atomicStore(&commands_data[meshlet_id], i32(command_id));
    } else {
        command_id = result.old_value;
    }
    instances.data[instance_id].command_id = i32(command_id);
    active_instances.data[instance_id].command_id = i32(command_id);

    let commands_count = i32(arrayLength(&commands.data));
    for (var i = command_id; i < commands_count; i++) {
        atomicAdd(&meshlet_counts[i], 1u); 
    }
}