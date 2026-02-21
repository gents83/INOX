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
var<storage, read_write> data_buffer_0: array<RayPackedData>;
@group(1) @binding(4)
var<storage, read_write> data_buffer_1: array<RadiancePackedData>;
@group(1) @binding(5)
var<storage, read_write> data_buffer_2: array<ThroughputPackedData>;

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

    let data_index = global_invocation_id.y * dimensions.x + global_invocation_id.x;

    // Read G-buffer
    let world_pos = read_gbuffer_world_pos(data_index);
    let normal = read_gbuffer_normal(data_index);
    let material_id = read_gbuffer_material_id(data_index);
    let tangent = read_gbuffer_tangent(data_index);
    let instance_id = read_gbuffer_instance_id(data_index);

    // Skip empty pixels
    if (length(normal) < 0.01) {
        data_buffer_1[data_index].data = vec4<f32>(0.);
        return;
    }

    // Read shadow + AO
    let shadow_mask = read_shadow_mask(data_index);
    let ao_factor = read_ao_factor(data_index);

    // Reconstruct PixelData for material evaluation
    var uv_set: array<vec4<f32>, 4>;
    let uv0 = read_gbuffer_uv0(data_index);
    uv_set[0] = vec4<f32>(uv0, 0., 0.);
    var pixel_data = PixelData(
        world_pos, material_id,
        vec4<f32>(1.), // color
        normal, instance_id,
        tangent, uv_set
    );

    // Evaluate full PBR material (shadows are now applied inside this function)
    var material_info = compute_color_from_material(material_id, &pixel_data, shadow_mask);

    // Extract diffuse contribution and modulate by AO
    // Use the pre-computed diffuse which includes IBL + punctual lighting
    // Apply AO to the IBL component
    let diffuse_radiance = material_info.f_diffuse * ao_factor + material_info.f_diffuse_ibl * ao_factor;

    // Write diffuse radiance to data_buffer_1
    // MaterialID and TangentW are safely stored in db0, so we don't need to pack them here.
    write_radiance(data_index, diffuse_radiance, 0u);
}
