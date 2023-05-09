#import "common.inc"
#import "utils.inc"

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) @interpolate(flat) meshlet_ids: u32,
    @location(1) world_pos: vec3<f32>,
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
    @location(3) gbuffer_4: vec4<f32>,  //uvs      
};


@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> runtime_vertices: RuntimeVertices;
@group(0) @binding(2)
var<storage, read> vertices_attributes: VerticesAttributes;
@group(0) @binding(3)
var<storage, read> meshes: Meshes;


@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) meshlet_id: u32,
    v_in: RuntimeVertexData,
) -> VertexOutput {
    let mvp = constant_data.proj * constant_data.view;
    var vertex_out: VertexOutput;

    vertex_out.world_pos = v_in.world_pos;
    vertex_out.clip_position = mvp * vec4<f32>(v_in.world_pos, 1.);
    vertex_out.meshlet_ids = meshlet_id + 1u;

    let vertex_layout = meshes.data[v_in.mesh_index].vertices_attribute_layout;
    let vertex_attribute_stride = vertex_layout_stride(vertex_layout);
    let attributes_offset = meshes.data[v_in.mesh_index].vertices_attribute_offset + vertex_index * vertex_attribute_stride;

    if(has_vertex_attribute(vertex_layout, VERTEX_ATTRIBUTE_HAS_COLOR)) {
        let offset = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_COLOR);
        vertex_out.color = unpack_unorm_to_4_f32(vertices_attributes.data[attributes_offset + offset]);
    } else {
        vertex_out.color = vec4<f32>(0.);
    }
    if(has_vertex_attribute(vertex_layout, VERTEX_ATTRIBUTE_HAS_NORMAL)) {
        let offset = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_NORMAL);
        vertex_out.normal = decode_as_vec3(vertices_attributes.data[attributes_offset + offset]);
    } else {
        vertex_out.normal = vec3<f32>(0.);
    }
    if(has_vertex_attribute(vertex_layout, VERTEX_ATTRIBUTE_HAS_UV1)) {
        let offset = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_UV1);
        vertex_out.uv_0 = unpack2x16float(vertices_attributes.data[attributes_offset + offset]);
    } else {
        vertex_out.uv_0 = vec2<f32>(0.);
    }
    if(has_vertex_attribute(vertex_layout, VERTEX_ATTRIBUTE_HAS_UV2)) {
        let offset = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_UV2);
        vertex_out.uv_1 = unpack2x16float(vertices_attributes.data[attributes_offset + offset]);
    } else {
        vertex_out.uv_1 = vec2<f32>(0.);
    }
    if(has_vertex_attribute(vertex_layout, VERTEX_ATTRIBUTE_HAS_UV3)) {
        let offset = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_UV3);
        vertex_out.uv_2 = unpack2x16float(vertices_attributes.data[attributes_offset + offset]);
    } else {
        vertex_out.uv_2 = vec2<f32>(0.);
    }
    if(has_vertex_attribute(vertex_layout, VERTEX_ATTRIBUTE_HAS_UV4)) {
        let offset = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_UV4);
        vertex_out.uv_3 = unpack2x16float(vertices_attributes.data[attributes_offset + offset]);
    } else {
        vertex_out.uv_3 = vec2<f32>(0.);
    }
    
    return vertex_out;
}

@fragment
fn fs_main(
    v_in: VertexOutput,
) -> FragmentOutput {    
    var fragment_out: FragmentOutput;
    let uv_set = vec4<f32>(
        f32(pack2x16float(v_in.uv_0)),
        f32(pack2x16float(v_in.uv_1)),
        f32(pack2x16float(v_in.uv_2)),
        f32(pack2x16float(v_in.uv_3))
    );

    fragment_out.gbuffer_1 = v_in.color;
    fragment_out.gbuffer_2 = unpack4x8unorm(pack2x16float(pack_normal(v_in.normal.xyz)));
    fragment_out.gbuffer_3 = unpack4x8unorm(v_in.meshlet_ids);
    fragment_out.gbuffer_4 = uv_set;
    
    return fragment_out;
}