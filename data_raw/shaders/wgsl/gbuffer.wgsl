#import "utils.wgsl"
#import "common.wgsl"

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) mesh_and_meshlet_ids: vec2<u32>,
    @location(1) world_pos_color: vec4<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec4<f32>,
    @location(4) uv_0_1: vec4<f32>,
    @location(5) uv_2_3: vec4<f32>,
};

struct FragmentOutput {
    @location(0) gbuffer_1: vec4<f32>, //world_pos.x, world_pos.y, world_pos.z, color
    @location(1) gbuffer_2: vec4<f32>, //normal.xy, mesh_id, meshlet_id  
    @location(2) gbuffer_3: vec4<f32>, //uv_0, uv_1, uv_2, uv_3
};


@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> positions_and_colors: PositionsAndColors;
@group(0) @binding(2)
var<storage, read> normals: NormalsAndPadding;
@group(0) @binding(3)
var<storage, read> tangents: Tangents;
@group(0) @binding(4)
var<storage, read> uvs: UVs;

@group(1) @binding(0)
var<storage, read> matrices: Matrices;
@group(1) @binding(1)
var<storage, read> meshes: Meshes;
@group(1) @binding(2)
var<storage, read> materials: Materials;
@group(1) @binding(3)
var<storage, read> textures: Textures;
@group(1) @binding(4)
var<storage, read> meshlets: Meshlets;

#import "texture_utils.wgsl"
#import "material_utils.wgsl"


@vertex
fn vs_main(
    v_in: DrawVertex,
    i_in: DrawInstance,
) -> VertexOutput {
    let instance_matrix = matrices.data[i_in.matrix_index];
    let p = &positions_and_colors.data[v_in.position_and_color_offset];
    let world_position = instance_matrix * vec4<f32>((*p).xyz, 1.0);
    let color = (*p).w;

    let mvp = constant_data.proj * constant_data.view;

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
    
    var vertex_out: VertexOutput;
    vertex_out.clip_position = mvp * world_position;
    vertex_out.mesh_and_meshlet_ids = vec2<u32>(u32(mesh_id), u32(meshlet_id));
    vertex_out.world_pos_color = vec4<f32>(world_position.xyz, f32(color));

    vertex_out.normal = normals.data[v_in.normal_offset].xyz; 
    vertex_out.tangent = tangents.data[v_in.tangent_offset].xyzw;
    vertex_out.uv_0_1 =  vec4<f32>(uvs.data[v_in.uvs_offset.x].xy, uvs.data[v_in.uvs_offset.y].xy);
    vertex_out.uv_2_3 =  vec4<f32>(uvs.data[v_in.uvs_offset.z].xy, uvs.data[v_in.uvs_offset.w].xy);
    
    return vertex_out;
}

fn sample_material_texture(material_index: u32, texture_type: u32, uv_0_1: vec4<f32>, uv_2_3: vec4<f32>) -> vec4<f32> {
    let material = &materials.data[material_index];    
    let texture_coords_set = (*material).textures_coord_set[texture_type];
    let texture_index = (*material).textures_indices[texture_type];
    var uv = uv_0_1.xy;
    if (texture_coords_set == 1u) {
        uv = uv_0_1.zw;
    } else if (texture_coords_set == 2u) {
        uv = uv_2_3.xy;
    } else if (texture_coords_set == 3u) {
        uv = uv_2_3.zw;
    }
    return sample_texture(vec3<f32>(uv, f32(texture_index)));
}

@fragment
fn fs_main(
    v_in: VertexOutput,
) -> FragmentOutput {    
    var fragment_out: FragmentOutput;

    fragment_out.gbuffer_1 = v_in.world_pos_color;

    let mesh_id = v_in.mesh_and_meshlet_ids.x;
    let mesh = &meshes.data[mesh_id];
    let material_id = u32((*mesh).material_index);
    // Retrieve the tangent space matrix
    var n = normalize(v_in.normal.xyz); 
    if (has_texture(material_id, TEXTURE_TYPE_NORMAL)) {
        var t = normalize(v_in.tangent.xyz - n * dot(v_in.tangent.xyz, n));
        var b = cross(n, t) * v_in.tangent.w;
        let tbn = mat3x3<f32>(t, b, n);
        let normal = sample_material_texture(material_id, TEXTURE_TYPE_NORMAL, v_in.uv_0_1, v_in.uv_2_3);
        n = tbn * (2.0 * normal.rgb - vec3<f32>(1.0));
        n = normalize(n);
    }
    let packed_normal = pack_normal(n);
    fragment_out.gbuffer_2 = vec4<f32>(packed_normal.x, packed_normal.y, f32(mesh_id), f32(v_in.mesh_and_meshlet_ids.y));
    
    //let uv0 = pack2x16float(v_in.uv_0_1.xy);
    //let uv1 = pack2x16float(v_in.uv_0_1.zw);
    //let uv2 = pack2x16float(v_in.uv_2_3.xy);
    //let uv3 = pack2x16float(v_in.uv_2_3.zw);
    //fragment_out.gbuffer_3 = vec4<f32>(f32(uv0), f32(uv1), f32(uv2), f32(uv3));
    fragment_out.gbuffer_3 = vec4<f32>(v_in.uv_0_1.xy, v_in.uv_0_1.zw);
    
    return fragment_out;
}