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
var<storage, read> vertices_attributes: VerticesAttributes;
@group(0) @binding(4)
var<storage, read> meshes: Meshes;
@group(0) @binding(5)
var<storage, read> meshlets: Meshlets;

@group(1) @binding(0)
var<storage, read> materials: Materials;
@group(1) @binding(1)
var<storage, read> textures: Textures;
@group(1) @binding(2)
var<storage, read> lights: Lights;
@group(1) @binding(3)
var visibility_buffer_texture: texture_2d<f32>;

#import "matrix_utils.inc"
#import "geom_utils.inc"
#import "visibility_utils.inc"
#import "texture_utils.inc"
#import "material_utils.inc"
#import "pbr_utils.inc"

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
    var color = vec4<f32>(0.);
    if v_in.uv.x < 0. || v_in.uv.x > 1. || v_in.uv.y < 0. || v_in.uv.y > 1. {
        discard;
    }
    let d = vec2<f32>(textureDimensions(visibility_buffer_texture));
    let pixel_coords = vec2<i32>(v_in.uv * d);
    
    let visibility_output = textureLoad(visibility_buffer_texture, pixel_coords.xy, 0);
    let visibility_id = pack4x8unorm(visibility_output);
    if (visibility_id == 0u || (visibility_id & 0xFFFFFFFFu) == 0xFF000000u) {
        return color;
    }
    
    let display_meshlets = constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS;
    if (display_meshlets != 0u) 
    {
        let meshlet_id = (visibility_id >> 8u); 
        let meshlet_color = hash(meshlet_id + 1u);
        color = vec4<f32>(vec3<f32>(
            f32(meshlet_color & 255u),
            f32((meshlet_color >> 8u) & 255u),
            f32((meshlet_color >> 16u) & 255u)
        ) / 255., 1.);
    }
    else 
    {
        var pixel_data = visibility_to_gbuffer(visibility_id, v_in.uv.xy);
        let material_id = pixel_data.material_id;
        let material = &materials.data[material_id];
        pixel_data.color *= (*material).base_color;
        if (has_texture(material_id, TEXTURE_TYPE_BASE_COLOR)) {  
            let uv = material_texture_uv(&pixel_data, TEXTURE_TYPE_BASE_COLOR);
            let texture_color = sample_texture(uv);
            pixel_data.color *= texture_color * (*material).diffuse_color;
        }
        let alpha = material_alpha(material_id, pixel_data.color.a);
        if (alpha < 0.) {
            discard;
        }
        let pbr_data = compute_color(material_id, &pixel_data);
        color = compute_brdf(pbr_data);
    }
    return color;
}