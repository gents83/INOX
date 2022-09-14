#import "utils.inc"
#import "common.inc"

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) @interpolate(flat) mesh_and_meshlet_ids: vec2<u32>,
    @location(1) world_pos: vec4<f32>,
    @location(2) color: vec4<f32>,
    @location(3) normal: vec3<f32>,
    @location(4) uv_0: vec2<f32>,
    @location(5) uv_1: vec2<f32>,
    @location(6) uv_2: vec2<f32>,
    @location(7) uv_3: vec2<f32>,
};

struct FragmentOutput {
    @location(0) gbuffer_1: vec4<f32>,  //color        
    @location(1) gbuffer_2: vec4<f32>,  //normal       
    @location(2) gbuffer_3: vec4<f32>,  //meshlet_id   
    @location(3) gbuffer_4: vec4<f32>,  //uv_0         
    @location(4) gbuffer_5: vec4<f32>,  //uv_1         
    @location(5) gbuffer_6: vec4<f32>,  //uv_2         
    @location(6) gbuffer_7: vec4<f32>,  //uv_3         
};


@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> positions: Positions;
@group(0) @binding(2)
var<storage, read> colors: Colors;
@group(0) @binding(3)
var<storage, read> normals: Normals;
@group(0) @binding(4)
var<storage, read> uvs: UVs;

@group(1) @binding(0)
var<storage, read> meshes: Meshes;
@group(1) @binding(1)
var<storage, read> materials: Materials;
@group(1) @binding(2)
var<storage, read> textures: Textures;
@group(1) @binding(3)
var<storage, read> meshlets: Meshlets;
@group(1) @binding(4)
var<storage, read> meshes_aabb: AABBs;

#import "texture_utils.inc"
#import "material_utils.inc"


@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) meshlet_id: u32,
    v_in: Vertex,
) -> VertexOutput {
    let mvp = constant_data.proj * constant_data.view;

    let mesh_id = u32(meshlets.data[meshlet_id].mesh_index);
    let mesh = &meshes.data[mesh_id];
    let aabb = &meshes_aabb.data[mesh_id];

    let aabb_size = abs((*aabb).max - (*aabb).min);
    
    let p = (*aabb).min + decode_as_vec3(positions.data[v_in.position_and_color_offset]) * aabb_size;
    let world_position = vec4<f32>(transform_vector(p, (*mesh).position, (*mesh).orientation, (*mesh).scale), 1.0);
    let color = unpack_unorm_to_4_f32(colors.data[v_in.position_and_color_offset]) / 255.;
    
    var vertex_out: VertexOutput;
    vertex_out.clip_position = mvp * world_position;
    vertex_out.mesh_and_meshlet_ids = vec2<u32>(mesh_id, meshlet_id);
    vertex_out.world_pos = world_position;
    vertex_out.color = color;
    vertex_out.normal = decode_as_vec3(normals.data[v_in.normal_offset]); 
    vertex_out.uv_0 = unpack2x16float(uvs.data[v_in.uvs_offset.x]);
    vertex_out.uv_1 = unpack2x16float(uvs.data[v_in.uvs_offset.y]);
    vertex_out.uv_2 = unpack2x16float(uvs.data[v_in.uvs_offset.z]);
    vertex_out.uv_3 = unpack2x16float(uvs.data[v_in.uvs_offset.w]);

    return vertex_out;
}

@fragment
fn fs_main(
    v_in: VertexOutput,
) -> FragmentOutput {    
    var fragment_out: FragmentOutput;

    let mesh_id = u32(v_in.mesh_and_meshlet_ids.x);
    let mesh = &meshes.data[mesh_id];
    let material_id = u32((*mesh).material_index);
    let uv_set = vec4<u32>(
        pack2x16float(v_in.uv_0),
        pack2x16float(v_in.uv_1),
        pack2x16float(v_in.uv_2),
        pack2x16float(v_in.uv_3)
    );

    fragment_out.gbuffer_1 = v_in.color;
    fragment_out.gbuffer_2 = unpack4x8unorm(pack2x16float(pack_normal(v_in.normal.xyz)));
    fragment_out.gbuffer_3 = unpack4x8unorm(v_in.mesh_and_meshlet_ids.y + 1u);
    fragment_out.gbuffer_4 = unpack4x8unorm(pack2x16float(v_in.uv_0));
    fragment_out.gbuffer_5 = unpack4x8unorm(pack2x16float(v_in.uv_1));
    fragment_out.gbuffer_6 = unpack4x8unorm(pack2x16float(v_in.uv_2));
    fragment_out.gbuffer_7 = unpack4x8unorm(pack2x16float(v_in.uv_3));
    
    return fragment_out;
}