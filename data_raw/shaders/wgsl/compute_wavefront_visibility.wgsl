#import "common.inc"
#import "packed_scene.inc"
#import "utils.inc"
#import "matrix_utils.inc"
#import "geom_utils.inc"

struct Reservoir { selected_light_id: u32, candidate_count: u32, weight_sum: f32, target_pdf: f32, selected_weight: f32, age: u32, flags: u32, confidence: f32 };
struct AoState { visibility: f32, radius: f32, sample_count: u32, flags: u32 };
struct Accumulator { radiance_and_weight: vec4<f32>, first_and_second_moment: vec2<f32>, history_length: u32, flags: u32 };
struct BvhNode { min: vec3<f32>, miss: i32, max: vec3<f32>, reference: i32 };

@group(0) @binding(0) var<uniform> constant_data: ConstantData;
@group(0) @binding(1) var<storage, read> scene_data: array<u32>;
@group(1) @binding(0) var<storage, read> gbuffer_world: array<f32>;
@group(1) @binding(1) var<storage, read> gbuffer_material: array<f32>;
@group(1) @binding(2) var<storage, read> reservoirs: array<Reservoir>;
@group(1) @binding(3) var<storage, read_write> direct_accumulator: array<Accumulator>;
@group(1) @binding(4) var<storage, read_write> ao_state: array<AoState>;

fn load_bvh(index: u32) -> BvhNode {
    if (!scene_index_valid(7u, index, 8u)) { return BvhNode(vec3<f32>(0.0), -1, vec3<f32>(0.0), -1); }
    let base = table_offset(7u) + index * 8u;
    return BvhNode(load_vec3(base), bitcast<i32>(scene_data[base + 3u]), load_vec3(base + 4u), bitcast<i32>(scene_data[base + 7u]));
}
fn aabb_hit(origin: vec3<f32>, direction: vec3<f32>, max_distance: f32, aabb_min: vec3<f32>, aabb_max: vec3<f32>) -> bool {
    let inv = 1.0 / direction;
    let t0 = (aabb_min - origin) * inv; let t1 = (aabb_max - origin) * inv;
    let near = max(max(min(t0.x, t1.x), min(t0.y, t1.y)), min(t0.z, t1.z));
    let far = min(min(max(t0.x, t1.x), max(t0.y, t1.y)), max(t0.z, t1.z));
    return near <= far && far > 0.0 && near < max_distance;
}
fn tri_hit(origin: vec3<f32>, direction: vec3<f32>, max_distance: f32, v0: vec3<f32>, v1: vec3<f32>, v2: vec3<f32>) -> bool {
    let e1 = v1 - v0; let e2 = v2 - v0; let p = cross(direction, e2); let det = dot(e1, p);
    if (det <= MATH_EPSILON) { return false; }
    let inv = 1.0 / det; let s = origin - v0; let u = dot(s, p) * inv; let q = cross(s, e1); let v = dot(direction, q) * inv; let t = dot(e2, q) * inv;
    return u >= 0.0 && v >= 0.0 && u + v <= 1.0 && t > HIT_EPSILON && t < max_distance;
}
fn occluded(origin: vec3<f32>, direction: vec3<f32>, max_distance: f32) -> bool {
    var tlas_stack: array<i32, 64>; var tlas_sp = 1u; tlas_stack[0] = i32(constant_data.tlas_starting_index); var steps = 0u;
    while (tlas_sp > 0u && steps < 256u) {
        steps += 1u; tlas_sp -= 1u; let node_index = tlas_stack[tlas_sp]; if (node_index < 0) { continue; }
        let node = load_bvh(u32(node_index)); if (!aabb_hit(origin, direction, max_distance, node.min, node.max)) { continue; }
        if (node.reference < 0) {
            let left = node_index + 1; let left_node = load_bvh(u32(left));
            if (tlas_sp < 62u && left_node.miss >= 0) { tlas_stack[tlas_sp] = left_node.miss; tlas_sp += 1u; }
            if (tlas_sp < 64u) { tlas_stack[tlas_sp] = left; tlas_sp += 1u; }
            continue;
        }
        let instance = u32(node.reference);
        if (!scene_index_valid(3u, instance, 4u)) { continue; }
        let ib = table_offset(3u) + instance * 4u;
        let transform_id = scene_data[ib];
        let mesh_index = scene_data[ib + 1u];
        if (!scene_index_valid(4u, transform_id, 16u) || !scene_index_valid(5u, mesh_index, 16u)) { continue; }
        let transform_base = table_offset(4u) + transform_id * 16u;
        let mb = table_offset(5u) + mesh_index * 16u;
        let orientation = vec4<f32>(load_f32(transform_base), load_f32(transform_base + 1u), load_f32(transform_base + 2u), load_f32(transform_base + 3u));
        let position = load_vec3(transform_base + 4u); let scale = vec3<f32>(load_f32(transform_base + 7u), load_f32(transform_base + 11u), load_f32(transform_base + 15u));
        let inv_matrix = matrix_inverse(transform_matrix(position, orientation, scale)); let local_origin = (inv_matrix * vec4<f32>(origin, 1.0)).xyz; let local_direction = (inv_matrix * vec4<f32>(direction, 0.0)).xyz;
        var blas_stack: array<i32, 64>; var blas_sp = 1u; blas_stack[0] = bitcast<i32>(scene_data[mb + 5u]); var blas_steps = 0u;
        while (blas_sp > 0u && blas_steps < 256u) {
            blas_steps += 1u; blas_sp -= 1u; let bi = blas_stack[blas_sp]; if (bi < 0) { continue; }
            let bn = load_bvh(u32(bi)); if (!aabb_hit(local_origin, local_direction, max_distance, bn.min, bn.max)) { continue; }
            if (bn.reference < 0) {
                let left = bi + 1; let left_node = load_bvh(u32(left));
                if (blas_sp < 62u && left_node.miss >= 0) { blas_stack[blas_sp] = left_node.miss; blas_sp += 1u; }
                if (blas_sp < 64u) { blas_stack[blas_sp] = left; blas_sp += 1u; }
                continue;
            }
            let meshlet_start = scene_data[mb + 4u];
            let relative_meshlet = u32(bn.reference);
            if (!scene_relative_index_valid(6u, meshlet_start, relative_meshlet, 20u)) { continue; }
            let meshlet_index = meshlet_start + relative_meshlet;
            let meshlet_base = table_offset(6u) + meshlet_index * 20u;
            let io = scene_data[meshlet_base + 1u];
            let index_count = scene_data[meshlet_base + 2u];
            if (index_count < 3u || (index_count % 3u) != 0u || !range_has_words(0u, io, index_count)) { continue; }
            let count = index_count / 3u;
            let vbase = scene_data[mb];
            if (!range_has_words(4u, 8u, 8u)) { continue; }
            let bbmin = load_vec3(transform_base + 8u); let bbmax = load_vec3(transform_base + 12u); let size = abs(bbmax - bbmin);
            for (var tri = 0u; tri < count; tri += 1u) {
                if (!range_has_indexed_words(0u, io, tri, 3u, 3u)) { continue; }
                let index_offset = io + tri * 3u;
                let base = table_offset(0u) + index_offset;
                let i0 = scene_data[base]; let i1 = scene_data[base + 1u]; let i2 = scene_data[base + 2u];
                if (!range_has_words_at(1u, vbase, i0, 1u) || !range_has_words_at(1u, vbase, i1, 1u)
                    || !range_has_words_at(1u, vbase, i2, 1u)) { continue; }
                let vertex_i0 = vbase + i0;
                let vertex_i1 = vbase + i1;
                let vertex_i2 = vbase + i2;
                let p0 = bbmin + unpack_unorm_to_3_f32(scene_data[table_offset(1u) + vertex_i0]) * size; let p1 = bbmin + unpack_unorm_to_3_f32(scene_data[table_offset(1u) + vertex_i1]) * size; let p2 = bbmin + unpack_unorm_to_3_f32(scene_data[table_offset(1u) + vertex_i2]) * size;
                if (tri_hit(origin, direction, max_distance, transform_vector(p0, position, orientation, scale), transform_vector(p1, position, orientation, scale), transform_vector(p2, position, orientation, scale))) { return true; }
            }
        }
    }
    return false;
}

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (!packed_scene_valid()) { return; }
    let width = u32(constant_data.screen_width); let height = u32(constant_data.screen_height);
    if (id.x >= width || id.y >= height) { return; }
    let pixel = id.y * width + id.x;
    let o = pixel * SIZE_OF_DATA_BUFFER_ELEMENT;
    if (o > arrayLength(&gbuffer_world) || arrayLength(&gbuffer_world) - o < SIZE_OF_DATA_BUFFER_ELEMENT
        || arrayLength(&gbuffer_material) - o < SIZE_OF_DATA_BUFFER_ELEMENT
        || pixel >= arrayLength(&reservoirs) || pixel >= arrayLength(&direct_accumulator)
        || pixel >= arrayLength(&ao_state)) { return; }
    let position = vec3<f32>(gbuffer_world[o], gbuffer_world[o + 1u], gbuffer_world[o + 2u]);
    let normal = octahedral_unmapping(unpack2x16float(u32(gbuffer_world[o + 3u])));
    let reservoir = reservoirs[pixel];
    let light_count = table_count(10u) / 16u;
    var visibility = 1.0;
    if (light_count > 0u && reservoir.selected_light_id < light_count
        && range_has_words(10u, reservoir.selected_light_id * 16u, 16u)) {
        let lb = table_offset(10u) + reservoir.selected_light_id * 16u;
        let lp = load_vec3(lb); let to_light = lp - position; let distance = length(to_light);
        if (dot(normal, to_light) > 0.0) { visibility = select(1.0, 0.0, occluded(position + normal * HIT_EPSILON, normalize(to_light), distance)); }
    }
    let seed = f32((pixel ^ constant_data.frame_index) & 1023u) / 1023.0; let phi = 6.2831853 * seed; let tangent = normalize(select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), abs(normal.x) > 0.5)); let bitangent = normalize(cross(normal, tangent)); let ao_dir = normalize(tangent * cos(phi) * 0.7 + bitangent * sin(phi) * 0.7 + normal * 0.7); let ao_visibility = select(0.0, 1.0, !occluded(position + normal * HIT_EPSILON, ao_dir, 1.0));
    var emissive = vec3<f32>(0.0);
    let material_value = gbuffer_material[o];
    if (material_value >= 0.0 && material_value < f32(MAX_NUM_MATERIALS)) {
        let material_id = u32(material_value);
        if (scene_index_valid(8u, material_id, 48u)) {
            emissive = max(vec3<f32>(
                load_scene_f32(table_offset(8u) + material_id * 48u + 24u),
                load_scene_f32(table_offset(8u) + material_id * 48u + 25u),
                load_scene_f32(table_offset(8u) + material_id * 48u + 26u)
            ), vec3<f32>(0.0)) * max(load_scene_f32(table_offset(8u) + material_id * 48u + 27u), 0.0);
        }
    }
    // Visibility may reject a sampled light on the same emissive surface;
    // restore its own emission after shadow masking without duplicating it
    // when the direct sample remained visible.
    let direct = direct_accumulator[pixel].radiance_and_weight.xyz * visibility
        + emissive * (1.0 - visibility);
    direct_accumulator[pixel].radiance_and_weight = vec4<f32>(direct, 1.0); ao_state[pixel] = AoState(ao_visibility, 1.0, 1u, 0u);
}
