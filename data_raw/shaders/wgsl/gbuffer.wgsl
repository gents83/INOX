#import "utils.wgsl"
#import "common.wgsl"

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) @interpolate(flat) mesh_and_meshlet_ids: vec2<u32>,
    @location(1) world_pos_color: vec4<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) uv_set: vec4<u32>,
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
var<storage, read> uvs: UVs;

@group(1) @binding(0)
var<storage, read> meshes: Meshes;
@group(1) @binding(1)
var<storage, read> materials: Materials;
@group(1) @binding(2)
var<storage, read> textures: Textures;
@group(1) @binding(3)
var<storage, read> meshlets: Meshlets;

#import "texture_utils.wgsl"
#import "material_utils.wgsl"


@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) meshlet_id: u32,
    v_in: DrawVertex,
) -> VertexOutput {
    let mvp = constant_data.proj * constant_data.view;

    let mesh_id = u32(meshlets.data[meshlet_id].mesh_index);
    let mesh = &meshes.data[mesh_id];

    let p = &positions_and_colors.data[v_in.position_and_color_offset];
    let world_position = (*mesh).transform * vec4<f32>((*p).xyz, 1.0);
    let color = (*p).w;
    
    var vertex_out: VertexOutput;
    vertex_out.clip_position = mvp * world_position;
    vertex_out.mesh_and_meshlet_ids = vec2<u32>(mesh_id, meshlet_id);
    vertex_out.world_pos_color = vec4<f32>(world_position.xyz, f32(color));

    vertex_out.normal = normals.data[v_in.normal_offset].xyz; 
    vertex_out.uv_set =  vec4<u32>(uvs.data[v_in.uvs_offset.x], uvs.data[v_in.uvs_offset.y], uvs.data[v_in.uvs_offset.z], uvs.data[v_in.uvs_offset.w]);

    return vertex_out;
}

@fragment
fn fs_main(
    v_in: VertexOutput,
) -> FragmentOutput {    
    var fragment_out: FragmentOutput;

    fragment_out.gbuffer_1 = v_in.world_pos_color;

    let mesh_id = u32(v_in.mesh_and_meshlet_ids.x);
    let mesh = &meshes.data[mesh_id];
    let material_id = u32((*mesh).material_index);
    // Retrieve the tangent space transform
    var n = normalize(v_in.normal.xyz); 
    if (has_texture(material_id, TEXTURE_TYPE_NORMAL)) {    
        let uv = compute_uvs(material_id, TEXTURE_TYPE_NORMAL, v_in.uv_set);    
        // get edge vectors of the pixel triangle 
        let dp1 = dpdx( v_in.world_pos_color.xyz ); 
        let dp2 = dpdy( v_in.world_pos_color.xyz ); 
        let duv1 = dpdx( uv.xy ); 
        let duv2 = dpdy( uv.xy );   
        // solve the linear system 
        let dp2perp = cross( dp2, n ); 
        let dp1perp = cross( n, dp1 ); 
        let tangent = dp2perp * duv1.x + dp1perp * duv2.x; 
        let bitangent = dp2perp * duv1.y + dp1perp * duv2.y;
        let t = normalize(tangent);
        let b = normalize(bitangent); 
        let tbn = mat3x3<f32>(t, b, n);
        let normal = sample_texture(uv);
        n = tbn * (2.0 * normal.rgb - vec3<f32>(1.0));
        n = normalize(n);
    }
    let packed_normal = pack_normal(n);
    fragment_out.gbuffer_2 = vec4<f32>(packed_normal.x, packed_normal.y, f32(mesh_id), f32(v_in.mesh_and_meshlet_ids.y));
    fragment_out.gbuffer_3 = vec4<f32>(f32(v_in.uv_set.x), f32(v_in.uv_set.y), f32(v_in.uv_set.z), f32(v_in.uv_set.w));
    
    return fragment_out;
}