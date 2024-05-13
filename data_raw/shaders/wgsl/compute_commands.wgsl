#import "common.inc"
#import "utils.inc"


@group(0) @binding(0)
var<storage, read> meshlets: Meshlets;
@group(0) @binding(1)
var<storage, read> meshes: Meshes;
@group(0) @binding(2)
var<storage, read> instances: Instances;
@group(0) @binding(3)
var<storage, read> commands_data: array<i32>;
@group(0) @binding(4)
var<storage, read_write> commands: DrawIndexedCommands;

@compute
@workgroup_size(32, 1, 1)
fn main(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>, 
) {  

    let work_id = global_invocation_id.x;
    if (work_id >= arrayLength(&commands_data)) {
        return;
    }

    let instance_id = commands_data[work_id];
    if(instance_id < 0) {
        return;
    }

    let instance = instances.data[instance_id];
    let mesh = meshes.data[instance.mesh_id];
    let meshlet = meshlets.data[instance.meshlet_id];

    let command_index = atomicAdd(&commands.count, 1u);    
    commands.data[command_index].vertex_count = meshlet.indices_count;
    commands.data[command_index].instance_count = 1u;
    commands.data[command_index].base_index = meshlet.indices_offset;
    commands.data[command_index].vertex_offset = i32(mesh.vertices_position_offset);
    commands.data[command_index].base_instance = u32(instance_id);
}