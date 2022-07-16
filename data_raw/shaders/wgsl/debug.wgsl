#import "utils.wgsl"
#import "common.wgsl"

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec3<f32>,
};

struct FragmentOutput {
    @location(0) color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> meshes: Meshes;
@group(0) @binding(2)
var<storage, read> meshlets: Meshlets;
@group(0) @binding(3)
var<storage, read> matrices: Matrices;

#import "shape_utils.wgsl"

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    //only one triangle, exceeding the viewport size
	let uv = vec2<f32>(f32((in_vertex_index << 1u) & 2u), f32(in_vertex_index & 2u));
	let pos = vec4<f32>(uv * vec2<f32>(2., -2.) + vec2<f32>(-1., 1.), 0., 1.);

    var vertex_out: VertexOutput;
    vertex_out.clip_position = pos;
    vertex_out.uv = vec3<f32>(uv.xy, f32(in_vertex_index));
    return vertex_out;
}


@fragment
fn fs_main(v_in: VertexOutput) -> @location(0) vec4<f32> {    
    let resolution = vec2<f32>(constant_data.screen_width, constant_data.screen_height);
    let aspect_ratio = resolution.x / resolution.y;
	var uv = (v_in.uv.xy * 2. - 1.);
    uv.x *= aspect_ratio;

    var color = vec4<f32>(1., 1., 1., 1.);
    var t = 0.;
    let radius = 0.2;
    let line_width = 0.01;

    let mvp = constant_data.proj * constant_data.view;

    let num_meshes = arrayLength(&meshes.data);
    for(var mesh_index = 0u; mesh_index < num_meshes; mesh_index++) {
        let mesh = &meshes.data[mesh_index];
        let m = &matrices.data[(*mesh).matrix_index];
        for(var meshlet_index = (*mesh).meshlet_offset; meshlet_index < (*mesh).meshlet_offset + (*mesh).meshlet_count; meshlet_index++) {
            let meshlet = &meshlets.data[meshlet_index];
            let clip_position = mvp * (*m) * vec4<f32>((*meshlet).center_radius.xyz, 1.);
            var c = clip_position.xy / resolution.xy;
            c.x *= aspect_ratio;
            t = max(t, draw_circle(uv, c, radius, line_width));
        }
    }
    color = color * t;

    return color;
}