#import "common.inc"
#import "utils.inc"

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct FragmentOutput {
    @builtin(frag_depth) depth_buffer: f32,     
    @location(0) gbuffer_1: vec4<f32>,  //albedo        
    @location(1) gbuffer_2: vec4<f32>,  //normal       
    @location(2) gbuffer_3: vec4<f32>,  //meshlet_id   
    @location(3) gbuffer_4: vec4<f32>,  //uvs      
};


@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> indices: Indices;
@group(0) @binding(2)
var<storage, read> runtime_vertices: RuntimeVertices;
@group(0) @binding(3)
var<storage, read> vertices_attributes: VerticesAttributes;
@group(0) @binding(4)
var<storage, read> meshes: Meshes;
@group(0) @binding(5)
var<storage, read> meshlets: Meshlets;
@group(0) @binding(6)
var visibility_buffer_texture: texture_2d<f32>;

#import "matrix_utils.inc"
#import "geom_utils.inc"


@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    //only one triangle, exceeding the viewport size
    let uv = vec2<f32>(f32((in_vertex_index << 1u) & 2u), f32(in_vertex_index & 2u));
    let pos = vec4<f32>(uv * vec2<f32>(2., -2.) + vec2<f32>(-1., 1.), 0., 1.);

    var vertex_out: VertexOutput;
    vertex_out.clip_position = pos;
    vertex_out.uv = uv;
    return vertex_out;
}

@fragment
fn fs_main(
    v_in: VertexOutput,
) -> FragmentOutput {    
    var fragment_out: FragmentOutput;
    if v_in.uv.x < 0. || v_in.uv.x > 1. || v_in.uv.y < 0. || v_in.uv.y > 1. {
        return fragment_out;
    }
    let d = vec2<f32>(textureDimensions(visibility_buffer_texture));
    let pixel_coords = vec2<i32>(i32(v_in.uv.x * d.x), i32(v_in.uv.y * d.y));
    let visibility_output = textureLoad(visibility_buffer_texture, pixel_coords.xy, 0);
    let visibility_id = pack4x8unorm(visibility_output);
    if (visibility_id == 0u || (visibility_id & 0xFFFFFFFFu) == 0xFF000000u) {
        return fragment_out;
    }
    let meshlet_id = (visibility_id >> 8u) - 1u; 
    let primitive_id = visibility_id & 255u;
    fragment_out.gbuffer_3 = unpack4x8unorm(meshlet_id + 1u);

    let meshlet = &meshlets.data[meshlet_id];
    let index_offset = (*meshlet).indices_offset + (primitive_id * 3u);

    let mesh = &meshes.data[(*meshlet).mesh_index];
    let position_offset = (*mesh).vertices_position_offset;
    let attributes_offset = (*mesh).vertices_attribute_offset;
    let vertex_layout = (*mesh).vertices_attribute_layout;
    let vertex_attribute_stride = vertex_layout_stride(vertex_layout);   
    let offset_color = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_COLOR);
    let offset_normal = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_NORMAL);
    let offset_uv0 = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_UV1);
    let offset_uv1 = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_UV2);
    let offset_uv2 = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_UV3);
    let offset_uv3 = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_UV4); 

    let vert_indices = vec3<u32>(indices.data[index_offset], indices.data[index_offset + 1u], indices.data[index_offset + 2u]);
    let pos_indices = vert_indices + vec3<u32>(position_offset, position_offset, position_offset);
    let attr_indices = vec3<u32>(attributes_offset + vert_indices.x * vertex_attribute_stride, 
                                 attributes_offset + vert_indices.y * vertex_attribute_stride,
                                 attributes_offset + vert_indices.z * vertex_attribute_stride);
    
    let v1 = runtime_vertices.data[pos_indices.x].world_pos;
    let v2 = runtime_vertices.data[pos_indices.y].world_pos;
    let v3 = runtime_vertices.data[pos_indices.z].world_pos;
    
    let mvp = constant_data.proj * constant_data.view;
    var p1 = mvp * vec4<f32>(v1, 1.);
    var p2 = mvp * vec4<f32>(v2, 1.);
    var p3 = mvp * vec4<f32>(v3, 1.);

    // Calculate the inverse of w, since it's going to be used several times
    let one_over_w = 1. / vec3<f32>(p1.w, p2.w, p3.w);
    p1 = (p1 * one_over_w.x + 1.) * 0.5;
    p2 = (p2 * one_over_w.y + 1.) * 0.5;
    p3 = (p3 * one_over_w.z + 1.) * 0.5;
    
    // Get delta vector that describes current screen point relative to vertex 0
    var screen_pixel = v_in.uv.xy;
    screen_pixel.y = 1. - screen_pixel.y;
    let delta = screen_pixel + -p1.xy;
    let barycentrics = compute_barycentrics(p1.xy, p2.xy, p3.xy, screen_pixel);
    let deriv = compute_partial_derivatives(p1.xy, p2.xy, p3.xy);

    if (offset_color >= 0) {
        let a1 = vertices_attributes.data[attr_indices.x + u32(offset_color)];
        let a2 = vertices_attributes.data[attr_indices.y + u32(offset_color)];
        let a3 = vertices_attributes.data[attr_indices.z + u32(offset_color)];
        let c1 = unpack_unorm_to_4_f32(a1);
        let c2 = unpack_unorm_to_4_f32(a2);
        let c3 = unpack_unorm_to_4_f32(a3);
        let vertex_color = barycentrics.x * c1 + barycentrics.y * c2 + barycentrics.z * c3;    
        fragment_out.gbuffer_1 = vertex_color;
    } else {
        fragment_out.gbuffer_1 = vec4<f32>(1.);
    }
    if (offset_normal >= 0) {
        let a1 = vertices_attributes.data[attr_indices.x + u32(offset_normal)];
        let a2 = vertices_attributes.data[attr_indices.y + u32(offset_normal)];
        let a3 = vertices_attributes.data[attr_indices.z + u32(offset_normal)];
        let n1 = decode_as_vec3(a1);
        let n2 = decode_as_vec3(a2);
        let n3 = decode_as_vec3(a3);
        let normal = barycentrics.x * n1 + barycentrics.y * n2 + barycentrics.z * n3;    
        fragment_out.gbuffer_2 = unpack4x8unorm(pack2x16float(pack_normal(normal.xyz)));
    }
    var uvs = vec4<f32>(0.);
    if(offset_uv0 >= 0) {
        let a1 = vertices_attributes.data[attr_indices.x + u32(offset_uv0)];
        let a2 = vertices_attributes.data[attr_indices.y + u32(offset_uv0)];
        let a3 = vertices_attributes.data[attr_indices.z + u32(offset_uv0)];
        let uv1 = unpack2x16float(a1);
        let uv2 = unpack2x16float(a2);
        let uv3 = unpack2x16float(a3);
        var uv = interpolate_2d_attribute(uv1, uv2, uv3, deriv, delta);
        uvs.x = uv.x;
        uvs.y = uv.y;
    }
    if(offset_uv1 >= 0) {
        let a1 = vertices_attributes.data[attr_indices.x + u32(offset_uv1)];
        let a2 = vertices_attributes.data[attr_indices.y + u32(offset_uv1)];
        let a3 = vertices_attributes.data[attr_indices.z + u32(offset_uv1)];
        let uv1 = unpack2x16float(a1);
        let uv2 = unpack2x16float(a2);
        let uv3 = unpack2x16float(a3);
        var uv = interpolate_2d_attribute(uv1, uv2, uv3, deriv, delta);
        uvs.z = uv.x;
        uvs.w = uv.y;
    }
    fragment_out.gbuffer_4 = uvs;

    let z = barycentrics.x * p1.z + barycentrics.y * p2.z + barycentrics.z * p3.z;  
    fragment_out.depth_buffer = z;
    
    return fragment_out;
}