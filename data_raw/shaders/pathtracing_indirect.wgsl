#include "raytracing_structs.inc"

struct DispatchIndirectArgs {
    workgroup_count_x: u32,
    workgroup_count_y: u32,
    workgroup_count_z: u32,
};

@group(0) @binding(0) var<uniform> constant_data: ConstantData;
@group(0) @binding(1) var<storage, read_write> counters: PathTracingCounters;
@group(0) @binding(2) var<storage, read_write> dispatch_args: DispatchIndirectArgs;

@compute @workgroup_size(1)
fn main() {
    // Read next_ray_count (produced by Lighting pass)
    let count = atomicLoad(&counters.next_ray_count);

    // Set as extension_ray_count for the next frame/bounce
    atomicStore(&counters.extension_ray_count, count);

    // Reset next_ray_count for the next pass
    atomicStore(&counters.next_ray_count, 0u);

    // Calculate dispatch args
    let workgroup_size = 64u;
    let groups = (count + workgroup_size - 1u) / workgroup_size;

    dispatch_args.workgroup_count_x = groups;
    dispatch_args.workgroup_count_y = 1u;
    dispatch_args.workgroup_count_z = 1u;
}
