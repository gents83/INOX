#import "utils.wgsl"
#import "common.wgsl"


struct PbrData {
    width: u32,
    height: u32,
    visibility_buffer_index: u32,
    _padding: u32,
};


@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<uniform> pbr_data: PbrData;
@group(0) @binding(2)
var<storage, read> indices: Indices;
@group(0) @binding(3)
var<storage, read> vertices: Vertices;
@group(0) @binding(4)
var<storage, read> positions_and_colors: PositionsAndColors;

@group(1) @binding(0)
var<storage, read> meshes: Meshes;
@group(1) @binding(1)
var<storage, read> meshlets: Meshlets;
@group(1) @binding(2)
var<storage, read> materials: Materials;
@group(1) @binding(3)
var<storage, read> textures: Textures;
@group(1) @binding(4)
var<storage, read> lights: Lights;

@group(3) @binding(0)
var render_target: texture_storage_2d_array<rgba8unorm, read_write>;



#import "texture_utils.wgsl"
#import "material_utils.wgsl"
#import "pbr_utils.wgsl"

struct Derivatives {
    dx: vec3<f32>,
    dy: vec3<f32>,
}

struct GradientInterpolation
{
	interp: vec2<f32>,
	dx: vec2<f32>,
	dy: vec2<f32>,
};

fn compute_barycentrics_2d(a: vec2<f32>, b: vec2<f32>, c: vec2<f32>, p: vec2<f32>) -> vec3<f32> {
    let v0 = b - a;
    let v1 = c - a;
    let v2 = p - a;
    
    let d00 = dot(v0, v0);    
    let d01 = dot(v0, v1);    
    let d11 = dot(v1, v1);
    let d20 = dot(v2, v0);
    let d21 = dot(v2, v1);
    
    let inv_denom = 1. / (d00 * d11 - d01 * d01);    
    let v = (d11 * d20 - d01 * d21) * inv_denom;    
    let w = (d00 * d21 - d01 * d20) * inv_denom;    
    let u = 1. - v - w;

    return vec3 (u,v,w);
}

fn compute_barycentrics(a: vec3<f32>, b: vec3<f32>, c: vec3<f32>, p: vec3<f32>) -> vec3<f32> {
    let v0 = b - a;
    let v1 = c - a;
    let v2 = p - a;
    
    let d00 = dot(v0, v0);    
    let d01 = dot(v0, v1);    
    let d11 = dot(v1, v1);
    let d20 = dot(v2, v0);
    let d21 = dot(v2, v1);
    
    let inv_denom = 1. / (d00 * d11 - d01 * d01);    
    let v = (d11 * d20 - d01 * d21) * inv_denom;    
    let w = (d00 * d21 - d01 * d20) * inv_denom;    
    let u = 1. - v - w;

    return vec3 (u,v,w);
}

fn compute_partial_derivatives(v0: vec2<f32>, v1: vec2<f32>, v2: vec2<f32>) -> Derivatives
{
    let d = 1. / determinant(mat2x2<f32>(v2-v1, v0-v1));
    
    var deriv: Derivatives;
    deriv.dx = vec3<f32>(v1.y - v2.y, v2.y - v0.y, v0.y - v1.y) * d;
    deriv.dy = vec3<f32>(v2.x - v1.x, v0.x - v2.x, v1.x - v0.x) * d;
    return deriv;
}

fn interpolate_attribute(attributes: vec3<f32>, db_dx: vec3<f32>, db_dy: vec3<f32>, d: vec2<f32>) -> f32
{
	let attribute_x = dot(attributes, db_dx);
	let attribute_y = dot(attributes, db_dy);
	let attribute_s = attributes.x;
	return (attribute_s + d.x * attribute_x + d.y * attribute_y);
}

// Interpolate 2D attributes using the partial derivatives and generates dx and dy for texture sampling.
fn interpolate_attribute_with_gradient(v0: vec2<f32>, v1: vec2<f32>, v2: vec2<f32>, 
    db_dx: vec3<f32>, db_dy: vec3<f32>, d: vec2<f32>, scale_over_resolution: vec2<f32>) -> GradientInterpolation
{
    let attr0 = vec3<f32>(v0.x, v1.x, v2.x);
    let attr1 = vec3<f32>(v0.y, v1.y, v2.y);
	let attribute_x = vec2<f32>(dot(db_dx, attr0), dot(db_dx, attr1));
	let attribute_y = vec2<f32>(dot(db_dy, attr0), dot(db_dy, attr1));
	let attribute_s = v0.xy;

    var r: GradientInterpolation;
	r.dx = attribute_x * scale_over_resolution.x;
	r.dy = attribute_y * scale_over_resolution.y;
	r.interp = (attribute_s + d.x * attribute_x + d.y * attribute_y);
	return r;
}


@compute
@workgroup_size(4, 4, 1)
fn main(
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, 
    @builtin(local_invocation_index) local_invocation_index: u32, 
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    let pixel = vec2<i32>(i32(global_invocation_id.x), i32(global_invocation_id.y));
    if (pixel.x >= i32(pbr_data.width) || pixel.y >= i32(pbr_data.height))
    {
        return;
    }
    
    var color = vec4<f32>(0., 0., 0., 0.);
    let visibility_output = load(pbr_data.visibility_buffer_index, pixel);
    let visibility_id = pack4x8unorm(visibility_output);
    if ((visibility_id & 0xFFFFFFFFu) == 0xFF000000u) {
        textureStore(render_target, pixel.xy, 0, color);
        return;
    }

    var meshlet_id = (visibility_id >> 8u) - 1u; 
    let primitive_id = visibility_id & 255u;

    let meshlet = &meshlets.data[meshlet_id];
    let mesh_id = (*meshlet).mesh_index;

    let display_meshlets = constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS;
    if (display_meshlets != 0u) {
        let c = hash(meshlet_id);
        color = vec4<f32>(vec3<f32>(
            f32(c & 255u), 
            f32((c >> 8u) & 255u), 
            f32((c >> 16u) & 255u)) / 255., 
            1.
        );
    } else {
        let mesh = &meshes.data[mesh_id];
        let material_id = u32((*mesh).material_index);

        let mvp = constant_data.proj * constant_data.view;

        let screen_pixel = vec2<f32>(f32(pixel.x), f32(pixel.y));
        let scale_over_resolution = vec2<f32>(1. / constant_data.screen_width, 1. / constant_data.screen_height);

        let index_offset = (*mesh).indices_offset + (*meshlet).indices_offset + 3u * primitive_id;
        let i1 = indices.data[index_offset];
        let i2 = indices.data[index_offset + 1u];
        let i3 = indices.data[index_offset + 2u];

        let vertex_offset = (*mesh).vertex_offset + (*meshlet).vertex_offset;
        let v1 = &vertices.data[vertex_offset + i1];
        let v2 = &vertices.data[vertex_offset + i2];
        let v3 = &vertices.data[vertex_offset + i3];

        let p1 = mvp * (*mesh).transform * vec4<f32>(positions_and_colors.data[(*v1).position_and_color_offset].xyz, 1.);
        let p2 = mvp * (*mesh).transform * vec4<f32>(positions_and_colors.data[(*v2).position_and_color_offset].xyz, 1.);
        let p3 = mvp * (*mesh).transform * vec4<f32>(positions_and_colors.data[(*v3).position_and_color_offset].xyz, 1.);

        let c1 = unpack_unorm_to_4_f32(u32(positions_and_colors.data[(*v1).position_and_color_offset].w));
        let c2 = unpack_unorm_to_4_f32(u32(positions_and_colors.data[(*v2).position_and_color_offset].w));
        let c3 = unpack_unorm_to_4_f32(u32(positions_and_colors.data[(*v3).position_and_color_offset].w));

        let b = compute_barycentrics_2d(p1.xy, p2.xy, p3.xy, screen_pixel.xy);
        let vertex_color = b.x * c1 + b.y * c2 + b.z * c3;
        color = vertex_color;
    }

    textureStore(render_target, pixel.xy, 0, color);
}