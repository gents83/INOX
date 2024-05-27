#import "common.inc"
#import "utils.inc"

@group(0) @binding(0)
var<storage, read_write> instances: Instances;
@group(0) @binding(1)
var<storage, read_write> active_instances: ActiveInstances;
@group(0) @binding(2)
var<storage, read_write> commands_data: array<atomic<i32>>;
@group(0) @binding(3)
var<storage, read_write> commands: DrawIndexedCommands;


@compute
@workgroup_size(32, 1, 1)
fn main(
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, 
    @builtin(local_invocation_index) local_invocation_index: u32, 
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    let id = global_invocation_id.x;
    atomicStore(&active_instances.count, 0u);
    if (id < arrayLength(&instances.data)) {
        instances.data[id].command_id = -1;
        active_instances.data[id] = instances.data[id];
    }
    if (id < arrayLength(&commands_data)) {
        atomicStore(&commands_data[id], -1);
    }
    atomicStore(&commands.count, 0u);
    if (id < arrayLength(&commands.data)) {
        commands.data[id].base_instance = 0u;
        commands.data[id].base_index = 0u;
        commands.data[id].vertex_count = 0u;
        commands.data[id].vertex_offset = 0;
        atomicStore(&commands.data[id].instance_count, 0u);
    }
}