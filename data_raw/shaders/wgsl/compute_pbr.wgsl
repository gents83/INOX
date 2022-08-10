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
@group(0) @binding(5)
var<storage, read> normals_and_padding: NormalsAndPadding;
@group(0) @binding(6)
var<storage, read> uvs: UVs;

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
#import "geom_utils.wgsl"
#import "pbr_utils.wgsl"


@compute
@workgroup_size(8, 4, 1)
fn main(
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, 
    @builtin(local_invocation_index) local_invocation_index: u32, 
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    for (var i = 0u; i < 8u; i++) {     
        for (var j = 0u; j < 8u; j++) {            
            let pixel = vec3<i32>(i32(global_invocation_id.x * 8u + i), i32(global_invocation_id.y * 8u + j), i32(pbr_data.visibility_buffer_index));
            if (pixel.x >= i32(pbr_data.width) || pixel.y >= i32(pbr_data.height))
            {
                continue;
            }
            
            var color = vec4<f32>(0., 0., 0., 0.);
            let visibility_output = load_texture(pixel);
            let visibility_id = pack4x8unorm(visibility_output);
            if ((visibility_id & 0xFFFFFFFFu) == 0xFF000000u) {
                textureStore(render_target, pixel.xy, 0, color);
                continue;
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

                var screen_pixel = vec2<f32>(f32(pixel.x), f32(pixel.y));
                screen_pixel = screen_pixel / vec2<f32>(f32(pbr_data.width), f32(pbr_data.height));
                screen_pixel.y = 1. - screen_pixel.y;
                
                let index_offset = (*mesh).indices_offset + (*meshlet).indices_offset + 3u * primitive_id;
                let i1 = indices.data[index_offset];
                let i2 = indices.data[index_offset + 1u];
                let i3 = indices.data[index_offset + 2u];

                let vertex_offset = (*mesh).vertex_offset;
                let v1 = &vertices.data[vertex_offset + i1];
                let v2 = &vertices.data[vertex_offset + i2];
                let v3 = &vertices.data[vertex_offset + i3];

                var p1 = mvp * (*mesh).transform * vec4<f32>(positions_and_colors.data[(*v1).position_and_color_offset].xyz, 1.);
                var p2 = mvp * (*mesh).transform * vec4<f32>(positions_and_colors.data[(*v2).position_and_color_offset].xyz, 1.);
                var p3 = mvp * (*mesh).transform * vec4<f32>(positions_and_colors.data[(*v3).position_and_color_offset].xyz, 1.);

                // Calculate the inverse of w, since it's going to be used several times
                let one_over_w = 1. / vec3<f32>(p1.w, p2.w, p3.w);
                p1 = (p1 * one_over_w.x + 1.) * 0.5;
                p2 = (p2 * one_over_w.y + 1.) * 0.5;
                p3 = (p3 * one_over_w.z + 1.) * 0.5;

                // Get delta vector that describes current screen point relative to vertex 0
                let delta = screen_pixel + -p1.xy;
                let barycentrics = compute_barycentrics(p1.xy, p2.xy, p3.xy, screen_pixel.xy);
                let deriv = compute_partial_derivatives(p1.xy, p2.xy, p3.xy);

                let c1 = unpack_unorm_to_4_f32(u32(positions_and_colors.data[(*v1).position_and_color_offset].w));
                let c2 = unpack_unorm_to_4_f32(u32(positions_and_colors.data[(*v2).position_and_color_offset].w));
                let c3 = unpack_unorm_to_4_f32(u32(positions_and_colors.data[(*v3).position_and_color_offset].w));

                let vertex_color = barycentrics.x * c1 + barycentrics.y * c2 + barycentrics.z * c3;        
                let alpha = compute_alpha(material_id, vertex_color.a);
                if alpha < 0. {
                    textureStore(render_target, pixel.xy, 0, color);
                    continue;
                }        

                let uv0_1 = uvs.data[(*v1).uvs_offset[0]].xy;
                let uv0_2 = uvs.data[(*v2).uvs_offset[0]].xy;
                let uv0_3 = uvs.data[(*v3).uvs_offset[0]].xy;
                
                let uv1_1 = uvs.data[(*v1).uvs_offset[1]].xy;
                let uv1_2 = uvs.data[(*v2).uvs_offset[1]].xy;
                let uv1_3 = uvs.data[(*v3).uvs_offset[1]].xy;

                var uv_0 = interpolate_2d_attribute(uv0_1, uv0_2, uv0_3, deriv, delta);
                var uv_1 = interpolate_2d_attribute(uv1_1, uv1_2, uv1_3, deriv, delta);
                let uv_0_1 = vec4<f32>(uv_0.xy, uv_1.xy);

                let texture_color = sample_material_texture(uv_0_1, material_id, TEXTURE_TYPE_BASE_COLOR);
                color = vec4<f32>(vertex_color.rgb * texture_color.rgb, vertex_color.a);

                let n1 = normals_and_padding.data[(*v1).normal_offset].xyz;
                let n2 = normals_and_padding.data[(*v2).normal_offset].xyz;
                let n3 = normals_and_padding.data[(*v3).normal_offset].xyz;

                //let world_pos = barycentrics.x * p1 + barycentrics.y * p2 + barycentrics.z * p3;
                //let n = barycentrics.x * n1 + barycentrics.y * n2 + barycentrics.z * n3;
                let world_pos = interpolate_3d_attribute(p1.xyz, p2.xyz, p3.xyz, deriv, delta);
                let n = interpolate_3d_attribute(n1, n2, n3, deriv, delta);

                color = pbr(world_pos.xyz, n, material_id, color, uv_0_1);
            }

            textureStore(render_target, pixel.xy, 0, color);
        }   
    }
}