#import "common.inc"
#import "utils.inc"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> indices: Indices;
@group(0) @binding(2)
var<storage, read> vertices_positions: VerticesPositions;
@group(0) @binding(3)
var<storage, read> vertices_attributes: VerticesAttributes;
@group(0) @binding(4)
var<storage, read> instances: Instances;
@group(0) @binding(5)
var<storage, read> transforms: Transforms;
@group(0) @binding(6)
var<storage, read> meshes: Meshes;
@group(0) @binding(7)
var<storage, read> meshlets: Meshlets;

@group(1) @binding(0)
var<uniform> materials: Materials;
@group(1) @binding(1)
var<uniform> textures: Textures;
@group(1) @binding(2)
var<uniform> lights: Lights;
@group(1) @binding(3)
var<storage, read_write> data_buffer_0: array<f32>;
@group(1) @binding(4)
var<storage, read_write> data_buffer_1: array<f32>;
@group(1) @binding(5)
var<storage, read_write> data_buffer_2: array<f32>;

#import "texture_utils.inc"
#import "matrix_utils.inc"
#import "geom_utils.inc"
#import "material_utils.inc"
#import "pbr_utils.inc"
#import "visibility_utils.inc"
#import "pathtracing.inc"

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

    let data_index = (global_invocation_id.y * dimensions.x + global_invocation_id.x) * SIZE_OF_DATA_BUFFER_ELEMENT;

    // Read G-buffer from db0 (intact)
    let world_pos = read_gbuffer_world_pos(data_index);
    let normal = read_gbuffer_normal(data_index);

    // Skip empty pixels
    if (length(normal) < 0.01) {
        // Write inactive ray
        write_ray_data(data_index, vec3<f32>(0.), vec3<f32>(0., 1., 0.));
        write_throughput(data_index, vec3<f32>(0.), false);
        // Read combined direct lighting from db1 (diffuse + specular + emissive already accumulated)
        let total = vec3<f32>(data_buffer_1[data_index], data_buffer_1[data_index + 1u], data_buffer_1[data_index + 2u]);
        write_radiance(data_index, total, 0u);
        return;
    }

    var seed = (pixel * dimensions) ^ vec2<u32>(constant_data.frame_index << 16u);

    // Read combined direct lighting from db1 (diffuse + specular + emissive already accumulated by previous passes)
    let total_direct = vec3<f32>(data_buffer_1[data_index], data_buffer_1[data_index + 1u], data_buffer_1[data_index + 2u]);

    // Read material_id from db1[3] (packed by diffuse pass with tangent_sign)
    let packed_mat = u32(data_buffer_1[data_index + 3u]);
    let material_id = packed_mat & 0xFFu;

    // Read G-buffer from db2 (intact) for BRDF sampling
    let packed_tangent_xyz = octahedral_unmapping(unpack2x16float(u32(data_buffer_2[data_index])));
    let tangent_sign = f32((packed_mat >> 8u) & 1u);
    let tangent_w = select(-1.0, 1.0, tangent_sign > 0.5);
    let tangent = vec4<f32>(packed_tangent_xyz, tangent_w);
    let instance_id = u32(data_buffer_2[data_index + 1u]);
    let uv0 = unpack2x16float(u32(data_buffer_2[data_index + 2u]));

    var uv_set: array<vec4<f32>, 4>;
    uv_set[0] = vec4<f32>(uv0, 0., 0.);
    var pixel_data = PixelData(
        world_pos, material_id,
        vec4<f32>(1.),
        normal, instance_id,
        tangent, uv_set
    );

    // Evaluate material for BRDF sampling (need roughness, metallic, f0, c_diff)
    var material_info = compute_color_from_material(material_id, &pixel_data, 0xFFFFFFFFu);

    // Compute view direction (from surface to camera)
    let camera_pos = vec3<f32>(
        constant_data.inv_view[3][0],
        constant_data.inv_view[3][1],
        constant_data.inv_view[3][2]
    );
    let view = normalize(camera_pos - world_pos);

    // Importance-sample BRDF for first bounce direction
    let brdf_sample = sample_brdf(normal, view, material_info.perceptual_roughness, material_info.metallic, &seed);

    // Compute initial throughput weight
    let NdotL = max(dot(normal, brdf_sample.direction), 0.);
    let brdf_eval = eval_brdf(&material_info, normal, view, brdf_sample.direction, brdf_sample.is_specular);
    var throughput: vec3<f32> = vec3<f32>(0.);
    if (brdf_sample.pdf > 0.) {
        throughput = brdf_eval * NdotL / brdf_sample.pdf;
    }

    let is_active = NdotL > 0. && length(throughput) > MATH_EPSILON;

    // Write bounce ray
    let ray_origin = world_pos + brdf_sample.direction * HIT_EPSILON;
    write_ray_data(data_index, ray_origin, brdf_sample.direction);

    // Write total direct radiance (already combined: diffuse + specular + emissive)
    write_radiance(data_index, total_direct, material_id);

    // Write throughput for indirect bounces
    write_throughput(data_index, throughput, is_active);
}
