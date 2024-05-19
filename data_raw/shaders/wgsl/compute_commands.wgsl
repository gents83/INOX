#import "common.inc"
#import "utils.inc"


@group(0) @binding(0)
var<storage, read> meshlets: Meshlets;
@group(0) @binding(1)
var<storage, read> meshes: Meshes;
@group(0) @binding(2)
var<storage, read_write> instances: Instances;
@group(0) @binding(3)
var<storage, read> active_instances: ActiveInstances;
@group(0) @binding(4)
var<storage, read> meshlet_counts: array<u32>;
@group(0) @binding(5)
var<storage, read> commands_data: array<i32>;
@group(0) @binding(6)
var<storage, read_write> commands: DrawIndexedCommands;

@compute
@workgroup_size(32, 1, 1)
fn main(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>, 
) {  
    let instance_id = global_invocation_id.x;
    if (instance_id >= arrayLength(&active_instances.data)) {
        return;
    }

    var command_index = atomicLoad(&active_instances.data[instance_id].command_id);
    if(command_index < 0) {
        return;
    }

    let instance = active_instances.data[instance_id];
    let mesh = meshes.data[instance.mesh_id];
    let meshlet_id = instance.meshlet_id;
    let meshlet = meshlets.data[meshlet_id];

    let index = commands_data[meshlet_id];
    var first_instance = 0u;
    let instance_count = atomicAdd(&commands.data[index].instance_count, 1u);
    if (index > 0) {
        first_instance = meshlet_counts[index - 1];
    }
    //same for everyone
    commands.data[index].vertex_count = meshlet.indices_count;
    commands.data[index].base_index = meshlet.indices_offset;
    commands.data[index].vertex_offset = i32(mesh.vertices_position_offset);
    //we need to find first instance
    commands.data[index].base_instance = first_instance;
    //we need to pack instances of same meshlet
    instances.data[first_instance + instance_count] = instance;
}