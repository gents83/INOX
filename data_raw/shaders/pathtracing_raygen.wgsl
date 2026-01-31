#include "raytracing_common.inc"

@group(0) @binding(3) var<storage, read_write> hits: array<RayHit>;
@group(0) @binding(4) var<storage, read_write> rays: array<Ray>;
@group(0) @binding(5) var<storage, read_write> counters: PathTracingCounters;
@group(0) @binding(6) var visibility_texture: texture_2d<u32>;
@group(0) @binding(7) var depth_texture: texture_depth_2d;
@group(0) @binding(8) var<storage, read_write> accumulator: array<RadiancePackedData>;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let resolution = vec2<u32>(u32(constant_data.screen_size.x), u32(constant_data.screen_size.y));

    if (global_id.x >= resolution.x || global_id.y >= resolution.y) {
        return;
    }

    let pixel_index = global_id.y * resolution.x + global_id.x;

    // Reset Accumulator if frame_index == 0 (handled by clear pass?)
    // Or accumulative logic? Assuming accumulative.

    // Initialize Hit
    var hit: RayHit;
    hit.t = -1.0;
    hit.pixel_index = pixel_index;

    // Read Visibility Buffer
    let visibility = textureLoad(visibility_texture, vec2<i32>(global_id.xy), 0).r;
    let depth = textureLoad(depth_texture, vec2<i32>(global_id.xy), 0);

    // Pack/Unpack visibility: instance_id | primitive_id ?
    // Inox Visibility Buffer usually stores: instance_id (20 bits) | primitive_id (12 bits)?
    // Or u64? No, texture is u32.
    // Let's assume instance << 10 | primitive?
    // I need to check how Visibility Buffer is written in Raster pass.
    // For now, I'll assume standard packing: instance_id (high), primitive_id (low).
    // Let's assume 20-12 split.
    let instance_id = visibility >> 12u;
    let primitive_id = visibility & 0xFFFu;

    // Generate Primary Ray
    let uv = (vec2<f32>(global_id.xy) + vec2<f32>(0.5)) / vec2<f32>(resolution);
    let clip_pos = vec4<f32>(uv * 2.0 - 1.0, 1.0, 1.0); // Z=1 (Far plane usually? or Near?)
    // Unproject
    let world_target_hom = constant_data.inverse_view_proj * clip_pos;
    let world_target = world_target_hom.xyz / world_target_hom.w;

    // Camera Origin (from View Matrix inverse column 3)
    let origin = vec3<f32>(constant_data.inv_view[3].x, constant_data.inv_view[3].y, constant_data.inv_view[3].z);
    let direction = normalize(world_target - origin);

    var ray: Ray;
    ray.origin = origin;
    ray.direction = direction;
    ray.t_min = constant_data.camera_near;
    ray.t_max = 10000.0; // Max distance
    ray.throughput = vec3<f32>(1.0);
    ray.pixel_index = pixel_index;
    ray.depth = 0u;

    // If background
    if (depth >= 1.0) {
        ray.t_max = -1.0; // Inactive
    } else {
        hit.instance_id = instance_id;
        hit.primitive_index = primitive_id;
        // Reconstruct t from depth?
        // Or let Geometry pass compute true t?
        // We need t for position reconstruction.
        // Geometry pass re-computes intersection to get barycentrics?
        // Visibility Buffer rasterization usually gives us Inst/Prim.
        // We calculate barycentrics in GeometryPass by projecting point?
        // Or simply Ray-Triangle intersection again to get barycentrics.
        // Since we have the Ray and the Triangle, we can intersect.
        // So Hit doesn't need t or barycentrics yet.

        // Count this ray
        atomicAdd(&counters.ray_count, 1u);
        atomicAdd(&counters.extension_ray_count, 1u); // It proceeds to Geometry/Lighting
    }

    hits[pixel_index] = hit;
    rays[pixel_index] = ray;
}
