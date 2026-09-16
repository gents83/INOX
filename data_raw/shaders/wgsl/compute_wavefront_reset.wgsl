#import "wavefront_records.inc"

@group(0) @binding(0)
var<storage, read_write> queue_header: array<WavefrontQueueHeader>;
@group(0) @binding(1)
var<storage, read_write> dispatch_args: array<WavefrontDispatchArgs>;

@compute
@workgroup_size(1, 1, 1)
fn main() {
    atomicStore(&queue_header[0].count, 0u);
    atomicStore(&queue_header[0].overflowed, 0u);
    dispatch_args[0] = WavefrontDispatchArgs(0u, 1u, 1u);
}
