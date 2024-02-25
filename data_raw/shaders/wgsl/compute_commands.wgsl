#import "common.inc"
#import "utils.inc"

@group(0) @binding(0)
var<storage, read> meshlets: Meshlets;
@group(0) @binding(1)
var<storage, read> meshes: Meshes;
@group(0) @binding(2)
var<storage, read_write> commands_count: atomic<u32>;
@group(0) @binding(3)
var<storage, read_write> commands: DrawIndexedCommands;
@group(0) @binding(4)
var<storage, read> meshlets_lod_level: array<u32>;

@compute
@workgroup_size(256, 1, 1)
fn main(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>, 
) {    
    let meshlet_id = global_invocation_id.x;
    if (meshlet_id >= arrayLength(&meshlets.data)) {
        return;
    }

    let meshlet = meshlets.data[global_invocation_id.x];
    let meshlet_lod_level = meshlet.mesh_index_and_lod_level & 7u;
    let desired_lod_level = meshlets_lod_level[meshlet_id];
    if(meshlet_lod_level != desired_lod_level) {     
        return;
    }
    let mesh_id = meshlet.mesh_index_and_lod_level >> 3u;
    let mesh = meshes.data[mesh_id];

    let command_index = atomicAdd(&commands_count, 1u);    
    commands.data[command_index].vertex_count = meshlet.indices_count;
    commands.data[command_index].instance_count = 1u;
    commands.data[command_index].base_index = meshlet.indices_offset;
    commands.data[command_index].vertex_offset = i32(mesh.vertices_position_offset);
    commands.data[command_index].base_instance = meshlet_id;
}