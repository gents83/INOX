#import "common.inc"
#import "utils.inc"

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) @interpolate(flat) meshlet_id: u32,
    @location(1) world_pos: vec3<f32>,
    @location(2) albedo: vec4<f32>,
    @location(3) normal: vec3<f32>,
    @location(4) uv_0: vec2<f32>,
    @location(5) uv_1: vec2<f32>,
    @location(6) uv_2: vec2<f32>,
    @location(7) uv_3: vec2<f32>,
};

struct FragmentOutput {
    @location(0) gbuffer_1: vec4<f32>,  //albedo        
    @location(1) gbuffer_2: vec4<f32>,  //normal       
    @location(2) gbuffer_3: vec4<f32>,  //meshlet_id   
    @location(3) gbuffer_4: vec4<f32>,  //uvs      
};


@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> vertices_attributes: VerticesAttributes;
@group(0) @binding(2)
var<storage, read> meshes: Meshes;


@vertex
fn vs_main(
    @builtin(vertex_index) vertex_id: u32,
    @builtin(instance_index) meshlet_id: u32,
    v_in: RuntimeVertexData,
) -> VertexOutput {
    let mvp = constant_data.proj * constant_data.view;
    var vertex_out: VertexOutput;

    vertex_out.world_pos = v_in.world_pos;
    vertex_out.clip_position = mvp * vec4<f32>(v_in.world_pos, 1.);
    vertex_out.meshlet_id = meshlet_id;
    vertex_out.albedo = vec4<f32>(1.);
    vertex_out.normal = vec3<f32>(1.);
    vertex_out.uv_0 = vec2<f32>(0.);
    vertex_out.uv_1 = vec2<f32>(0.);
    vertex_out.uv_2 = vec2<f32>(0.);
    vertex_out.uv_3 = vec2<f32>(0.);

    let vertex_index = vertex_id - meshes.data[v_in.mesh_index].vertices_position_offset;
    let vertex_layout = meshes.data[v_in.mesh_index].vertices_attribute_layout;
    let vertex_attribute_stride = vertex_layout_stride(vertex_layout);
    let attributes_offset = meshes.data[v_in.mesh_index].vertices_attribute_offset + vertex_index * vertex_attribute_stride;
    
    let offset_color = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_COLOR);
    let offset_normal = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_NORMAL);
    let offset_uv0 = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_UV1);
    let offset_uv1 = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_UV2);
    let offset_uv2 = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_UV3);
    let offset_uv3 = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_UV4);

    if(offset_color >= 0) {
        vertex_out.albedo = unpack_unorm_to_4_f32(vertices_attributes.data[attributes_offset + u32(offset_color)]);
    }
    if(offset_normal >= 0) {
        vertex_out.normal = decode_as_vec3(vertices_attributes.data[attributes_offset + u32(offset_normal)]);
    }
    if(offset_uv0 >= 0) {
        vertex_out.uv_0 = unpack2x16float(vertices_attributes.data[attributes_offset + u32(offset_uv0)]);
    }
    if(offset_uv1 >= 0) {
        vertex_out.uv_1 = unpack2x16float(vertices_attributes.data[attributes_offset + u32(offset_uv1)]);
    }
    if(offset_uv2 >= 0) {
        vertex_out.uv_2 = unpack2x16float(vertices_attributes.data[attributes_offset + u32(offset_uv2)]);
    }
    if(offset_uv3 >= 0) {
        vertex_out.uv_3 = unpack2x16float(vertices_attributes.data[attributes_offset + u32(offset_uv3)]);
    }     
    return vertex_out;
}

@fragment
fn fs_main(
    v_in: VertexOutput,
) -> FragmentOutput {    
    var fragment_out: FragmentOutput;

    fragment_out.gbuffer_1 = v_in.albedo;
    fragment_out.gbuffer_2 = unpack4x8unorm(pack2x16float(pack_normal(v_in.normal.xyz)));
    fragment_out.gbuffer_3 = unpack4x8unorm(v_in.meshlet_id + 1u);
    fragment_out.gbuffer_4 = vec4<f32>(v_in.uv_0, v_in.uv_1);
    
    return fragment_out;
}