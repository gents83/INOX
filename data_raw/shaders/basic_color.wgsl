// Vertex shader

struct ConstantData {
    view: mat4x4<f32>;
    proj: mat4x4<f32>;
    screen_width: f32;
    screen_height: f32;
    padding: vec2<f32>;
};
[[group(0), binding(0)]]
var<uniform> constant_data: ConstantData;

struct InstanceInput {
    //[[builtin(instance_index)]] index: u32;
    [[location(8)]] id: vec4<f32>;
    [[location(9)]] model_matrix_0: vec4<f32>;
    [[location(10)]] model_matrix_1: vec4<f32>;
    [[location(11)]] model_matrix_2: vec4<f32>;
    [[location(12)]] model_matrix_3: vec4<f32>;
    [[location(13)]] material_index: i32;
};

struct VertexInput {
    //[[builtin(vertex_index)]] index: u32;
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[location(2)]] tangent: vec3<f32>;
    [[location(3)]] color: vec4<f32>;
    [[location(4)]] tex_coords_0: vec2<f32>;
    [[location(5)]] tex_coords_1: vec2<f32>;
    [[location(6)]] tex_coords_2: vec2<f32>;
    [[location(7)]] tex_coords_3: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let instance_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = constant_data.proj * constant_data.view * instance_matrix * vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return in.color;
}