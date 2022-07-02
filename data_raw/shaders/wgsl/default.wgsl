#import "utils.wgsl"
#import "common.wgsl"

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};


@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> positions_and_colors: PositionsAndColors;
@group(0) @binding(2)
var<storage, read> matrices: Matrices;
@group(0) @binding(3)
var<storage, read> meshes: Meshes;
@group(0) @binding(4)
var<storage, read> meshlets: Meshlets;

@vertex
fn vs_main(
    v_in: DrawVertex,
    i_in: DrawInstance,
) -> VertexOutput {
    let instance_matrix = matrices.data[i_in.matrix_index];
    let position = positions_and_colors.data[v_in.position_and_color_offset].xyz;

    let mvp = constant_data.proj * constant_data.view * instance_matrix;
    var vertex_out: VertexOutput;
    vertex_out.clip_position = mvp * vec4<f32>(position, 1.0);

    let r = f32(i_in.index);
    let g = f32(i_in.mesh_index);
    var b = 0.;
    for (var i = meshes.data[i_in.mesh_index].meshlet_offset; i < meshes.data[i_in.mesh_index].meshlet_offset + meshes.data[i_in.mesh_index].meshlet_count; i++) {
        if (v_in.index >= meshlets.data[i].indices_offset && v_in.index <= meshlets.data[i].indices_offset + meshlets.data[i].indices_count) {
            b = f32(i);
        }
    }
    let a = positions_and_colors.data[v_in.position_and_color_offset].w;
    vertex_out.color = vec4<f32>(r, g, b, a);


    return vertex_out;
}

@fragment
fn fs_main(
    v_in: VertexOutput,
) -> @location(0) vec4<f32> {
    let display_meshlets = constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS;
    if (display_meshlets != 0u) {
        let h = hash(u32(v_in.color.b));
        return vec4<f32>(vec3<f32>(f32(h & 255u), f32((h >> 8u) & 255u), f32((h >> 16u) & 255u)) / 255., 1.);
    }
    return v_in.color;
}