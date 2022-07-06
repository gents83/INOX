#import "utils.wgsl"
#import "common.wgsl"

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) params: vec4<f32>,
    @location(1) normals: vec4<f32>,
    @location(2) tex_coords: vec4<f32>,
};

struct FragmentOutput {
    @location(0) albedo: vec4<f32>,
    @location(1) normals: vec4<f32>,
    @location(2) material_params: vec4<f32>,
};


@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> positions_and_colors: PositionsAndColors;
@group(0) @binding(2)
var<storage, read> normals: NormalsAndPadding;
@group(0) @binding(3)
var<storage, read> uvs: UVs;

@group(1) @binding(0)
var<storage, read> matrices: Matrices;
@group(1) @binding(1)
var<storage, read> meshes: Meshes;
@group(1) @binding(2)
var<storage, read> meshlets: Meshlets;
@group(1) @binding(3)
var<storage, read> materials: Materials;
@group(1) @binding(4)
var<storage, read> textures: Textures;


#import "texture_utils.wgsl"

@vertex
fn vs_main(
    v_in: DrawVertex,
    i_in: DrawInstance,
) -> VertexOutput {
    let instance_matrix = matrices.data[i_in.matrix_index];
    let p = &positions_and_colors.data[v_in.position_and_color_offset];
    let position = (*p).xyz;
    let color = (*p).w;

    let mvp = constant_data.proj * constant_data.view * instance_matrix;
    var vertex_out: VertexOutput;
    vertex_out.clip_position = mvp * vec4<f32>(position, 1.0);

    let instance_id = i_in.index;
    let mesh_id = i_in.mesh_index;
    let mesh = &meshes.data[mesh_id];
    var i = (*mesh).meshlet_offset + (*mesh).meshlet_count - 1u;
    var meshlet_id = f32(i + 1u);
    while(i > 0u) {
        if ((v_in.index - (*mesh).vertex_offset) > meshlets.data[i].vertex_offset) {
            break;
        }
        meshlet_id = f32(i - 1u);
        i -= 1u;
    }
    let material_id = (*mesh).material_index;

    vertex_out.params = vec4<f32>(f32(instance_id), f32(mesh_id), f32(meshlet_id), f32(material_id));

    let material = &materials.data[material_id];
    let texture_id = (*material).textures_indices[TEXTURE_TYPE_BASE_COLOR];
    var uv = vec2<f32>(0., 0.);
    if ((*material).textures_coord_set[TEXTURE_TYPE_BASE_COLOR] == 0u) {
        uv = uvs.data[v_in.uvs_offset.x].xy;
    } else if ((*material).textures_coord_set[TEXTURE_TYPE_BASE_COLOR] == 1u) {
        uv = uvs.data[v_in.uvs_offset.y].xy;
    } else if ((*material).textures_coord_set[TEXTURE_TYPE_BASE_COLOR] == 2u) {
        uv = uvs.data[v_in.uvs_offset.z].xy;
    } else if ((*material).textures_coord_set[TEXTURE_TYPE_BASE_COLOR] == 3u) {
        uv = uvs.data[v_in.uvs_offset.w].xy;
    }
    let unused = 0.;
    vertex_out.tex_coords =  vec4<f32>(uv.xy, f32(texture_id), color);

    let normal = pack_normal(normals.data[v_in.normal_offset].xyz);
    vertex_out.normals = vec4<f32>(normal.xy, position.z, 0.);

    return vertex_out;
}

@fragment
fn fs_main(
    v_in: VertexOutput,
) -> FragmentOutput {    
    var fragment_out: FragmentOutput;

    let display_meshlets = constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS;
    if (display_meshlets != 0u) {
        let meshlet_index = hash(u32(v_in.params.b));
        fragment_out.albedo = vec4<f32>(vec3<f32>(
            f32(meshlet_index & 255u), 
            f32((meshlet_index >> 8u) & 255u), 
            f32((meshlet_index >> 16u) & 255u)) / 255., 
            1.
        );
    } else {
        let texture_color = sample_texture(v_in.tex_coords.xyz);
        let vertex_color = unpack_unorm_to_4_f32(u32(v_in.tex_coords.w));
        let final_color = vec4<f32>(vertex_color.rgb * texture_color.rgb, vertex_color.a);
        fragment_out.albedo = final_color;
    }
    fragment_out.normals = v_in.normals;
    fragment_out.material_params = v_in.params;
    
    return fragment_out;
}