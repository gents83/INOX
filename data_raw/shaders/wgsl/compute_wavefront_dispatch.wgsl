#import "common.inc"
#import "wavefront_records.inc"

@group(0) @binding(0)
var<storage, read_write> queue_header: array<WavefrontQueueHeader>;
@group(0) @binding(1)
var<storage, read_write> dispatch_args: array<WavefrontDispatchArgs>;

@compute
@workgroup_size(1, 1, 1)
fn main() {
    let count = min(atomicLoad(&queue_header[0].count), queue_header[0].capacity);
    dispatch_args[0] = WavefrontDispatchArgs(
        select(0u, (count + 63u) / 64u, count > 0u),
        1u,
        1u
    );
}
