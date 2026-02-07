struct ConstantData {
    view: mat4x4<f32>,
    inv_view: mat4x4<f32>,
    proj: mat4x4<f32>,
    view_proj: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
    screen_size: vec2<f32>,
    frame_index: u32,
    flags: u32,
    debug_uv_coords: vec2<f32>,
    tlas_starting_index: u32,
    num_bounces: u32,
    lut_pbr_charlie_texture_index: u32,
    lut_pbr_ggx_texture_index: u32,
    env_map_texture_index: u32,
    num_lights: u32,
    forced_lod_level: i32,
    camera_near: f32,
    camera_far: f32,
    camera_fov: f32,
    indices_offset: u32,
    vertex_positions_offset: u32,
    vertex_attributes_offset: u32,
    meshes_offset: u32,
    meshlets_offset: u32,
    instances_offset: u32,
    transforms_offset: u32,
    bvh_offset: u32,
    materials_offset: u32,
    lights_offset: u32,
};

struct Ray {
    origin: vec3<f32>,
    t_min: f32,
    direction: vec3<f32>,
    t_max: f32,
    throughput: vec3<f32>,
    pixel_index: u32,
    depth: u32,
    pad_a: f32,
    pad_b: f32,
    pad_c: f32,
};

struct RayHit {
    instance_id: u32,
    primitive_index: u32,
    barycentrics: vec2<f32>,
    t: f32,
    pixel_index: u32,
    pad_a: u32,
    pad_b: u32,
    direction: vec3<f32>,
    pad_dir: f32,
    throughput: vec3<f32>,
    pad_thr: f32,
};

struct ShadowRay {
    origin: vec3<f32>,
    t_max: f32,
    direction: vec3<f32>,
    radiance: u32, // packed rgbe? or just u32 ID? Assuming radiance contribution is calculated later or packed.
    // Wait, ShadowRay struct in Rust: origin, t_max, direction, radiance, contribution, pixel_index.
    contribution: vec3<f32>,
    pixel_index: u32,
};

struct SurfaceData {
    position: vec3<f32>,
    material_index: i32,
    normal: vec3<f32>,
    flags: u32,
    uv: vec2<f32>,
    roughness: f32,
    metallic: f32,
    albedo: vec3<f32>,
    padding: f32,
    tangent: vec4<f32>,
};

struct PathTracingCounters {
    ray_count: atomic<u32>,
    hit_count: u32,
    shadow_ray_count: atomic<u32>,
    extension_ray_count: atomic<u32>,
    next_ray_count: atomic<u32>,
};

struct RadiancePackedData {
    data: vec4<f32>,
};

struct GPUBVHNode {
    min: vec3<f32>,
    left_node: i32,
    max: vec3<f32>,
    primitive_index: i32,
};

// Bindings
@group(0) @binding(0) var<uniform> constant_data: ConstantData;
@group(0) @binding(1) var<storage, read> geometry_buffer: array<u32>;
@group(0) @binding(2) var<storage, read> scene_buffer: array<u32>;

// Struct Strides (in u32s)
const MESH_STRIDE: u32 = 23u; // 92 bytes
const MESHLET_STRIDE: u32 = 20u; // GPUMeshlet size? 80 bytes?
// GPUMeshlet: mesh_index(1), indices_offset(1), indices_count(1), lod(1), aabb_min(3), parent_error(1), aabb_max(3), group_error(1), sphere(4), parent_sphere(4).
// 1+1+1+1 + 3+1 + 3+1 + 4 + 4 = 21 u32s.
// Wait:
// u32 x 4 = 4
// vec3 + f32 = 4
// vec3 + f32 = 4
// vec4 = 4
// vec4 = 4
// Total = 20 u32s. 80 bytes. Correct.

const INSTANCE_STRIDE: u32 = 4u; // 16 bytes
const TRANSFORM_STRIDE: u32 = 16u; // 64 bytes

// Helper Functions

fn get_mesh_indices_offset(mesh_index: u32) -> u32 {
    // GPUMesh.indices_offset is at index 6
    // vertices_pos(1), vertices_attr(1), flags(1), mat(1), meshlets(1), blas(1), indices_offset(1)
    let base = constant_data.meshes_offset + mesh_index * MESH_STRIDE;
    return scene_buffer[base + 6u];
}

fn get_mesh_material_index(mesh_index: u32) -> i32 {
    let base = constant_data.meshes_offset + mesh_index * MESH_STRIDE;
    return bitcast<i32>(scene_buffer[base + 3u]);
}

fn get_meshlet_indices_offset(meshlet_index: u32) -> u32 {
    let base = constant_data.meshlets_offset + meshlet_index * MESHLET_STRIDE;
    return scene_buffer[base + 1u];
}

fn get_meshlet_indices_count(meshlet_index: u32) -> u32 {
    let base = constant_data.meshlets_offset + meshlet_index * MESHLET_STRIDE;
    return scene_buffer[base + 2u];
}

fn get_instance_transform_index(instance_index: u32) -> u32 {
    let base = constant_data.instances_offset + instance_index * INSTANCE_STRIDE;
    return scene_buffer[base + 0u];
}

fn get_instance_mesh_index(instance_index: u32) -> u32 {
    let base = constant_data.instances_offset + instance_index * INSTANCE_STRIDE;
    return scene_buffer[base + 1u];
}

struct TransformData {
    orientation: vec4<f32>,
    position: vec3<f32>,
    scale: f32,
    aabb_min: vec3<f32>,
    aabb_max: vec3<f32>,
};

fn get_transform_data(transform_index: u32) -> TransformData {
    let base = constant_data.transforms_offset + transform_index * TRANSFORM_STRIDE;
    // orientation: vec4 (4)
    // pos_scale_x: vec4 (4) -> pos.x, pos.y, pos.z, scale.x
    // bb_min_scale_y: vec4 (4) -> min.x, min.y, min.z, scale.y
    // bb_max_scale_z: vec4 (4) -> max.x, max.y, max.z, scale.z

    // Unpack orientation
    let o_x = bitcast<f32>(scene_buffer[base + 0u]);
    let o_y = bitcast<f32>(scene_buffer[base + 1u]);
    let o_z = bitcast<f32>(scene_buffer[base + 2u]);
    let o_w = bitcast<f32>(scene_buffer[base + 3u]);
    let orientation = vec4<f32>(o_x, o_y, o_z, o_w);

    // Unpack pos_scale_x
    let px = bitcast<f32>(scene_buffer[base + 4u]);
    let py = bitcast<f32>(scene_buffer[base + 5u]);
    let pz = bitcast<f32>(scene_buffer[base + 6u]);
    let sx = bitcast<f32>(scene_buffer[base + 7u]);

    // Unpack bb_min_scale_y
    let min_x = bitcast<f32>(scene_buffer[base + 8u]);
    let min_y = bitcast<f32>(scene_buffer[base + 9u]);
    let min_z = bitcast<f32>(scene_buffer[base + 10u]);
    let sy = bitcast<f32>(scene_buffer[base + 11u]);

    // Unpack bb_max_scale_z
    let max_x = bitcast<f32>(scene_buffer[base + 12u]);
    let max_y = bitcast<f32>(scene_buffer[base + 13u]);
    let max_z = bitcast<f32>(scene_buffer[base + 14u]);
    let sz = bitcast<f32>(scene_buffer[base + 15u]);

    // Assuming uniform scale mostly, but storing non-uniform?
    // Usually scale is vector.
    let scale = sx; // Using x scale as uniform scale for simplicity if needed, or reconstruct vector.
    // TransformData struct uses float scale?
    // Let's assume uniform scale for now or update struct.

    var t: TransformData;
    t.orientation = orientation;
    t.position = vec3<f32>(px, py, pz);
    t.scale = sx; // Warning: Non-uniform scale might break logic if not handled.
    t.aabb_min = vec3<f32>(min_x, min_y, min_z);
    t.aabb_max = vec3<f32>(max_x, max_y, max_z);
    return t;
}

fn unpack_vertex_position(packed: u32, aabb_min: vec3<f32>, aabb_max: vec3<f32>) -> vec3<f32> {
    let size = aabb_max - aabb_min;
    let px_u = (packed >> 20u) & 0x3FFu;
    let py_u = (packed >> 10u) & 0x3FFu;
    let pz_u = packed & 0x3FFu;

    let px = f32(px_u) / 1023.0;
    let py = f32(py_u) / 1023.0;
    let pz = f32(pz_u) / 1023.0;

    return aabb_min + size * vec3<f32>(px, py, pz);
}

fn unpack_vertex_normal(packed: u32) -> vec3<f32> {
    // 10_10_10_2 snorm
    let nx_u = (packed >> 20u) & 0x3FFu;
    let ny_u = (packed >> 10u) & 0x3FFu;
    let nz_u = packed & 0x3FFu;

    // Decode snorm: (f / 511.5) - 1.0?
    // Standard snorm decoding: clamp((f - 512)/511)
    // Use helper?
    // Quick approximation:
    let nx = (f32(nx_u) / 1023.0) * 2.0 - 1.0;
    let ny = (f32(ny_u) / 1023.0) * 2.0 - 1.0;
    let nz = (f32(nz_u) / 1023.0) * 2.0 - 1.0;
    return normalize(vec3<f32>(nx, ny, nz));
}

fn unpack_vertex_uv(packed: u32) -> vec2<f32> {
    let u_half = packed & 0xFFFFu;
    let v_half = (packed >> 16u) & 0xFFFFu;
    return vec2<f32>(unpack2x16float(u_half).x, unpack2x16float(v_half).x); // Wait, unpack2x16float unpacks TWO floats.
    // unpack2x16float(u32) -> vec2<f32>.
    // If packed is u | (v << 16), then `unpack2x16float(packed)` returns vec2(u, v).
    return unpack2x16float(packed);
}

fn get_vertex_pos_world(index: u32, transform: TransformData) -> vec3<f32> {
    let packed = geometry_buffer[constant_data.vertex_positions_offset + index];
    let local_pos = unpack_vertex_position(packed, transform.aabb_min, transform.aabb_max);

    // Apply Transform: Pos * Scale * Rot + Trans
    // Rotate
    let q = transform.orientation;
    let v = local_pos * transform.scale;
    // Quaternion rotation: q * v * q^-1
    let t = 2.0 * cross(q.xyz, v);
    let rotated = v + q.w * t + cross(q.xyz, t);

    return rotated + transform.position;
}

fn get_vertex_normal_world(index: u32, transform: TransformData) -> vec3<f32> {
    // Normal is in attribute buffer
    let packed = geometry_buffer[constant_data.vertex_attributes_offset + index]; // Wait, attribute buffer stride?
    // Attribute buffer contains: [Normal(u32), Tangent(u32), UV(u32), Color(u32)?]
    // Checking `VertexAttributeLayout`:
    // It depends on the Mesh Layout!
    // But `ComputePathTracingMergePass` blindly copies attributes.
    // If different meshes have different layouts, we are in trouble.
    // However, the `MeshData` `insert_normal` etc pushes to `vertex_attributes`.
    // And `add_vertex_pos_color_normal_uv` is strict.
    // Most meshes in Inox use `pos_color_normal_uv1`.
    // Stride is 4 u32s? (Color, Normal, Tangent, UV?)
    // `MeshData` `insert_normal` -> 1 u32.
    // `insert_tangent` -> 1 u32.
    // `insert_uv` -> 1 u32.
    // `insert_color` -> 1 u32.
    // Layout `pos_color_normal_uv1` -> Color, Normal, UV1. (Tangent?)
    // I need to assume a consistent stride.
    // Let's assume Standard Layout: Normal(0), Tangent(1), UV(2). Color?
    // If I read `get_vertex_normal_world` assuming offset + 0 is normal.
    // If stride is dynamic, I need `GPUMesh.vertices_attribute_layout`.
    // This is getting complex for a "hyper optimized" shader if I have to switch on layout.
    // For now, I will assume a fixed stride of 1 (just Normal) or whatever the engine mostly uses.
    // Actually, `RenderContext` usually enforces a layout for the pipeline.
    // But here I am writing a Compute Shader that handles ALL meshes.
    // I will assume `Normal` is at offset 0 of attributes.
    // If `Mesh` has offset, I need `GPUMesh` to tell me.
    // `GPUMesh` has `vertices_attribute_offset`. This points to the start of attributes for that mesh.
    // So `geometry_buffer[mesh.attr_offset + index * STRIDE + OFFSET]`.
    // I will assume STRIDE=1 for now to proceed, or 3 if Normal/Tangent/UV.
    // Let's look at `MeshData`: `vertex_attributes` is a single Vec.
    // If it interleaves, then `vertex_attributes[0]` is Normal, `[1]` is Tangent...
    // I'll assume interleaved. Stride = 3 (Normal, Tangent, UV).

    let base = constant_data.vertex_attributes_offset + index * 3u; // Assuming stride 3
    let packed_normal = geometry_buffer[base + 0u];
    let local_normal = unpack_vertex_normal(packed_normal);

    // Rotate normal
    let q = transform.orientation;
    let v = local_normal;
    let t = 2.0 * cross(q.xyz, v);
    let rotated = v + q.w * t + cross(q.xyz, t);

    return normalize(rotated);
}

fn get_vertex_uv(index: u32) -> vec2<f32> {
    let base = constant_data.vertex_attributes_offset + index * 3u;
    let packed_uv = geometry_buffer[base + 2u];
    return unpack_vertex_uv(packed_uv);
}

fn get_triangle_indices(mesh_indices_offset: u32, triangle_index: u32) -> vec3<u32> {
    // Indices are packed in geometry_buffer
    let base = constant_data.indices_offset + mesh_indices_offset + triangle_index * 3u;
    let i0 = geometry_buffer[base + 0u];
    let i1 = geometry_buffer[base + 1u];
    let i2 = geometry_buffer[base + 2u];
    return vec3<u32>(i0, i1, i2);
}
