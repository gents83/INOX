#import "common.inc"
#import "utils.inc"
#import "matrix_utils.inc"
#import "geom_utils.inc"
#import "wavefront_records.inc"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> scene_data: array<u32>;

@group(1) @binding(0)
var<storage, read> ray_queue_header: array<WavefrontQueueHeader>;
@group(1) @binding(1)
var<storage, read> ray_queue: array<WavefrontRay>;
@group(1) @binding(2)
var<storage, read_write> hit_queue_header: array<WavefrontQueueHeader>;
@group(1) @binding(3)
var<storage, read_write> hit_queue: array<WavefrontHit>;

fn append_traversal_hit(hit: WavefrontHit) {
    let slot = atomicAdd(&hit_queue_header[0].count, 1u);
    if (slot < hit_queue_header[0].capacity) {
        hit_queue[slot] = hit;
    } else {
        atomicStore(&hit_queue_header[0].overflowed, 1u);
    }
}

struct BvhNode {
    min: vec3<f32>,
    miss: i32,
    max: vec3<f32>,
    reference: i32,
};

struct MeshInfo {
    vertices_position_offset: u32,
    material_index: i32,
    meshlets_offset: u32,
    blas_index: u32,
    indices_offset: u32,
};

fn table_offset(kind: u32) -> u32 {
    return scene_data[4u + kind * 4u];
}

fn load_bvh(index: u32) -> BvhNode {
    let base = table_offset(7u) + index * 8u;
    return BvhNode(
        vec3<f32>(
            bitcast<f32>(scene_data[base]),
            bitcast<f32>(scene_data[base + 1u]),
            bitcast<f32>(scene_data[base + 2u])
        ),
        bitcast<i32>(scene_data[base + 3u]),
        vec3<f32>(
            bitcast<f32>(scene_data[base + 4u]),
            bitcast<f32>(scene_data[base + 5u]),
            bitcast<f32>(scene_data[base + 6u])
        ),
        bitcast<i32>(scene_data[base + 7u])
    );
}

fn load_mesh(index: u32) -> MeshInfo {
    let base = table_offset(5u) + index * 16u;
    return MeshInfo(
        scene_data[base],
        bitcast<i32>(scene_data[base + 3u]),
        scene_data[base + 4u],
        scene_data[base + 5u],
        scene_data[base + 6u]
    );
}

fn intersect_aabb(
    origin: vec3<f32>,
    direction: vec3<f32>,
    max_distance: f32,
    aabb_min: vec3<f32>,
    aabb_max: vec3<f32>,
) -> f32 {
    let inverse_direction = 1.0 / direction;
    let t0 = (aabb_min - origin) * inverse_direction;
    let t1 = (aabb_max - origin) * inverse_direction;
    let near_distance = max(max(min(t0.x, t1.x), min(t0.y, t1.y)), min(t0.z, t1.z));
    let far_distance = min(min(max(t0.x, t1.x), max(t0.y, t1.y)), max(t0.z, t1.z));
    return select(max_distance, near_distance, near_distance <= far_distance && far_distance > 0.0 && near_distance < max_distance);
}

fn intersect_triangle(
    origin: vec3<f32>,
    direction: vec3<f32>,
    max_distance: f32,
    v0: vec3<f32>,
    v1: vec3<f32>,
    v2: vec3<f32>,
) -> vec3<f32> {
    let e1 = v1 - v0;
    let e2 = v2 - v0;
    let p = cross(direction, e2);
    let determinant = dot(e1, p);
    if (determinant <= MATH_EPSILON) {
        return vec3<f32>(max_distance, 0.0, 0.0);
    }
    let inverse_determinant = 1.0 / determinant;
    let s = origin - v0;
    let u = dot(s, p) * inverse_determinant;
    let q = cross(s, e1);
    let v = dot(direction, q) * inverse_determinant;
    let distance = dot(e2, q) * inverse_determinant;
    if (u < 0.0 || v < 0.0 || u + v > 1.0 || distance <= HIT_EPSILON || distance >= max_distance) {
        return vec3<f32>(max_distance, 0.0, 0.0);
    }
    return vec3<f32>(distance, u, v);
}

fn trace_blas(
    world_origin: vec3<f32>,
    world_direction: vec3<f32>,
    local_origin: vec3<f32>,
    local_direction: vec3<f32>,
    mesh: MeshInfo,
    instance_id: u32,
    position: vec3<f32>,
    orientation: vec4<f32>,
    scale: vec3<f32>,
    bb_min: vec3<f32>,
    bb_max: vec3<f32>,
    initial_distance: f32,
) -> WavefrontHit {
    var best = WavefrontHit(initial_distance, 0u, instance_id, 0u, vec2<f32>(0.0), 0u, 0u);
    var stack: array<i32, 64>;
    var stack_pointer = 1u;
    stack[0] = i32(mesh.blas_index);
    var steps = 0u;
    let hit_size = bb_max - bb_min;
    while (stack_pointer > 0u && steps < 256u) {
        steps += 1u;
        stack_pointer -= 1u;
        let node_index = stack[stack_pointer];
        if (node_index < 0) {
            continue;
        }
        let node = load_bvh(u32(node_index));
        if (intersect_aabb(local_origin, local_direction, best.distance, node.min, node.max) >= best.distance) {
            continue;
        }
        if (node.reference < 0) {
            let left_index = node_index + 1;
            let left_node = load_bvh(u32(left_index));
            if (stack_pointer < 62u && left_node.miss >= 0) {
                stack[stack_pointer] = left_node.miss;
                stack_pointer += 1u;
            }
            if (stack_pointer < 64u) {
                stack[stack_pointer] = left_index;
                stack_pointer += 1u;
            }
            continue;
        }
        let meshlet_base = table_offset(6u) + (mesh.meshlets_offset + u32(node.reference)) * 20u;
        let indices_offset = scene_data[meshlet_base + 1u];
        let indices_count = scene_data[meshlet_base + 2u];
        let triangle_count = indices_count / 3u;
        for (var triangle = 0u; triangle < triangle_count; triangle += 1u) {
            let index_base = table_offset(0u) + indices_offset + triangle * 3u;
            let i0 = scene_data[index_base];
            let i1 = scene_data[index_base + 1u];
            let i2 = scene_data[index_base + 2u];
            let p0 = bb_min + unpack_unorm_to_3_f32(scene_data[table_offset(1u) + mesh.vertices_position_offset + i0]) * hit_size;
            let p1 = bb_min + unpack_unorm_to_3_f32(scene_data[table_offset(1u) + mesh.vertices_position_offset + i1]) * hit_size;
            let p2 = bb_min + unpack_unorm_to_3_f32(scene_data[table_offset(1u) + mesh.vertices_position_offset + i2]) * hit_size;
            let result = intersect_triangle(world_origin, world_direction,
                best.distance,
                transform_vector(p0, position, orientation, scale),
                transform_vector(p1, position, orientation, scale),
                transform_vector(p2, position, orientation, scale));
            if (result.x < best.distance) {
                best.distance = result.x;
                best.visibility_id = ((instance_id + 1u) << 8u) | triangle;
                best.primitive_id = triangle;
                best.barycentric = result.yz;
                best.flags = 1u;
            }
        }
    }
    return best;
}

fn trace_scene(origin: vec3<f32>, direction: vec3<f32>) -> WavefrontHit {
    var best = WavefrontHit(MAX_TRACING_DISTANCE, 0u, 0u, 0u, vec2<f32>(0.0), 0u, 0u);
    var stack: array<i32, 64>;
    var stack_pointer = 1u;
    stack[0] = i32(constant_data.tlas_starting_index);
    var steps = 0u;
    while (stack_pointer > 0u && steps < 256u) {
        steps += 1u;
        stack_pointer -= 1u;
        let node_index = stack[stack_pointer];
        if (node_index < 0) {
            continue;
        }
        let node = load_bvh(u32(node_index));
        if (intersect_aabb(origin, direction, best.distance, node.min, node.max) >= best.distance) {
            continue;
        }
        if (node.reference < 0) {
            let left_index = node_index + 1;
            let left_node = load_bvh(u32(left_index));
            if (stack_pointer < 62u && left_node.miss >= 0) {
                stack[stack_pointer] = left_node.miss;
                stack_pointer += 1u;
            }
            if (stack_pointer < 64u) {
                stack[stack_pointer] = left_index;
                stack_pointer += 1u;
            }
            continue;
        }
        let instance_id = u32(node.reference);
        let instance_base = table_offset(3u) + instance_id * 4u;
        let mesh = load_mesh(scene_data[instance_base + 1u]);
        let transform_base = table_offset(4u) + scene_data[instance_base] * 16u;
        let orientation = vec4<f32>(
            bitcast<f32>(scene_data[transform_base]),
            bitcast<f32>(scene_data[transform_base + 1u]),
            bitcast<f32>(scene_data[transform_base + 2u]),
            bitcast<f32>(scene_data[transform_base + 3u]));
        let position_scale = vec4<f32>(
            bitcast<f32>(scene_data[transform_base + 4u]),
            bitcast<f32>(scene_data[transform_base + 5u]),
            bitcast<f32>(scene_data[transform_base + 6u]),
            bitcast<f32>(scene_data[transform_base + 7u]));
        let bb_min_scale = vec4<f32>(
            bitcast<f32>(scene_data[transform_base + 8u]),
            bitcast<f32>(scene_data[transform_base + 9u]),
            bitcast<f32>(scene_data[transform_base + 10u]),
            bitcast<f32>(scene_data[transform_base + 11u]));
        let bb_max_scale = vec4<f32>(
            bitcast<f32>(scene_data[transform_base + 12u]),
            bitcast<f32>(scene_data[transform_base + 13u]),
            bitcast<f32>(scene_data[transform_base + 14u]),
            bitcast<f32>(scene_data[transform_base + 15u]));
        let position = position_scale.xyz;
        let scale = vec3<f32>(position_scale.w, bb_min_scale.w, bb_max_scale.w);
        let inverse_matrix = matrix_inverse(transform_matrix(position, orientation, scale));
        let local_origin = (inverse_matrix * vec4<f32>(origin, 1.0)).xyz;
        let local_direction = (inverse_matrix * vec4<f32>(direction, 0.0)).xyz;
        let candidate = trace_blas(
            origin, direction, local_origin, local_direction, mesh, instance_id,
            position, orientation, scale, bb_min_scale.xyz, bb_max_scale.xyz, best.distance);
        if (candidate.distance < best.distance) {
            best = candidate;
        }
    }
    return best;
}

@compute
@workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let ray_index = global_invocation_id.x;
    let ray_count = min(atomicLoad(&ray_queue_header[0].count), ray_queue_header[0].capacity);
    if (ray_index >= ray_count) {
        return;
    }
    let ray = ray_queue[ray_index];
    let hit = trace_scene(ray.origin_and_t_min.xyz, ray.direction_and_t_max.xyz);
    var result = hit;
    result.ray_index = ray_index;
    append_traversal_hit(result);
}
