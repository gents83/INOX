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
var<storage, read> bvh: BVH;
@group(0) @binding(4)
var<storage, read> transforms: Transforms;
@group(0) @binding(5)
var<storage, read_write> instances: Instances;
@group(0) @binding(6)
var<storage, read_write> commands_data: array<atomic<i32>>;
@group(0) @binding(7)
var<storage, read_write> commands: DrawIndexedCommands;

@group(1) @binding(0)
var<storage, read_write> active_instances: ActiveInstances;
@group(1) @binding(1)
var<storage, read_write> meshlet_counts: array<atomic<u32>>;

#import "matrix_utils.inc"
#import "geom_utils.inc"


fn is_sphere_inside_frustum(min: vec3<f32>, max: vec3<f32>, position: vec3<f32>, orientation: vec4<f32>, scale: vec3<f32>, frustum: vec4<f32>, znear: f32, zfar: f32) -> bool {
    var center = min + (max - min) * 0.5;
    center = (culling_data.view * vec4<f32>(transform_vector(center, position, orientation, scale), 1.)).xyz;
    let max_scale = max(max(scale.x, scale.y), scale.z);
    let radius = length((max - min) * 0.5) * max_scale;
    
    let m1 = (culling_data.view * vec4<f32>(transform_vector(min, position, orientation, scale), 1.)).xyz;
    let m2 = (culling_data.view * vec4<f32>(transform_vector(max, position, orientation, scale), 1.)).xyz;

    let view_dir = vec3<f32>(0., 0., -1.);
    if(dot(m1, view_dir) < 0. && dot(m2, view_dir) < 0.)
    {
        return false;
    }
    var visible = true;
	// the left/top/right/bottom plane culling utilizes frustum symmetry to cull against two planes at the same time
	visible = visible && center.z * frustum.y - abs(center.x) * frustum.x > -radius;
	visible = visible && center.z * frustum.w - abs(center.y) * frustum.z > -radius;
    return visible;
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
    let instance = instances.data[instance_id];
    let transform = transforms.data[instance.transform_id];
    let meshlet_id = instance.meshlet_id;
    let meshlet = meshlets.data[meshlet_id];
    let meshlet_lod_level = meshlet.mesh_index_and_lod_level & 7u;
    let bvh_node = bvh.data[meshlet.bvh_offset];
    
    let position = transform.position_scale_x.xyz;
    let scale = vec3<f32>(transform.position_scale_x.w, transform.bb_min_scale_y.w, transform.bb_min_scale_y.w);
    let bb_min = transform_vector(bvh_node.min, position, transform.orientation, scale);
    let bb_max = transform_vector(bvh_node.max, position, transform.orientation, scale);
    let min = min(bb_min, bb_max);
    let max = max(bb_min, bb_max);
    
    let perspective_t = matrix_transpose(constant_data.proj);
    // x + w < 0
    let frustum_x = normalize(perspective_t[3] + perspective_t[0]);
    // y + w < 0
    let frustum_y = normalize(perspective_t[3] + perspective_t[1]);
    let frustum = vec4<f32>(frustum_x.x, frustum_x.z, frustum_y.y, frustum_y.z);
    if(!is_sphere_inside_frustum(bvh_node.min, bvh_node.max, position, transform.orientation, scale, frustum, constant_data.camera_near, constant_data.camera_far)) {
        return;
    } 
    
    //Evaluate screen occupancy to decide if lod is ok to use for this meshlet or to use childrens
    var screen_lod_level = 0u;   
    let f_max = f32(MAX_LOD_LEVELS);   
    //
    let ncd_min = view_proj * vec4<f32>(min, 1.);
    let clip_min = ncd_min.xyz / ncd_min.w;
    let screen_min = clip_to_normalized(clip_min.xy);
    let ncd_max = view_proj * vec4<f32>(max, 1.);
    let clip_max = ncd_max.xyz / ncd_max.w;
    let screen_max = clip_to_normalized(clip_max.xy);
    let screen_diff = max(screen_max, screen_min) - min(screen_max, screen_min);
    let screen_occupancy = clamp(max(screen_diff.x, screen_diff.y), 0., 1.);  
    screen_lod_level =  clamp(u32(screen_occupancy * f_max), 0u, MAX_LOD_LEVELS - 1u);
    //
    let center = min + (max-min) * 0.5;
    let distance = length(view_pos() - center);
    let distance_lod_level = MAX_LOD_LEVELS - 1u - clamp(u32(((distance * distance) / (constant_data.camera_far - constant_data.camera_near)) * f_max), 0u, MAX_LOD_LEVELS - 1u);
    //
    var desired_lod_level = max(distance_lod_level, screen_lod_level);
    //
    if (constant_data.forced_lod_level >= 0) {
        desired_lod_level = MAX_LOD_LEVELS - 1u - u32(constant_data.forced_lod_level);
    }
    //
    var command_id = -1;
    if(meshlet_lod_level == desired_lod_level) {
        let result = atomicCompareExchangeWeak(&commands_data[meshlet_id], -1, 0);
        command_id = result.old_value;
        if(result.exchanged) {
            let command_index = atomicAdd(&commands.count, 1u);
            command_id = i32(command_index);
            atomicStore(&commands_data[meshlet_id], command_id); 
        }
        atomicStore(&instances.data[instance_id].command_id, command_id);

        let active_instance_id = atomicAdd(&active_instances.count, 1u);
        active_instances.data[active_instance_id] = instances.data[instance_id];
        
        for (var i = 0u; i <= meshlet_id; i++) {
            atomicAdd(&meshlet_counts[meshlet_id], 1u); 
        }
    }
}