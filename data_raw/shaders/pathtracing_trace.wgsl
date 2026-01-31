#include "raytracing_bindings.inc"

@group(1) @binding(0) var<storage, read> rays: array<Ray>;
@group(1) @binding(1) var<storage, read_write> hits: array<RayHit>;
@group(1) @binding(2) var<storage, read_write> counters: PathTracingCounters;

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    let ray_count = atomicLoad(&counters.extension_ray_count);

    if (index >= ray_count) {
        return;
    }

    // Dense or Sparse?
    // If compacted, rays[index] is valid.
    let ray = rays[index];
    if (ray.t_max < 0.0) {
        return;
    }

    var hit = hits[ray.pixel_index];
    hit.t = ray.t_max;
    hit.instance_id = 0xFFFFFFFFu;
    hit.primitive_index = 0xFFFFFFFFu;

    traverse_bvh(ray, &hit);

    hits[ray.pixel_index] = hit;
}
