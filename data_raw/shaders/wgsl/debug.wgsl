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
var<storage, read> vertices_positions: VerticesPositions;
@group(0) @binding(3)
var<storage, read> vertices_attributes: VerticesAttributes;
@group(0) @binding(4)
var<storage, read> meshes: Meshes;
@group(0) @binding(5)
var<storage, read> meshlets: Meshlets;
@group(0) @binding(6)
var<storage, read> instances: Instances;
@group(0) @binding(7)
var<storage, read> transforms: Transforms;

@group(1) @binding(0)
var<storage, read> materials: Materials;
@group(1) @binding(1)
var<storage, read> textures: Textures;
@group(1) @binding(2)
var<uniform> lights: Lights;
@group(1) @binding(3)
var visibility_texture: texture_2d<u32>;
@group(1) @binding(4)
var depth_texture: texture_depth_2d;

@group(3) @binding(0)
var<storage, read> data_buffer_0: array<f32>;
@group(3) @binding(1)
var<storage, read> data_buffer_1: array<f32>;
@group(3) @binding(2)
var<storage, read> data_buffer_2: array<f32>;
@group(3) @binding(3)
var<storage, read> data_buffer_debug: array<f32>;

#import "texture_utils.inc"
#import "geom_utils.inc"
#import "shape_utils.inc"
#import "matrix_utils.inc"
#import "material_utils.inc"
#import "pbr_utils.inc"
#import "visibility_utils.inc"
#import "color_utils.inc"


fn draw_triangle_from_visibility(visibility_id: u32, pixel: vec2<u32>, dimensions: vec2<u32>) -> vec3<f32>{
    let instance_id = (visibility_id >> 8u) - 1u;
    let meshlet_id = instances.data[instance_id].meshlet_id;
    let primitive_id = visibility_id & 255u;
    let meshlet = &meshlets.data[meshlet_id];
    let index_offset = (*meshlet).indices_offset + (primitive_id * 3u);

    let mesh_id = (*meshlet).mesh_index;
    let mesh = &meshes.data[mesh_id];
    let position_offset = (*mesh).vertices_position_offset;
    
    let vert_indices = vec3<u32>(indices.data[index_offset], indices.data[index_offset + 1u], indices.data[index_offset + 2u]);
    
    let instance = &instances.data[instance_id];
    let transform = transforms.data[(*instance).transform_id];
    let orientation = transform.orientation;
    let position = transform.position_scale_x.xyz;
    let scale = vec3<f32>(transform.position_scale_x.w, transform.bb_min_scale_y.w, transform.bb_min_scale_y.w);
    
    let min = transform.bb_min_scale_y.xyz;
    let size = abs(transform.bb_max_scale_z.xyz - min);
    let p1 = min + unpack_unorm_to_3_f32(vertices_positions.data[vert_indices.x + position_offset]) * size;
    let p2 = min + unpack_unorm_to_3_f32(vertices_positions.data[vert_indices.y + position_offset]) * size;
    let p3 = min + unpack_unorm_to_3_f32(vertices_positions.data[vert_indices.z + position_offset]) * size;
    let v1 = transform_vector(p1, position, orientation, scale);
    let v2 = transform_vector(p2, position, orientation, scale);
    let v3 = transform_vector(p3, position, orientation, scale);
    
    let line_color = vec3<f32>(0., 1., 1.);
    let line_size = 0.001;
    var color = vec3<f32>(0.);
    color += draw_line_3d(pixel, dimensions, v1, v2, line_color, line_size);
    color += draw_line_3d(pixel, dimensions, v2, v3, line_color, line_size);
    color += draw_line_3d(pixel, dimensions, v3, v1, line_color, line_size);
    return color;
}

fn draw_cube_from_min_max(min: vec3<f32>, max:vec3<f32>, pixel: vec2<u32>, dimensions: vec2<u32>) -> vec3<f32> {  
    let line_color = vec3<f32>(1., 1., 0.);
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
    let dimensions = vec2<u32>(u32(constant_data.screen_width), u32(constant_data.screen_height));
    let screen_pixel = vec2<u32>(u32(v_in.uv.x * f32(dimensions.x)), u32(v_in.uv.y * f32(dimensions.y)));

    var out_color = vec4<f32>(0.);
    let pixel = vec2<f32>(screen_pixel);
    
    if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS) != 0) {
        let visibility_dimensions = textureDimensions(visibility_texture);
        let visibility_scale = vec2<f32>(visibility_dimensions) / vec2<f32>(dimensions);
        let visibility_pixel = vec2<u32>(pixel * visibility_scale);
        let visibility_value = textureLoad(visibility_texture, visibility_pixel, 0);
        let visibility_id = visibility_value.r;
        var meshlet_id = 0xFFFFFFFFu;
        if (visibility_id != 0u && (visibility_id & 0xFFFFFFFFu) != 0xFF000000u) {
            let instance_id = (visibility_id >> 8u) - 1u; 
            meshlet_id = instances.data[instance_id].meshlet_id;
            let meshlet = meshlets.data[meshlet_id];
            let meshlet_color = get_color_for_int(meshlet_id);
            out_color = vec4<f32>(meshlet_color, 1.);
        }
        
        let debug_pixel = vec2<u32>(constant_data.debug_uv_coords * vec2<f32>(visibility_dimensions));
        let debug_visibility_value = textureLoad(visibility_texture, debug_pixel, 0);
        let debug_visibility_id = debug_visibility_value.r;
        if (debug_visibility_id != 0u && (debug_visibility_id & 0xFFFFFFFFu) != 0xFF000000u) {
            let debug_instance_id = (debug_visibility_id >> 8u) - 1u; 
            let debug_instance = instances.data[debug_instance_id];
            let debug_meshlet_id = debug_instance.meshlet_id;
            let debug_meshlet = meshlets.data[debug_meshlet_id];
            if(debug_meshlet_id == meshlet_id) {
                out_color = vec4<f32>(0.2, 0.2, 0.2, 0.1);
            }
                
            let transform = transforms.data[debug_instance.transform_id];
            let orientation = transform.orientation;
            let position = transform.position_scale_x.xyz;
            let scale = vec3<f32>(transform.position_scale_x.w, transform.bb_min_scale_y.w, transform.bb_min_scale_y.w);
            let min = transform_vector(debug_meshlet.aabb_min, position, orientation, scale);
            let max = transform_vector(debug_meshlet.aabb_max, position, orientation, scale);

            out_color += vec4<f32>(draw_cube_from_min_max(min, max, screen_pixel, dimensions), 1.);
        }
    } 
    else if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_LOD_LEVEL) != 0) {
        let visibility_dimensions = textureDimensions(visibility_texture);
        let visibility_scale = vec2<f32>(visibility_dimensions) / vec2<f32>(dimensions);
        let visibility_pixel = vec2<u32>(pixel * visibility_scale);
        let visibility_value = textureLoad(visibility_texture, visibility_pixel, 0);
        let visibility_id = visibility_value.r;
        if (visibility_id != 0u && (visibility_id & 0xFFFFFFFFu) != 0xFF000000u) {
            let instance_id = (visibility_id >> 8u) - 1u; 
            let meshlet_id = instances.data[instance_id].meshlet_id;
            let meshlet = meshlets.data[meshlet_id];
            if (meshlet.lod_level == MAX_LOD_LEVELS - 1) {
                out_color = vec4<f32>(vec3<f32>(1., 0., 0.), 1.);
            }
            else if (meshlet.lod_level == MAX_LOD_LEVELS - 2) {
                out_color = vec4<f32>(vec3<f32>(1., 1., 0.), 1.);
            }
            else if (meshlet.lod_level == MAX_LOD_LEVELS - 3) {
                out_color = vec4<f32>(vec3<f32>(0., 1., 0.), 1.);
            }
            else if (meshlet.lod_level == MAX_LOD_LEVELS - 4) {
                out_color = vec4<f32>(vec3<f32>(0., 1., 1.), 1.);
            }
            else if (meshlet.lod_level == MAX_LOD_LEVELS - 5) {
                out_color = vec4<f32>(vec3<f32>(0., 0., 1.), 1.);
            }
            else if (meshlet.lod_level == MAX_LOD_LEVELS - 6) {
                out_color = vec4<f32>(vec3<f32>(0.3, 0.3, 0.3), 1.);
            }
            else if (meshlet.lod_level == MAX_LOD_LEVELS - 7) {
                out_color = vec4<f32>(vec3<f32>(0.5, 0.5, 0.5), 1.);
            }
            else if (meshlet.lod_level == 0) {
                out_color = vec4<f32>(vec3<f32>(1., 1., 1.), 1.);
            } else {
                out_color = vec4<f32>(vec3<f32>(0.1, 0.1, 0.1), 1.);
            }
        }
    } 
    else if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_UV_0) != 0) {
        let depth_dimensions = textureDimensions(depth_texture);
        let depth_scale = vec2<f32>(depth_dimensions) / vec2<f32>(dimensions);
        let depth_pixel = vec2<u32>(pixel * depth_scale);
        let depth = textureLoad(depth_texture, depth_pixel, 0);
        let hit_point = pixel_to_world(depth_pixel, depth_dimensions, depth); 

        let visibility_dimensions = textureDimensions(visibility_texture);
        let visibility_scale = vec2<f32>(visibility_dimensions) / vec2<f32>(dimensions);
        let visibility_pixel = vec2<u32>(pixel * visibility_scale);
        let visibility_value = textureLoad(visibility_texture, visibility_pixel, 0);
        let visibility_id = visibility_value.r;
        
        if (visibility_id != 0u && (visibility_id & 0xFFFFFFFFu) != 0xFF000000u) {
            var pixel_data = visibility_to_gbuffer(visibility_id, hit_point);
            out_color = vec4<f32>(vec3<f32>(pixel_data.uv_set[0].xy, 0.), 1.);
        }
    } 
    else if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_UV_1) != 0) {
        let depth_dimensions = textureDimensions(depth_texture);
        let depth_scale = vec2<f32>(depth_dimensions) / vec2<f32>(dimensions);
        let depth_pixel = vec2<u32>(pixel * depth_scale);
        let depth = textureLoad(depth_texture, depth_pixel, 0);
        let hit_point = pixel_to_world(depth_pixel, depth_dimensions, depth); 

        let visibility_dimensions = textureDimensions(visibility_texture);
        let visibility_scale = vec2<f32>(visibility_dimensions) / vec2<f32>(dimensions);
        let visibility_pixel = vec2<u32>(pixel * visibility_scale);
        let visibility_value = textureLoad(visibility_texture, visibility_pixel, 0);
        let visibility_id = visibility_value.r;
        
        if (visibility_id != 0u && (visibility_id & 0xFFFFFFFFu) != 0xFF000000u) {
            var pixel_data = visibility_to_gbuffer(visibility_id, hit_point);
            out_color = vec4<f32>(vec3<f32>(pixel_data.uv_set[1].xy, 0.), 1.);
        }
    } 
    else if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_UV_2) != 0) {
        let depth_dimensions = textureDimensions(depth_texture);
        let depth_scale = vec2<f32>(depth_dimensions) / vec2<f32>(dimensions);
        let depth_pixel = vec2<u32>(pixel * depth_scale);
        let depth = textureLoad(depth_texture, depth_pixel, 0);
        let hit_point = pixel_to_world(depth_pixel, depth_dimensions, depth); 

        let visibility_dimensions = textureDimensions(visibility_texture);
        let visibility_scale = vec2<f32>(visibility_dimensions) / vec2<f32>(dimensions);
        let visibility_pixel = vec2<u32>(pixel * visibility_scale);
        let visibility_value = textureLoad(visibility_texture, visibility_pixel, 0);
        let visibility_id = visibility_value.r;
        
        if (visibility_id != 0u && (visibility_id & 0xFFFFFFFFu) != 0xFF000000u) {
            var pixel_data = visibility_to_gbuffer(visibility_id, hit_point);
            out_color = vec4<f32>(vec3<f32>(pixel_data.uv_set[2].xy, 0.), 1.);
        }
    } 
    else if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_UV_3) != 0) {
        let depth_dimensions = textureDimensions(depth_texture);
        let depth_scale = vec2<f32>(depth_dimensions) / vec2<f32>(dimensions);
        let depth_pixel = vec2<u32>(pixel * depth_scale);
        let depth = textureLoad(depth_texture, depth_pixel, 0);
        let hit_point = pixel_to_world(depth_pixel, depth_dimensions, depth); 

        let visibility_dimensions = textureDimensions(visibility_texture);
        let visibility_scale = vec2<f32>(visibility_dimensions) / vec2<f32>(dimensions);
        let visibility_pixel = vec2<u32>(pixel * visibility_scale);
        let visibility_value = textureLoad(visibility_texture, visibility_pixel, 0);
        let visibility_id = visibility_value.r;
        
        if (visibility_id != 0u && (visibility_id & 0xFFFFFFFFu) != 0xFF000000u) {
            var pixel_data = visibility_to_gbuffer(visibility_id, hit_point);
            out_color = vec4<f32>(vec3<f32>(pixel_data.uv_set[3].xy, 0.), 1.);
        }
    } 
    else if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_NORMALS) != 0) {
        let depth_dimensions = textureDimensions(depth_texture);
        let depth_scale = vec2<f32>(depth_dimensions) / vec2<f32>(dimensions);
        let depth_pixel = vec2<u32>(pixel * depth_scale);
        let depth = textureLoad(depth_texture, depth_pixel, 0);
        let hit_point = pixel_to_world(depth_pixel, depth_dimensions, depth); 

        let visibility_dimensions = textureDimensions(visibility_texture);
        let visibility_scale = vec2<f32>(visibility_dimensions) / vec2<f32>(dimensions);
        let visibility_pixel = vec2<u32>(pixel * visibility_scale);
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
        let depth_pixel = vec2<u32>(pixel * depth_scale);
        let depth = textureLoad(depth_texture, depth_pixel, 0);
        let hit_point = pixel_to_world(depth_pixel, depth_dimensions, depth); 

        let visibility_dimensions = textureDimensions(visibility_texture);
        let visibility_scale = vec2<f32>(visibility_dimensions) / vec2<f32>(dimensions);
        let visibility_pixel = vec2<u32>(pixel * visibility_scale);
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
        let depth_pixel = vec2<u32>(pixel * depth_scale);
        let depth = textureLoad(depth_texture, depth_pixel, 0);
        let hit_point = pixel_to_world(depth_pixel, depth_dimensions, depth); 

        let visibility_dimensions = textureDimensions(visibility_texture);
        let visibility_scale = vec2<f32>(visibility_dimensions) / vec2<f32>(dimensions);
        let visibility_pixel = vec2<u32>(pixel * visibility_scale);
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
        let depth_pixel = vec2<u32>(pixel * depth_scale);
        let depth = textureLoad(depth_texture, depth_pixel, 0);
        let hit_point = pixel_to_world(depth_pixel, depth_dimensions, depth); 

        let visibility_dimensions = textureDimensions(visibility_texture);
        let visibility_scale = vec2<f32>(visibility_dimensions) / vec2<f32>(dimensions);
        let visibility_pixel = vec2<u32>(pixel * visibility_scale);
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
        let depth_pixel = vec2<u32>(pixel * depth_scale);
        let depth = textureLoad(depth_texture, depth_pixel, 0);
        let hit_point = pixel_to_world(depth_pixel, depth_dimensions, depth); 

        let visibility_dimensions = textureDimensions(visibility_texture);
        let visibility_scale = vec2<f32>(visibility_dimensions) / vec2<f32>(dimensions);
        let visibility_pixel = vec2<u32>(pixel * visibility_scale);
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
        let depth_pixel = vec2<u32>(pixel * depth_scale);
        let depth = textureLoad(depth_texture, depth_pixel, 0);
        let hit_point = pixel_to_world(depth_pixel, depth_dimensions, depth); 

        let visibility_dimensions = textureDimensions(visibility_texture);
        let visibility_scale = vec2<f32>(visibility_dimensions) / vec2<f32>(dimensions);
        let visibility_pixel = vec2<u32>(pixel * visibility_scale);
        let visibility_value = textureLoad(visibility_texture, visibility_pixel, 0);
        let visibility_id = visibility_value.r;
        
        if (visibility_id != 0u && (visibility_id & 0xFFFFFFFFu) != 0xFF000000u) {
            var pixel_data = visibility_to_gbuffer(visibility_id, hit_point);
            let material_info = compute_color_from_material(pixel_data.material_id, &pixel_data);
            out_color = vec4<f32>(vec3<f32>(material_info.perceptual_roughness), 1.);
        }
    } 
    else if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_RADIANCE_BUFFER) != 0) {
        let data_dimensions = vec2<u32>(DEFAULT_WIDTH, DEFAULT_HEIGHT);
        let data_scale = vec2<f32>(data_dimensions) / vec2<f32>(dimensions);
        let data_pixel = vec2<u32>(pixel * data_scale);
        let data_index = (data_pixel.y * data_dimensions.x + data_pixel.x) * SIZE_OF_DATA_BUFFER_ELEMENT;
        let radiance = vec3<f32>(data_buffer_1[data_index], data_buffer_1[data_index + 1u], data_buffer_1[data_index + 2u]);
        out_color = vec4<f32>(radiance, 1.);
    } 
    else if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_DEPTH_BUFFER) != 0) {
        let depth_dimensions = textureDimensions(depth_texture);
        let depth_scale = vec2<f32>(depth_dimensions) / vec2<f32>(dimensions);
        let depth_pixel = vec2<u32>(pixel * depth_scale);
        let depth = textureLoad(depth_texture, depth_pixel, 0);
        let v = vec3<f32>(1. - depth) * 10.;
        out_color = vec4<f32>(v, 1.);
    } 
    /*
    else if ((constant_data.flags & CONSTANT_DATA_FLAGS_DISPLAY_PATHTRACE) != 0) {
        var origin = vec3<f32>(0.);
        var direction = vec3<f32>(0.);
        let line_color = vec3<f32>(0., 1., 0.);
        let line_size = 0.003;
        var bounce_index = 0u;
        
        let data_dimensions = vec2<u32>(DEFAULT_WIDTH, DEFAULT_HEIGHT);
        let data_scale = vec2<f32>(data_dimensions) / vec2<f32>(dimensions);
        let data_pixel = vec2<u32>(pixel * data_scale);
        let data_index = (data_pixel.y * data_dimensions.x + data_pixel.x) * SIZE_OF_DATA_BUFFER_ELEMENT;
        let radiance = vec3<f32>(data_buffer_1[data_index], data_buffer_1[data_index + 1u], data_buffer_1[data_index + 2u]);
        var color = radiance.rgb;        
        /*
        var debug_bvh_index = 100u;
        let max_bvh_index = u32(read_value_from_data_buffer(&data_buffer_debug, &debug_bvh_index));
        while(debug_bvh_index < max_bvh_index) 
        {
            var min = read_vec3_from_data_buffer(&data_buffer_debug, &debug_bvh_index);
            var max = read_vec3_from_data_buffer(&data_buffer_debug, &debug_bvh_index);
            color += draw_cube_from_min_max(min, max, screen_pixel, dimensions);
            //color += draw_line_3d(screen_pixel, dimensions, min, max, vec3<f32>(0.,0.,1.), line_size);
        }
        */
        
        var debug_index = 0u;        
        let max_index = u32(data_buffer_debug[debug_index]);
        debug_index = debug_index + 1u;
        if(max_index > 8u) {
            while(debug_index < max_index) {
                let visibility_id = u32(data_buffer_debug[debug_index]);
                color += draw_triangle_from_visibility(visibility_id, screen_pixel, dimensions);
                
                var previous = origin;
                origin = vec3<f32>(data_buffer_debug[debug_index + 1u], data_buffer_debug[debug_index + 2u], data_buffer_debug[debug_index + 3u]);
                direction = vec3<f32>(data_buffer_debug[debug_index + 4u], data_buffer_debug[debug_index + 5u], data_buffer_debug[debug_index + 6u]);
                
                if (bounce_index > 0u) {
                    color += draw_line_3d(screen_pixel, dimensions, previous, origin, line_color, line_size);
                }
                bounce_index += 1u;
                debug_index = debug_index + 7u;
            }
            color += draw_line_3d(screen_pixel, dimensions, origin, origin + direction * 5., line_color, line_size);
            out_color = vec4<f32>(color, 1.);
        }
    }*/
    
    return select(vec4<f32>(0.), out_color, out_color.a > 0.);
}