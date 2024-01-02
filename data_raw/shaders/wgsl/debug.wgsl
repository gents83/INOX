#import "common.inc"
#import "utils.inc"

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct FragmentOutput {
    @location(0) color: vec4<f32>,
};


@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> indices: Indices;
@group(0) @binding(2)
var<storage, read> runtime_vertices: RuntimeVertices;
@group(0) @binding(3)
var<storage, read> meshes: Meshes;
@group(0) @binding(4)
var<storage, read> meshlets: Meshlets;

@group(1) @binding(0)
var finalize_texture: texture_2d<f32>;
@group(1) @binding(1)
var visibility_texture: texture_2d<f32>;
@group(1) @binding(2)
var radiance_texture: texture_2d<f32>;
@group(1) @binding(3)
var depth_texture: texture_depth_2d;
@group(1) @binding(4)
var debug_data_texture: texture_2d<f32>;

#import "geom_utils.inc"
#import "shape_utils.inc"

fn read_value_from_debug_data_texture(i: ptr<function, u32>) -> f32 {
    let dimensions = textureDimensions(debug_data_texture);
    var index = *i;
    let v = textureLoad(debug_data_texture, vec2<u32>(index % dimensions.x, index / dimensions.x), 0);
    *i = index + 1u;
    return v.r;
}

fn draw_triangle_from_visibility(visibility_id: u32, pixel: vec2<u32>, dimensions: vec2<u32>) -> vec3<f32>{
    let meshlet_id = (visibility_id >> 8u) - 1u;
    let primitive_id = visibility_id & 255u;
    let meshlet = &meshlets.data[meshlet_id];
    let index_offset = (*meshlet).indices_offset + (primitive_id * 3u);

    let mesh_id = u32((*meshlet).mesh_index);
    let mesh = &meshes.data[mesh_id];
    let position_offset = (*mesh).vertices_position_offset;
    
    let vert_indices = vec3<u32>(indices.data[index_offset], indices.data[index_offset + 1u], indices.data[index_offset + 2u]);
    
    let p1 = runtime_vertices.data[vert_indices.x + position_offset].world_pos;
    let p2 = runtime_vertices.data[vert_indices.y + position_offset].world_pos;
    let p3 = runtime_vertices.data[vert_indices.z + position_offset].world_pos;
    
    let line_color = vec3<f32>(0., 1., 1.);
    let line_size = 0.001;
    var color = vec3<f32>(0.);
    color += draw_line_3d(pixel, dimensions, p1, p2, line_color, line_size);
    color += draw_line_3d(pixel, dimensions, p2, p3, line_color, line_size);
    color += draw_line_3d(pixel, dimensions, p3, p1, line_color, line_size);
    return color;
}

fn debug_color_override(color: vec4<f32>, pixel: vec2<u32>) -> vec4<f32> {
    var out_color = color;
    if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS) != 0) {
        let visibility_output = textureLoad(visibility_texture, pixel, 0);
        let visibility_id = pack4x8unorm(visibility_output);
        if (visibility_id != 0u && (visibility_id & 0xFFFFFFFFu) != 0xFF000000u) {
            let meshlet_id = (visibility_id >> 8u); 
            let meshlet_color = hash(meshlet_id + 1u);
            out_color = vec4<f32>(vec3<f32>(
                f32(meshlet_color & 255u),
                f32((meshlet_color >> 8u) & 255u),
                f32((meshlet_color >> 16u) & 255u)
            ) / 255., 1.);
        }
    } 
    else if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_VISIBILITY_BUFFER) != 0) {
        out_color = textureLoad(visibility_texture, pixel, 0);
    } 
    else if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_DEPTH_BUFFER) != 0) {
        let depth = textureLoad(depth_texture, pixel, 0);
        let v = vec3<f32>(1. - depth);
        out_color = vec4<f32>(v, 1.);
    } 
    else if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_PATHTRACE) != 0) {
        let dimensions = textureDimensions(debug_data_texture);
        var debug_index = 0u;
        let max_index = u32(read_value_from_debug_data_texture(&debug_index));
        var start = vec3<f32>(0.);
        let line_color = vec3<f32>(0., 1., 0.);
        let line_size = 0.001;
        var bounce_index = 0u;
        var color = out_color.rgb;
        while(debug_index < max_index) {
            let visibility_id = u32(read_value_from_debug_data_texture(&debug_index));
            color += draw_triangle_from_visibility(visibility_id, pixel, dimensions);
            
            var origin = vec3<f32>(0.);
            origin.x = read_value_from_debug_data_texture(&debug_index);
            origin.y = read_value_from_debug_data_texture(&debug_index);
            origin.z = read_value_from_debug_data_texture(&debug_index);
            var direction = vec3<f32>(0.);
            direction.x = read_value_from_debug_data_texture(&debug_index);
            direction.y = read_value_from_debug_data_texture(&debug_index);
            direction.z = read_value_from_debug_data_texture(&debug_index);
            if (bounce_index > 0u) {
                color += draw_line_3d(pixel, dimensions, start, origin, line_color, line_size);
            }
            bounce_index += 1u;
            start = origin;
        }
        out_color = vec4<f32>(color, 1.);
    } 
    return out_color;
}



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
fn fs_main(v_in: VertexOutput) -> @location(0) vec4<f32> {
    let target_dimensions = vec2<f32>(textureDimensions(debug_data_texture));
    let source_dimensions = vec2<f32>(textureDimensions(finalize_texture));
    let scale = vec2<f32>(source_dimensions) / vec2<f32>(target_dimensions);
    let target_pixel = vec2<u32>(u32(v_in.uv.x * target_dimensions.x), u32(v_in.uv.y * target_dimensions.y));
    let source_pixel = vec2<u32>(vec2<f32>(target_pixel) * scale);

    var out_color = textureLoad(finalize_texture, source_pixel, 0);    
    out_color = debug_color_override(out_color, target_pixel); 
    return out_color;
}