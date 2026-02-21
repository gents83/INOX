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
        return;
    }

    // Read material_id + tangent_sign packed by diffuse pass in db1[3]
    let packed_mat = u32(data_buffer_1[data_index + 3u]);
    let material_id = packed_mat & 0xFFu;
    let tangent_sign = f32((packed_mat >> 8u) & 1u);
    let tangent_w = select(-1.0, 1.0, tangent_sign > 0.5);

    // Read G-buffer from db2 (intact - diffuse pass didn't touch db2)
    let packed_tangent_xyz = octahedral_unmapping(unpack2x16float(u32(data_buffer_2[data_index])));
    let tangent = vec4<f32>(packed_tangent_xyz, tangent_w);
    let instance_id = u32(data_buffer_2[data_index + 1u]);
    let uv0 = unpack2x16float(u32(data_buffer_2[data_index + 2u]));

    // Read shadow mask from db2[3] (intact)
    let shadow_mask = read_shadow_mask(data_index);

    // Reconstruct PixelData
    var uv_set: array<vec4<f32>, 4>;
    uv_set[0] = vec4<f32>(uv0, 0., 0.);
    var pixel_data = PixelData(
        world_pos, material_id,
        vec4<f32>(1.),
        normal, instance_id,
        tangent, uv_set
    );

    // Evaluate full PBR material
    let material_info = compute_color_from_material(material_id, &pixel_data, shadow_mask);

    // Specular contribution + emissive
    let specular_radiance = material_info.f_specular;
    let emissive = material_info.f_emissive;

    // Accumulate specular + emissive on top of diffuse already in db1
    data_buffer_1[data_index] += specular_radiance.x + emissive.x;
    data_buffer_1[data_index + 1u] += specular_radiance.y + emissive.y;
    data_buffer_1[data_index + 2u] += specular_radiance.z + emissive.z;
    // db1[3] keeps the packed material_id + tangent_sign
}
