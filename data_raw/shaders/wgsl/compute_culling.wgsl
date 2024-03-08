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
var<storage, read> meshes: Meshes;
@group(0) @binding(4)
var<storage, read> bhv: BHV;
@group(0) @binding(5)
var<storage, read_write> meshlets_lod_level: array<atomic<u32>>;

#import "matrix_utils.inc"
#import "geom_utils.inc"

//ScreenSpace Frustum Culling
fn is_box_inside_frustum(min: vec3<f32>, max: vec3<f32>, frustum: array<vec4<f32>, 4>) -> bool {
    var visible: bool = false;    
    var points: array<vec3<f32>, 8>;
    points[0] = min;
    points[1] = max;
    points[2] = vec3<f32>(min.x, min.y, max.z);
    points[3] = vec3<f32>(min.x, max.y, max.z);
    points[4] = vec3<f32>(min.x, max.y, min.z);
    points[5] = vec3<f32>(max.x, min.y, min.z);
    points[6] = vec3<f32>(max.x, max.y, min.z);
    points[7] = vec3<f32>(max.x, min.y, max.z);
      
    var f = frustum;
    for(var i = 0; !visible && i < 4; i = i + 1) {  
        for(var p = 0; !visible && p < 8; p = p + 1) {        
            visible = visible || !(dot(f[i].xyz, points[p]) + f[i].w <= 0.);
        }
    }   
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
    let meshlet_id = global_invocation_id.x;
    if (meshlet_id >= arrayLength(&meshlets.data)) {
        return;
    }

    let meshlet = meshlets.data[meshlet_id];
    let mesh_id = meshlet.mesh_index_and_lod_level >> 3u;
    let meshlet_lod_level = meshlet.mesh_index_and_lod_level & 7u;
    var mesh = meshes.data[mesh_id];
    let flags = (mesh.flags_and_vertices_attribute_layout & 0xFFFF0000u) >> 16u;
    if (flags != culling_data.mesh_flags) {   
        return;
    }

    let bb_id = mesh.blas_index + meshlet.bvh_offset;
    let bb = &bhv.data[bb_id];
    let bb_max = transform_vector((*bb).max, mesh.position, mesh.orientation, mesh.scale);
    let bb_min = transform_vector((*bb).min, mesh.position, mesh.orientation, mesh.scale);
    let min = min(bb_min, bb_max);
    let max = max(bb_min, bb_max);

    let clip_mvp = constant_data.proj * culling_data.view;
    let row0 = matrix_row(clip_mvp, 0u);
    let row1 = matrix_row(clip_mvp, 1u);
    let row3 = matrix_row(clip_mvp, 3u);
    var frustum: array<vec4<f32>, 4>;
    frustum[0] = normalize_plane(row3 + row0);
    frustum[1] = normalize_plane(row3 - row0);
    frustum[2] = normalize_plane(row3 + row1);
    frustum[3] = normalize_plane(row3 - row1);
    if !is_box_inside_frustum(min, max, frustum) {
        return;
    }

    //Evaluate screen occupancy to decide if lod is ok to use for this meshlet or to use childrens
    var screen_lod_level = 0u;   
    let f_max = f32(MAX_LOD_LEVELS);   

    let ncd_min = clip_mvp * vec4<f32>(min, 1.);
    let clip_min = ncd_min.xyz / ncd_min.w;
    let screen_min = clip_to_normalized(clip_min.xy);
    let ncd_max = clip_mvp * vec4<f32>(max, 1.);
    let clip_max = ncd_max.xyz / ncd_max.w;
    let screen_max = clip_to_normalized(clip_max.xy);
    let screen_diff = max(screen_max, screen_min) - min(screen_max, screen_min);
    let screen_occupancy = clamp(max(screen_diff.x, screen_diff.y), 0., 1.);  
    screen_lod_level =  clamp(u32(screen_occupancy * f_max), 0u, MAX_LOD_LEVELS - 1u);

    let center = min + (max-min) * 0.5;
    let distance = length(view_pos() - center);
    let distance_lod_level = MAX_LOD_LEVELS - 1u - clamp(u32(((distance * distance) / (constant_data.camera_far - constant_data.camera_near)) * f_max), 0u, MAX_LOD_LEVELS - 1u);

    var desired_lod_level = max(distance_lod_level, screen_lod_level);

    if (constant_data.forced_lod_level >= 0) {
        desired_lod_level = MAX_LOD_LEVELS - 1u - u32(constant_data.forced_lod_level);
    }

    if(meshlet_lod_level == desired_lod_level) {
        atomicAnd(&meshlets_lod_level[meshlet_id], 0u);
    }
    else if(desired_lod_level == (meshlet_lod_level + 1u)) {
        if(meshlet.child_meshlets.x >= 0) {
            atomicAnd(&meshlets_lod_level[meshlet.child_meshlets.x], 0u);
        }
        if(meshlet.child_meshlets.y >= 0) {
            atomicAnd(&meshlets_lod_level[meshlet.child_meshlets.y], 0u);
        }
        if(meshlet.child_meshlets.z >= 0) {
            atomicAnd(&meshlets_lod_level[meshlet.child_meshlets.z], 0u);
        }
        if(meshlet.child_meshlets.w >= 0) {
            atomicAnd(&meshlets_lod_level[meshlet.child_meshlets.w], 0u);
        }
    }
}