#import "common.inc"
#import "pathtracing_common.inc"

@group(0) @binding(0) var<uniform> constant_data: ConstantData;
@group(0) @binding(1) var<storage, read_write> hits: array<RayHit>;
@group(0) @binding(2) var<storage, read_write> rays: array<Ray>;
@group(0) @binding(3) var<storage, read_write> counters: PathTracingCounters;
@group(0) @binding(4) var visibility_texture: texture_2d<u32>;
@group(0) @binding(5) var depth_texture: texture_depth_2d;
@group(0) @binding(6) var<storage, read_write> data_buffer_1: array<f32>;

// Need access to geometry buffers to compute barycentrics
@group(1) @binding(0) var<storage, read> indices: Indices;
@group(1) @binding(1) var<storage, read> vertices_positions: VerticesPositions;
@group(1) @binding(2) var<storage, read> vertices_attributes: VerticesAttributes;
@group(1) @binding(3) var<storage, read> instances: Instances;
@group(1) @binding(4) var<storage, read> transforms: Transforms;
@group(1) @binding(5) var<storage, read> meshes: Meshes;
@group(1) @binding(6) var<storage, read> meshlets: Meshlets;

#import "utils.inc"
#import "texture_utils.inc"
#import "matrix_utils.inc"
#import "geom_utils.inc"

const WORKGROUP_SIZE: u32 = 8u;

@compute
@workgroup_size(WORKGROUP_SIZE, WORKGROUP_SIZE, 1)
fn main(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let dimensions = vec2<u32>(DEFAULT_WIDTH, DEFAULT_HEIGHT);
    let pixel = vec2<u32>(global_invocation_id.x, global_invocation_id.y);

    if (pixel.x >= dimensions.x || pixel.y >= dimensions.y) {
        return;
    }

    let data_index = (global_invocation_id.y * u32(constant_data.screen_width) + global_invocation_id.x) * 4u;
    data_buffer_1[data_index] = 0.0;
    data_buffer_1[data_index + 1u] = 0.0;
    data_buffer_1[data_index + 2u] = 0.0;
    data_buffer_1[data_index + 3u] = 0.0;

    let visibility_dimensions = textureDimensions(visibility_texture);
    let visibility_scale = vec2<f32>(visibility_dimensions) / vec2<f32>(dimensions);
    let visibility_pixel = vec2<u32>(vec2<f32>(pixel) * visibility_scale);
    let visibility_value = textureLoad(visibility_texture, visibility_pixel, 0);
    let visibility_id = visibility_value.r;

    if (visibility_id == 0u || (visibility_id & 0xFFFFFFFFu) == 0xFF000000u) {
        return;
    }

    let depth_dimensions = textureDimensions(depth_texture);
    let depth_scale = vec2<f32>(depth_dimensions) / vec2<f32>(dimensions);
    let depth_pixel = vec2<u32>(vec2<f32>(pixel) * depth_scale);
    let depth = textureLoad(depth_texture, depth_pixel, 0);

    let hit_point = pixel_to_world(depth_pixel, depth_dimensions, depth);
    let distance = length(hit_point - constant_data.view[3].xyz);

    // Compute Barycentrics
    let instance_id = (visibility_id >> 8u) - 1u;
    let primitive_id = visibility_id & 255u;

    let instance = instances.data[instance_id];
    let meshlet = meshlets.data[instance.meshlet_id];
    let index_offset = meshlet.indices_offset + (primitive_id * 3u);

    let mesh_id = meshlet.mesh_index;
    let mesh = meshes.data[mesh_id];
    let position_offset = mesh.vertices_position_offset;

    let vert_indices = vec3<u32>(indices.data[index_offset], indices.data[index_offset + 1u], indices.data[index_offset + 2u]);

    let transform = transforms.data[instance.transform_id];
    let orientation = transform.orientation;
    let position = transform.position_scale_x.xyz;
    let scale = vec3<f32>(transform.position_scale_x.w, transform.bb_min_scale_y.w, transform.bb_min_scale_y.w);

    let min = transform.bb_min_scale_y.xyz;
    let size = abs(transform.bb_max_scale_z.xyz - min);
    let p1 = min + unpack_unorm_to_3_f32(vertices_positions.data[vert_indices.x + position_offset]) * size;
    let p2 = min + unpack_unorm_to_3_f32(vertices_positions.data[vert_indices.y + position_offset]) * size;
    let p3 = min + unpack_unorm_to_3_f32(vertices_positions.data[vert_indices.z + position_offset]) * size;
    let v1 = transform_vector(p1, position, orientation, scale);
    let v2 = transform_vector(p2, position, orientation, scale);
    let v3 = transform_vector(p3, position, orientation, scale);

    let barycentrics = compute_barycentrics_3d(v1,v2,v3,hit_point);

    let pixel_index = pixel.y * dimensions.x + pixel.x;

    var hit: RayHit;
    hit.instance_id = instance_id;
    hit.primitive_index = primitive_id;
    hit.barycentrics = barycentrics.xy;
    hit.t = distance;
    hit.pixel_index = pixel_index;

    hit.direction = normalize(hit_point - constant_data.view[3].xyz);
    hit.throughput = vec3<f32>(1.0);

    hits[pixel_index] = hit;

    var ray: Ray;
    ray.origin = constant_data.view[3].xyz;
    ray.t_min = 0.0;
    ray.direction = hit.direction;
    ray.t_max = distance;
    ray.throughput = vec3<f32>(1.0);
    ray.pixel_index = pixel_index;
    ray.depth = 0u;
    rays[pixel_index] = ray;

    // Use atomic counter only if we compact. For now we use dense indexing matching pixel_index.
    atomicAdd(&counters.hit_count, 1u);
}
