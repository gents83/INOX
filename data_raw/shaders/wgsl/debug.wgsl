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
var finalize_texture: texture_2d<f32>;
@group(1) @binding(4)
var visibility_texture: texture_2d<u32>;
@group(1) @binding(5)
var radiance_texture: texture_2d<f32>;
@group(1) @binding(6)
var depth_texture: texture_depth_2d;
@group(1) @binding(7)
var debug_data_texture: texture_2d<f32>;

#import "texture_utils.inc"
#import "geom_utils.inc"
#import "shape_utils.inc"
#import "matrix_utils.inc"
#import "material_utils.inc"
#import "pbr_utils.inc"
#import "visibility_utils.inc"
#import "color_utils.inc"


fn read_value_from_debug_data_texture(i: ptr<function, u32>) -> f32 {
    let dimensions = textureDimensions(debug_data_texture);
    var index = *i;
    let v = textureLoad(debug_data_texture, vec2<u32>(index % dimensions.x, index / dimensions.x), 0);
    *i = index + 1u;
    return v.r;
}

fn draw_triangle_from_visibility(visibility_id: u32, pixel: vec2<u32>, dimensions: vec2<u32>) -> vec3<f32>{
    let meshlet_id = (visibility_id >> 8u) - 1u;
    let primitive_id = visibility_id & 255u;
    let meshlet = &meshlets.data[meshlet_id];
    let index_offset = (*meshlet).indices_offset + (primitive_id * 3u);

    let mesh_id = u32((*meshlet).mesh_index);
    let mesh = &meshes.data[mesh_id];
    let position_offset = (*mesh).vertices_position_offset;
    
    let vert_indices = vec3<u32>(indices.data[index_offset], indices.data[index_offset + 1u], indices.data[index_offset + 2u]);
    
    let p1 = runtime_vertices.data[vert_indices.x + position_offset].world_pos;
    let p2 = runtime_vertices.data[vert_indices.y + position_offset].world_pos;
    let p3 = runtime_vertices.data[vert_indices.z + position_offset].world_pos;
    
    let line_color = vec3<f32>(0., 1., 1.);
    let line_size = 0.001;
    var color = vec3<f32>(0.);
    color += draw_line_3d(pixel, dimensions, p1, p2, line_color, line_size);
    color += draw_line_3d(pixel, dimensions, p2, p3, line_color, line_size);
    color += draw_line_3d(pixel, dimensions, p3, p1, line_color, line_size);
    return color;
}

fn draw_cube_from_min_max(min: vec3<f32>, max:vec3<f32>, pixel: vec2<u32>, dimensions: vec2<u32>) -> vec3<f32> {  
    let line_color = vec3<f32>(0., 0., 1.);
    let line_size = 0.003;
    var color = vec3<f32>(0.);
    color += draw_line_3d(pixel, dimensions, vec3<f32>(min.x, min.y, min.z), vec3<f32>(max.x, min.y, min.z), line_color, line_size);
    color += draw_line_3d(pixel, dimensions, vec3<f32>(max.x, min.y, min.z), vec3<f32>(max.x, min.y, max.z), line_color, line_size);
    color += draw_line_3d(pixel, dimensions, vec3<f32>(max.x, min.y, max.z), vec3<f32>(min.x, min.y, max.z), line_color, line_size);
    color += draw_line_3d(pixel, dimensions, vec3<f32>(min.x, min.y, max.z), vec3<f32>(min.x, min.y, min.z), line_color, line_size);
    //
    color += draw_line_3d(pixel, dimensions, vec3<f32>(min.x, max.y, min.z), vec3<f32>(max.x, max.y, min.z), line_color, line_size);
    color += draw_line_3d(pixel, dimensions, vec3<f32>(max.x, max.y, min.z), vec3<f32>(max.x, max.y, max.z), line_color, line_size);
    color += draw_line_3d(pixel, dimensions, vec3<f32>(max.x, max.y, max.z), vec3<f32>(min.x, max.y, max.z), line_color, line_size);
    color += draw_line_3d(pixel, dimensions, vec3<f32>(min.x, max.y, max.z), vec3<f32>(min.x, max.y, min.z), line_color, line_size);
    //
    color += draw_line_3d(pixel, dimensions, vec3<f32>(min.x, min.y, min.z), vec3<f32>(min.x, max.y, min.z), line_color, line_size);
    color += draw_line_3d(pixel, dimensions, vec3<f32>(min.x, min.y, max.z), vec3<f32>(min.x, max.y, max.z), line_color, line_size);
    color += draw_line_3d(pixel, dimensions, vec3<f32>(max.x, min.y, min.z), vec3<f32>(max.x, max.y, min.z), line_color, line_size);
    color += draw_line_3d(pixel, dimensions, vec3<f32>(max.x, min.y, max.z), vec3<f32>(max.x, max.y, max.z), line_color, line_size);
    return color;
}

fn debug_color_override(color: vec4<f32>, pixel: vec2<u32>, dimensions: vec2<u32>) -> vec4<f32> {
    var out_color = color;
    if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS) != 0) {
        let visibility_dimensions = textureDimensions(visibility_texture);
        let visibility_scale = vec2<f32>(visibility_dimensions) / vec2<f32>(dimensions);
        let visibility_pixel = vec2<u32>(vec2<f32>(pixel) * visibility_scale);
        let visibility_value = textureLoad(visibility_texture, visibility_pixel, 0);
        let visibility_id = visibility_value.r;
        if (visibility_id != 0u && (visibility_id & 0xFFFFFFFFu) != 0xFF000000u) {
            let meshlet_id = (visibility_id >> 8u); 
            let meshlet_color = hash(meshlet_id + 1u);
            out_color = vec4<f32>(vec3<f32>(
                f32(meshlet_color & 255u),
                f32((meshlet_color >> 8u) & 255u),
                f32((meshlet_color >> 16u) & 255u)
            ) / 255., 1.);
        }
    } 
    else if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_UV_0) != 0) {
        let depth_dimensions = textureDimensions(depth_texture);
        let depth_scale = vec2<f32>(depth_dimensions) / vec2<f32>(dimensions);
        let depth_pixel = vec2<u32>(vec2<f32>(pixel) * depth_scale);
        let depth = textureLoad(depth_texture, depth_pixel, 0);
        let hit_point = pixel_to_world(depth_pixel, depth_dimensions, depth); 

        let visibility_dimensions = textureDimensions(visibility_texture);
        let visibility_scale = vec2<f32>(visibility_dimensions) / vec2<f32>(dimensions);
        let visibility_pixel = vec2<u32>(vec2<f32>(pixel) * visibility_scale);
        let visibility_value = textureLoad(visibility_texture, visibility_pixel, 0);
        let visibility_id = visibility_value.r;
        if (visibility_id != 0u && (visibility_id & 0xFFFFFFFFu) != 0xFF000000u) {
            var pixel_data = visibility_to_gbuffer(visibility_id, hit_point);
            out_color = vec4<f32>(vec3<f32>(pixel_data.uv_set[0], 0.), 1.);
        }
    } 
    else if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_UV_1) != 0) {
        let depth_dimensions = textureDimensions(depth_texture);
        let depth_scale = vec2<f32>(depth_dimensions) / vec2<f32>(dimensions);
        let depth_pixel = vec2<u32>(vec2<f32>(pixel) * depth_scale);
        let depth = textureLoad(depth_texture, depth_pixel, 0);
        let hit_point = pixel_to_world(depth_pixel, depth_dimensions, depth); 

        let visibility_dimensions = textureDimensions(visibility_texture);
        let visibility_scale = vec2<f32>(visibility_dimensions) / vec2<f32>(dimensions);
        let visibility_pixel = vec2<u32>(vec2<f32>(pixel) * visibility_scale);
        let visibility_value = textureLoad(visibility_texture, visibility_pixel, 0);
        let visibility_id = visibility_value.r;
        if (visibility_id != 0u && (visibility_id & 0xFFFFFFFFu) != 0xFF000000u) {
            var pixel_data = visibility_to_gbuffer(visibility_id, hit_point);
            out_color = vec4<f32>(vec3<f32>(pixel_data.uv_set[1], 0.), 1.);
        }
    } 
    else if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_UV_1) != 0) {
        let depth_dimensions = textureDimensions(depth_texture);
        let depth_scale = vec2<f32>(depth_dimensions) / vec2<f32>(dimensions);
        let depth_pixel = vec2<u32>(vec2<f32>(pixel) * depth_scale);
        let depth = textureLoad(depth_texture, depth_pixel, 0);
        let hit_point = pixel_to_world(depth_pixel, depth_dimensions, depth); 

        let visibility_dimensions = textureDimensions(visibility_texture);
        let visibility_scale = vec2<f32>(visibility_dimensions) / vec2<f32>(dimensions);
        let visibility_pixel = vec2<u32>(vec2<f32>(pixel) * visibility_scale);
        let visibility_value = textureLoad(visibility_texture, visibility_pixel, 0);
        let visibility_id = visibility_value.r;
        if (visibility_id != 0u && (visibility_id & 0xFFFFFFFFu) != 0xFF000000u) {
            var pixel_data = visibility_to_gbuffer(visibility_id, hit_point);
            out_color = vec4<f32>(vec3<f32>(pixel_data.uv_set[2], 0.), 1.);
        }
    } 
    else if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_UV_3) != 0) {
        let depth_dimensions = textureDimensions(depth_texture);
        let depth_scale = vec2<f32>(depth_dimensions) / vec2<f32>(dimensions);
        let depth_pixel = vec2<u32>(vec2<f32>(pixel) * depth_scale);
        let depth = textureLoad(depth_texture, depth_pixel, 0);
        let hit_point = pixel_to_world(depth_pixel, depth_dimensions, depth); 

        let visibility_dimensions = textureDimensions(visibility_texture);
        let visibility_scale = vec2<f32>(visibility_dimensions) / vec2<f32>(dimensions);
        let visibility_pixel = vec2<u32>(vec2<f32>(pixel) * visibility_scale);
        let visibility_value = textureLoad(visibility_texture, visibility_pixel, 0);
        let visibility_id = visibility_value.r;
        if (visibility_id != 0u && (visibility_id & 0xFFFFFFFFu) != 0xFF000000u) {
            var pixel_data = visibility_to_gbuffer(visibility_id, hit_point);
            out_color = vec4<f32>(vec3<f32>(pixel_data.uv_set[3], 0.), 1.);
        }
    } 
    else if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_NORMALS) != 0) {
        let depth_dimensions = textureDimensions(depth_texture);
        let depth_scale = vec2<f32>(depth_dimensions) / vec2<f32>(dimensions);
        let depth_pixel = vec2<u32>(vec2<f32>(pixel) * depth_scale);
        let depth = textureLoad(depth_texture, depth_pixel, 0);
        let hit_point = pixel_to_world(depth_pixel, depth_dimensions, depth); 

        let visibility_dimensions = textureDimensions(visibility_texture);
        let visibility_scale = vec2<f32>(visibility_dimensions) / vec2<f32>(dimensions);
        let visibility_pixel = vec2<u32>(vec2<f32>(pixel) * visibility_scale);
        let visibility_value = textureLoad(visibility_texture, visibility_pixel, 0);
        let visibility_id = visibility_value.r;
        if (visibility_id != 0u && (visibility_id & 0xFFFFFFFFu) != 0xFF000000u) {
            var pixel_data = visibility_to_gbuffer(visibility_id, hit_point);
            var material = materials.data[pixel_data.material_id];
            let tbn = compute_tbn(&material, &pixel_data);
            out_color = vec4<f32>((vec3<f32>(1.) + tbn.normal) / vec3<f32>(2.), 1.);
        }
    } 
    else if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_TANGENT) != 0) {
        let depth_dimensions = textureDimensions(depth_texture);
        let depth_scale = vec2<f32>(depth_dimensions) / vec2<f32>(dimensions);
        let depth_pixel = vec2<u32>(vec2<f32>(pixel) * depth_scale);
        let depth = textureLoad(depth_texture, depth_pixel, 0);
        let hit_point = pixel_to_world(depth_pixel, depth_dimensions, depth); 

        let visibility_dimensions = textureDimensions(visibility_texture);
        let visibility_scale = vec2<f32>(visibility_dimensions) / vec2<f32>(dimensions);
        let visibility_pixel = vec2<u32>(vec2<f32>(pixel) * visibility_scale);
        let visibility_value = textureLoad(visibility_texture, visibility_pixel, 0);
        let visibility_id = visibility_value.r;
        if (visibility_id != 0u && (visibility_id & 0xFFFFFFFFu) != 0xFF000000u) {
            var pixel_data = visibility_to_gbuffer(visibility_id, hit_point);
            var material = materials.data[pixel_data.material_id];
            let tbn = compute_tbn(&material, &pixel_data);
            out_color = vec4<f32>((vec3<f32>(1.) + tbn.tangent) / vec3<f32>(2.), 1.);
        }
    } 
    else if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_BITANGENT) != 0) {
        let depth_dimensions = textureDimensions(depth_texture);
        let depth_scale = vec2<f32>(depth_dimensions) / vec2<f32>(dimensions);
        let depth_pixel = vec2<u32>(vec2<f32>(pixel) * depth_scale);
        let depth = textureLoad(depth_texture, depth_pixel, 0);
        let hit_point = pixel_to_world(depth_pixel, depth_dimensions, depth); 

        let visibility_dimensions = textureDimensions(visibility_texture);
        let visibility_scale = vec2<f32>(visibility_dimensions) / vec2<f32>(dimensions);
        let visibility_pixel = vec2<u32>(vec2<f32>(pixel) * visibility_scale);
        let visibility_value = textureLoad(visibility_texture, visibility_pixel, 0);
        let visibility_id = visibility_value.r;
        if (visibility_id != 0u && (visibility_id & 0xFFFFFFFFu) != 0xFF000000u) {
            var pixel_data = visibility_to_gbuffer(visibility_id, hit_point);
            var material = materials.data[pixel_data.material_id];
            let tbn = compute_tbn(&material, &pixel_data);
            out_color = vec4<f32>((vec3<f32>(1.) + tbn.binormal) / vec3<f32>(2.), 1.);
        }
    } 
    else if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_BASE_COLOR) != 0) {
        let depth_dimensions = textureDimensions(depth_texture);
        let depth_scale = vec2<f32>(depth_dimensions) / vec2<f32>(dimensions);
        let depth_pixel = vec2<u32>(vec2<f32>(pixel) * depth_scale);
        let depth = textureLoad(depth_texture, depth_pixel, 0);
        let hit_point = pixel_to_world(depth_pixel, depth_dimensions, depth); 

        let visibility_dimensions = textureDimensions(visibility_texture);
        let visibility_scale = vec2<f32>(visibility_dimensions) / vec2<f32>(dimensions);
        let visibility_pixel = vec2<u32>(vec2<f32>(pixel) * visibility_scale);
        let visibility_value = textureLoad(visibility_texture, visibility_pixel, 0);
        let visibility_id = visibility_value.r;
        if (visibility_id != 0u && (visibility_id & 0xFFFFFFFFu) != 0xFF000000u) {
            var pixel_data = visibility_to_gbuffer(visibility_id, hit_point);
            let material_info = compute_color_from_material(pixel_data.material_id, &pixel_data);
            out_color = vec4<f32>(vec3<f32>(material_info.base_color.rgb), 1.);
        }
    } 
    else if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_METALLIC) != 0) {
        let depth_dimensions = textureDimensions(depth_texture);
        let depth_scale = vec2<f32>(depth_dimensions) / vec2<f32>(dimensions);
        let depth_pixel = vec2<u32>(vec2<f32>(pixel) * depth_scale);
        let depth = textureLoad(depth_texture, depth_pixel, 0);
        let hit_point = pixel_to_world(depth_pixel, depth_dimensions, depth); 

        let visibility_dimensions = textureDimensions(visibility_texture);
        let visibility_scale = vec2<f32>(visibility_dimensions) / vec2<f32>(dimensions);
        let visibility_pixel = vec2<u32>(vec2<f32>(pixel) * visibility_scale);
        let visibility_value = textureLoad(visibility_texture, visibility_pixel, 0);
        let visibility_id = visibility_value.r;
        if (visibility_id != 0u && (visibility_id & 0xFFFFFFFFu) != 0xFF000000u) {
            var pixel_data = visibility_to_gbuffer(visibility_id, hit_point);
            let material_info = compute_color_from_material(pixel_data.material_id, &pixel_data);
            out_color = vec4<f32>(vec3<f32>(material_info.metallic), 1.);
        }
    } 
    else if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_ROUGHNESS) != 0) {
        let depth_dimensions = textureDimensions(depth_texture);
        let depth_scale = vec2<f32>(depth_dimensions) / vec2<f32>(dimensions);
        let depth_pixel = vec2<u32>(vec2<f32>(pixel) * depth_scale);
        let depth = textureLoad(depth_texture, depth_pixel, 0);
        let hit_point = pixel_to_world(depth_pixel, depth_dimensions, depth); 

        let visibility_dimensions = textureDimensions(visibility_texture);
        let visibility_scale = vec2<f32>(visibility_dimensions) / vec2<f32>(dimensions);
        let visibility_pixel = vec2<u32>(vec2<f32>(pixel) * visibility_scale);
        let visibility_value = textureLoad(visibility_texture, visibility_pixel, 0);
        let visibility_id = visibility_value.r;
        if (visibility_id != 0u && (visibility_id & 0xFFFFFFFFu) != 0xFF000000u) {
            var pixel_data = visibility_to_gbuffer(visibility_id, hit_point);
            let material_info = compute_color_from_material(pixel_data.material_id, &pixel_data);
            out_color = vec4<f32>(vec3<f32>(material_info.perceptual_roughness), 1.);
        }
    } 
    else if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_RADIANCE_BUFFER) != 0) {
        let radiance_dimensions = textureDimensions(radiance_texture);
        let radiance_scale = vec2<f32>(radiance_dimensions) / vec2<f32>(dimensions);
        let radiance_pixel = vec2<u32>(vec2<f32>(pixel) * radiance_scale);
        out_color = textureLoad(radiance_texture, radiance_pixel, 0);
    } 
    else if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_DEPTH_BUFFER) != 0) {
        let depth_dimensions = textureDimensions(depth_texture);
        let depth_scale = vec2<f32>(depth_dimensions) / vec2<f32>(dimensions);
        let depth_pixel = vec2<u32>(vec2<f32>(pixel) * depth_scale);
        let depth = textureLoad(depth_texture, depth_pixel, 0);
        let v = vec3<f32>(1. - depth) * 10.;
        out_color = vec4<f32>(v, 1.);
    } 
    else if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_PATHTRACE) != 0) {
        let debug_dimensions = textureDimensions(debug_data_texture);
        let debug_scale = vec2<f32>(debug_dimensions) / vec2<f32>(dimensions);
        let debug_pixel = vec2<u32>(vec2<f32>(pixel) * debug_scale);
        var origin = vec3<f32>(0.);
        var direction = vec3<f32>(0.);
        let line_color = vec3<f32>(0., 1., 0.);
        let line_size = 0.003;
        var bounce_index = 0u;
        var color = out_color.rgb;        
        /*
        var debug_bhv_index = 100u;
        let max_bhv_index = u32(read_value_from_debug_data_texture(&debug_bhv_index));
        while(debug_bhv_index < max_bhv_index) 
        {
            var min = vec3<f32>(0.);
            min.x = read_value_from_debug_data_texture(&debug_bhv_index);
            min.y = read_value_from_debug_data_texture(&debug_bhv_index);
            min.z = read_value_from_debug_data_texture(&debug_bhv_index);
            var max = vec3<f32>(0.);
            max.x = read_value_from_debug_data_texture(&debug_bhv_index);
            max.y = read_value_from_debug_data_texture(&debug_bhv_index);
            max.z = read_value_from_debug_data_texture(&debug_bhv_index);
            color += draw_cube_from_min_max(min, max, debug_pixel, debug_dimensions);
            //color += draw_line_3d(debug_pixel, debug_dimensions, min, max, vec3<f32>(0.,0.,1.), line_size);
        }
        */
        
        var debug_index = 0u;
        let max_index = u32(read_value_from_debug_data_texture(&debug_index));
        while(debug_index < max_index) {
            let visibility_id = u32(read_value_from_debug_data_texture(&debug_index));
            color += draw_triangle_from_visibility(visibility_id, debug_pixel, debug_dimensions);
            
            var previous = origin;
            origin.x = read_value_from_debug_data_texture(&debug_index);
            origin.y = read_value_from_debug_data_texture(&debug_index);
            origin.z = read_value_from_debug_data_texture(&debug_index);
            direction.x = read_value_from_debug_data_texture(&debug_index);
            direction.y = read_value_from_debug_data_texture(&debug_index);
            direction.z = read_value_from_debug_data_texture(&debug_index);
            if (bounce_index > 0u) {
                color += draw_line_3d(debug_pixel, debug_dimensions, previous, origin, line_color, line_size);
            }
            bounce_index += 1u;
        }
        color += draw_line_3d(debug_pixel, debug_dimensions, origin, origin + direction * 5., line_color, line_size);
        out_color = vec4<f32>(color, 1.);
    } 
    return out_color;
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
    let dimensions = textureDimensions(finalize_texture);
    let pixel = vec2<u32>(u32(v_in.uv.x * f32(dimensions.x)), u32(v_in.uv.y * f32(dimensions.y)));

    var out_color = textureLoad(finalize_texture, pixel, 0);    
    out_color = debug_color_override(out_color, pixel, dimensions); 
    return out_color;
}