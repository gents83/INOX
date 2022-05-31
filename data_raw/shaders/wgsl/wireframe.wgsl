struct ConstantData {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    screen_width: f32,
    screen_height: f32,
    flags: u32,
};

struct VertexInput {
    //@builtin(vertex_index) index: u32,
    @location(0) position: vec3<f32>,
    @location(1) color: u32,
};

struct InstanceInput {
    //@builtin(instance_index) index: u32,
    @location(2) draw_area: vec4<f32>,
    @location(3) model_matrix_0: vec4<f32>,
    @location(4) model_matrix_1: vec4<f32>,
    @location(5) model_matrix_2: vec4<f32>,
    @location(6) model_matrix_3: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};


@group(0) @binding(0)
var<uniform> constant_data: ConstantData;

fn rgba_from_integer(color: u32) -> vec4<f32> {
    return vec4<f32>(
        f32(color & 255u),
        f32((color >> 8u) & 255u),
        f32((color >> 16u) & 255u),
        f32((color >> 24u) & 255u),
    );
}

@vertex
fn vs_main(
    v: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let instance_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    var vertex_out: VertexOutput;
    vertex_out.clip_position = constant_data.proj * constant_data.view * instance_matrix * vec4<f32>(v.position, 1.0);

    let color = rgba_from_integer(v.color);
    vertex_out.color = color;

    return vertex_out;
}

@fragment
fn fs_main(v_in: VertexOutput) -> @location(0) vec4<f32> {
    return v_in.color;
}