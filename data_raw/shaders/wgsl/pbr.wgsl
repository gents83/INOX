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
var<storage, read> meshes: Meshes;
@group(0) @binding(2)
var<storage, read> meshlets: Meshlets;
@group(0) @binding(3)
var<storage, read> materials: Materials;
@group(0) @binding(4)
var<storage, read> textures: Textures;
@group(0) @binding(5)
var<storage, read> lights: Lights;

@group(1) @binding(0)
var gbuffer_1_texture: texture_2d<f32>;
@group(1) @binding(1)
var gbuffer_2_texture: texture_2d<f32>;
@group(1) @binding(2)
var gbuffer_3_texture: texture_2d<f32>;
@group(1) @binding(3)
var gbuffer_4_texture: texture_2d<f32>;
@group(1) @binding(4)
var depth_texture: texture_depth_2d;

#import "texture_utils.inc"
#import "material_utils.inc"
#import "matrix_utils.inc"
#import "pbr_utils.inc"




fn sample_gbuffer(i: u32, pixel_coords: vec2<i32>) -> vec4<f32> {
    var v = vec4<f32>(0.);
    switch (i) {
        case 0u: { 
            v = textureLoad(gbuffer_1_texture, pixel_coords, 0); 
        }
        case 1u: { 
            v = textureLoad(gbuffer_2_texture, pixel_coords, 0); 
        }
        case 2u: { 
            v = textureLoad(gbuffer_3_texture, pixel_coords, 0); 
        }
        case 3u: { 
            v = textureLoad(gbuffer_4_texture, pixel_coords, 0); 
        }
        default: { 
            v = vec4<f32>(textureLoad(depth_texture, pixel_coords, 0)); 
        }
    }
    return v;
}

fn compute_world_position_from_depth(pixel_coords: vec2<i32>, uv: vec2<f32>) -> vec3<f32> {
    let z = sample_gbuffer(4u, pixel_coords).r;
    let clip_position = vec4<f32>(uv * 2. - 1., z * 2. - 1., 1.);
    let homogeneous = constant_data.inverse_view_proj * clip_position;
    return homogeneous.xyz / homogeneous.w;
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
    let d = vec2<f32>(textureDimensions(depth_texture));
    let pixel_coords = vec2<i32>(v_in.uv * d);
    
    let vertex_color = sample_gbuffer(0u, pixel_coords);
    let meshlet_id = pack4x8unorm(sample_gbuffer(2u, pixel_coords));
    let num_meshlets = arrayLength(&meshlets.data);
    if meshlet_id == 0u || meshlet_id > num_meshlets {
        return vec4<f32>(0., 0., 0., 0.);
    }

    var color = vec4<f32>(0., 0., 0., 0.);
    let display_meshlets = constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS;
    if (display_meshlets != 0u) {
        let meshlet_color = hash(meshlet_id);
        color = vec4<f32>(vec3<f32>(
            f32(meshlet_color & 255u),
            f32((meshlet_color >> 8u) & 255u),
            f32((meshlet_color >> 16u) & 255u)
        ) / 255., 1.);
    } else {
        let uv_set = sample_gbuffer(3u, pixel_coords);

        let mesh_id = meshlets.data[meshlet_id - 1u].mesh_index;
        let mesh = &meshes.data[mesh_id];
        let material_id = u32((*mesh).material_index);
        let texture_color = sample_material_texture(material_id, TEXTURE_TYPE_BASE_COLOR, uv_set);

        let alpha = compute_alpha(material_id, vertex_color.a);
        if alpha < 0. {
            discard;
        }

        color = vec4<f32>(vertex_color.rgb * texture_color.rgb, alpha);

        let packed_normal = unpack2x16float(pack4x8unorm(sample_gbuffer(1u, pixel_coords)));
        let n = unpack_normal(packed_normal);
        let world_pos = compute_world_position_from_depth(pixel_coords, v_in.uv);
        let normal = rotate_vector(n, (*mesh).orientation);
        //color = compute_brdf(world_pos, normal, material_id, color, uv_set);
    }

    return color;
}