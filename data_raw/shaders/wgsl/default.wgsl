#import "utils.wgsl"
#import "common.wgsl"

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) params: vec4<f32>,
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

    let instance_id = i_in.index;
    let mesh_id = i_in.mesh_index;
    var i = meshes.data[mesh_id].meshlet_offset + meshes.data[mesh_id].meshlet_count - 1u;
    var meshlet_id = f32(i + 1u);
    while(i > 0u) {
        if ((v_in.index - meshes.data[mesh_id].vertex_offset) > meshlets.data[i].vertex_offset) {
            break;
        }
        meshlet_id = f32(i - 1u);
        i -= 1u;
    }
    let color = positions_and_colors.data[v_in.position_and_color_offset].w;
    vertex_out.params = vec4<f32>(f32(instance_id), f32(mesh_id), f32(meshlet_id), color);


    return vertex_out;
}

@fragment
fn fs_main(
    v_in: VertexOutput,
) -> @location(0) vec4<f32> {
    let display_meshlets = constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS;
    if (display_meshlets != 0u) {
        let h = hash(u32(v_in.params.b));
        return vec4<f32>(vec3<f32>(f32(h & 255u), f32((h >> 8u) & 255u), f32((h >> 16u) & 255u)) / 255., 1.);
    }
    return v_in.params;
}