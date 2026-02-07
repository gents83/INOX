#include "raytracing_bindings.inc"

@group(1) @binding(0) var<storage, read> hits: array<RayHit>;
@group(1) @binding(1) var<storage, read_write> surface_data: array<SurfaceData>;
@group(1) @binding(2) var<storage, read> counters: PathTracingCounters;
@group(1) @binding(3) var<storage, read> rays: array<Ray>;

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    let ray_count = atomicLoad(&counters.extension_ray_count);

    if (index >= ray_count) {
        return;
    }

    let ray = rays[index];
    if (ray.t_max < 0.0) {
        return;
    }

    let hit = hits[ray.pixel_index];
    if (hit.t < 0.0 || hit.instance_id == 0xFFFFFFFFu) {
        // Miss
        // SurfaceData flag?
        surface_data[ray.pixel_index].flags = 0u; // Invalid
        return;
    }

    let instance_idx = hit.instance_id;
    let tri_idx = hit.primitive_index;
    let mesh_idx = get_instance_mesh_index(instance_idx);
    let transform = get_transform_data(get_instance_transform_index(instance_idx));

    let idx_vec = get_triangle_indices(get_mesh_indices_offset(mesh_idx), tri_idx);

    // Interpolate
    let b = vec3<f32>(1.0 - hit.barycentrics.x - hit.barycentrics.y, hit.barycentrics.x, hit.barycentrics.y);

    // Position
    let p0 = get_vertex_pos_world(idx_vec.x, transform);
    let p1 = get_vertex_pos_world(idx_vec.y, transform);
    let p2 = get_vertex_pos_world(idx_vec.z, transform);
    let position = p0 * b.x + p1 * b.y + p2 * b.z;

    // Normal
    let n0 = get_vertex_normal_world(idx_vec.x, transform);
    let n1 = get_vertex_normal_world(idx_vec.y, transform);
    let n2 = get_vertex_normal_world(idx_vec.z, transform);
    let normal = normalize(n0 * b.x + n1 * b.y + n2 * b.z);

    // UV
    let uv0 = get_vertex_uv(idx_vec.x);
    let uv1 = get_vertex_uv(idx_vec.y);
    let uv2 = get_vertex_uv(idx_vec.z);
    let uv = uv0 * b.x + uv1 * b.y + uv2 * b.z;

    // Tangent
    let t0 = get_vertex_tangent(idx_vec.x);
    let t1 = get_vertex_tangent(idx_vec.y);
    let t2 = get_vertex_tangent(idx_vec.z);
    let tangent = normalize(t0 * b.x + t1 * b.y + t2 * b.z);

    var surface: SurfaceData;
    surface.position = position;
    surface.normal = normal;
    surface.uv = uv;
    surface.tangent = tangent;
    surface.material_index = get_mesh_material_index(mesh_idx);
    surface.flags = 1u; // Valid

    surface_data[ray.pixel_index] = surface;
}
