#import "utils.wgsl"
#import "common.wgsl"


struct PbrData {
    width: u32,
    height: u32,
    gbuffer_1: u32,
    gbuffer_2: u32,
    gbuffer_3: u32,
    depth: u32,
    _padding_2: u32,
    _padding_3: u32,
};


@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<uniform> pbr_data: PbrData;
@group(0) @binding(2)
var<storage, read> meshes: Meshes;
@group(0) @binding(3)
var<storage, read> materials: Materials;
@group(0) @binding(4)
var<storage, read> textures: Textures;
@group(0) @binding(5)
var<storage, read> lights: Lights;

@group(1) @binding(0)
var render_target: texture_storage_2d_array<rgba8unorm, read_write>;



#import "texture_utils.wgsl"
#import "material_utils.wgsl"



@compute
@workgroup_size(64, 1, 1)
fn main(
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, 
    @builtin(local_invocation_index) local_invocation_index: u32, 
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    let pixel = vec2<i32>(i32(global_invocation_id.x), i32(global_invocation_id.y));
    if (pixel.x >= i32(pbr_data.width) || pixel.y >= i32(pbr_data.height))
    {
        return;
    }
    
    let gbuffer_1 = load(pbr_data.gbuffer_1, pixel);
    let gbuffer_2 = load(pbr_data.gbuffer_2, pixel);

    var color = vec4<f32>(0., 0., 0., 0.);
    
    let mesh_id = u32(gbuffer_2.z);
    let vertex_color = u32(gbuffer_1.w);
    if mesh_id == 0u && vertex_color == 1u {
        textureStore(render_target, pixel.xy, 0, color);
        return;
    }

    let display_meshlets = constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS;
    if (display_meshlets != 0u) {
        let meshlet_id = hash(u32(gbuffer_2.w));
        color = vec4<f32>(vec3<f32>(
            f32(meshlet_id & 255u), 
            f32((meshlet_id >> 8u) & 255u), 
            f32((meshlet_id >> 16u) & 255u)) / 255., 
            1.
        );
    } else {
        let gbuffer_3 = load(pbr_data.gbuffer_3, pixel);

        let material_id = u32(meshes.data[mesh_id].material_index);
        let texture_color = sample_material_texture(gbuffer_3, material_id, TEXTURE_TYPE_BASE_COLOR);
        let vertex_color = unpack_unorm_to_4_f32(vertex_color);
        color = vec4<f32>(vertex_color.rgb * texture_color.rgb, vertex_color.a);

        let alpha = compute_alpha(material_id, vertex_color.a);
        if alpha < 0. {
            return;
        }

        let world_pos = gbuffer_1.xyz;
        let n = unpack_normal(gbuffer_2.xy);
        color = pbr(world_pos, n, material_id, color, gbuffer_3);
    }

    textureStore(render_target, pixel.xy, 0, color);
}