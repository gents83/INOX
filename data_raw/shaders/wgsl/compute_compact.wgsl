

#import "utils.inc"
#import "common.inc"

struct CullingData {
    view: mat4x4<f32>,
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
var<storage, read> meshlets_bb: AABBs;

@group(1) @binding(0)
var<storage, read_write> count: atomic<u32>;
@group(1) @binding(1)
var<storage, read_write> commands: DrawIndexedCommands;
@group(1) @binding(2)
var<storage, read_write> visible_draw_data: array<atomic<u32>>;



@compute
@workgroup_size(32, 1, 1)
fn main(
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, 
    @builtin(local_invocation_index) local_invocation_index: u32, 
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    let total = arrayLength(&meshlets.data);
    let meshlet_id = global_invocation_id.x;
    if (meshlet_id >= total) {
        return;
    }
    let meshlet = &meshlets.data[meshlet_id];
    let mesh_id = (*meshlet).mesh_index;
    let mesh = &meshes.data[mesh_id];
    
    let draw_group_index = workgroup_id.x;

    let bits = atomicLoad(&visible_draw_data[draw_group_index]);
    let shift = 1u << local_invocation_id.x;
    let is_visible = bits & shift;
    if (is_visible != 0u) {
        let mask = 0xFFFFFFFFu << local_invocation_id.x;
        let result = bits & mask;
        let group_count = countOneBits(result);

        var previous_count = 0u;
        for(var i = 0u; i < draw_group_index; i = i + 1u) {
            let b = atomicLoad(&visible_draw_data[i]);
            previous_count = previous_count + countOneBits(b);
        }
        let index = previous_count + group_count;

        let command = &commands.data[index - 1u];
        (*command).vertex_count = (*meshlet).indices_count;
        (*command).instance_count = 1u;
        (*command).base_index = (*mesh).indices_offset + (*meshlet).indices_offset;
        (*command).vertex_offset = i32((*mesh).vertex_offset);
        (*command).base_instance = meshlet_id;
    }
}