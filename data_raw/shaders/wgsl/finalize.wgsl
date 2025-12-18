#import "common.inc"
#import "utils.inc"
#import "color_utils.inc"

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
var direct_texture: texture_2d<f32>;
@group(0) @binding(2)
var indirect_diffuse_texture: texture_2d<f32>;
@group(0) @binding(3)
var indirect_specular_texture: texture_2d<f32>;
@group(0) @binding(4)
var previous_texture: texture_2d<f32>;
@group(0) @binding(5)
var shadow_texture: texture_2d<f32>;
@group(0) @binding(6)
var ao_texture: texture_2d<f32>;

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

const EXPOSURE: f32 = 0.5;

@fragment
fn fs_main(v_in: VertexOutput) -> @location(0) vec4<f32> {
    let dimensions = textureDimensions(direct_texture);
    let coords = vec2<i32>(i32(v_in.uv.x * f32(dimensions.x)), i32(v_in.uv.y * f32(dimensions.y)));

    let direct = textureLoad(direct_texture, coords, 0).rgb;
    let indirect_diffuse = textureLoad(indirect_diffuse_texture, coords, 0).rgb;
    let indirect_specular = textureLoad(indirect_specular_texture, coords, 0).rgb;
    let shadow = textureLoad(shadow_texture, coords, 0).r;
    let ao = textureLoad(ao_texture, coords, 0).r;
    
    // Combine with shadow and AO
    // Direct lighting affected by both shadow and AO
    // Indirect diffuse affected by AO
    // Indirect specular less affected by AO (view-dependent)
    var radiance = direct * shadow * ao + indirect_diffuse * ao + indirect_specular;
    
    // Temporal Accumulation
    if (constant_data.frame_index > 0u) {
        let prev_value = textureLoad(previous_texture, coords, 0).rgb;
        let frame_index = f32(min(constant_data.frame_index + 1u, 1000u));
        let weight = 1.0 / frame_index; // Converge to average
        // Or fixed weight for moving objects?
        // Using accumulation weight 1/N is good for static image convergence.
        radiance = mix(prev_value, radiance, weight);
    }
    
    var out_color = vec4<f32>(radiance, 1.0);
    
    // Tone Mapping
    out_color = vec4<f32>(tonemap_ACES_Hill(out_color.rgb), 1.0);
    
    // Gamma (sRGB conversion handling? linearTosRGB handles gamma usually approx pow(1/2.2))
    out_color = vec4<f32>(linearTosRGB(out_color.rgb), 1.0);

    return out_color;
}
