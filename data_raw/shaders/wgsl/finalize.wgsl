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
var indirect_diffuse_texture: texture_storage_2d<rgba32uint, read>;
@group(0) @binding(3)
var indirect_specular_texture: texture_storage_2d<rgba32uint, read>;
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

    let direct = textureLoad(direct_texture, coords, 0);
    let indirect_diffuse_data = textureLoad(indirect_diffuse_texture, coords);
    let indirect_specular_data = textureLoad(indirect_specular_texture, coords);
    
    // Normalize by global frame count (plus 1 to avoid div by zero)
    // Since we clear the indirect textures every frame now (to fix ghosting), 
    // we should NOT divide by accumulated frame count. The buffer contains only THe CURRENT frame's energy.
    let sample_count = f32(constant_data.frame_index + 1u);
    
    // Decoding: The buffer contains raw accumulated radiance for THIS frame.
    let indirect_diffuse = decode_uvec3_to_vec3(indirect_diffuse_data.rgb);
    let indirect_specular = decode_uvec3_to_vec3(indirect_specular_data.rgb);
    
    let shadow = textureLoad(shadow_texture, coords, 0).r;
    let ao = textureLoad(ao_texture, coords, 0).r;
    
    // Debug: Visualize Frame Index
    // If frame_index is increasing, this should slowly turn white
    // If it stays black, frame_index is stuck at 0
    // let debug_val = f32(constant_data.frame_index) / 255.0; // Wrap every 255 frames
    // var radiance = vec3(debug_val);
    
    // Normal Output
    // Normal Output
    
    // Nearest Neighbor Upsampling for Half-Res GI
    // The texture size is Width/2.
    // Screen Coord "coords" (0..W) maps to Texture Coord (0..W/2) via division.
    let indirect_coord = vec2<i32>(coords.x / 2, coords.y / 2);
    
    let indirect_diffuse_packed = textureLoad(indirect_diffuse_texture, indirect_coord).rgb;
    let indirect_specular_packed = textureLoad(indirect_specular_texture, indirect_coord).rgb;
    
    let ind_diff = decode_uvec3_to_vec3(indirect_diffuse_packed);
    let ind_spec = decode_uvec3_to_vec3(indirect_specular_packed);

    // Amplified for visibility test (20x)
    var radiance = direct.rgb * shadow * ao + (ind_diff + ind_spec) * 20.0;
    
    var out_color = vec4<f32>(radiance, direct.a);
    
    // Tone Mapping
    out_color = vec4<f32>(tonemap_ACES_Hill(out_color.rgb), out_color.a);
    
    // Gamma
    out_color = vec4<f32>(linearTosRGB(out_color.rgb), out_color.a);

    return out_color;
}
