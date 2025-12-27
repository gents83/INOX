#import "common.inc"
#import "utils.inc"
#import "vertex_utils.inc"
#import "visibility_utils.inc"
#import "material_utils.inc"
#import "pbr_utils.inc"
#import "ray_data.inc"
#import "ray_types.inc"
#import "sampling.inc"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var visibility_texture: texture_2d<u32>;
@group(0) @binding(2)
var depth_texture: texture_depth_2d;
@group(0) @binding(3)
var direct_lighting_texture: texture_storage_2d<rgba16float, write>;
@group(0) @binding(4)
var<storage, read_write> rays: Rays;
@group(0) @binding(5)
var indirect_diffuse_texture: texture_storage_2d<rgba32uint, write>;
@group(0) @binding(6)
var indirect_specular_texture: texture_storage_2d<rgba32uint, write>;

// Group 1: Geometry
@group(1) @binding(0)
var<storage, read> indices: Indices;
@group(1) @binding(1)
var<storage, read> vertices_positions: VerticesPositions;
@group(1) @binding(2)
var<storage, read> vertices_attributes: VerticesAttributes;
@group(1) @binding(3)
var<storage, read> instances: Instances;
@group(1) @binding(4)
var<storage, read> transforms: Transforms;
@group(1) @binding(5)
var<storage, read> meshes: Meshes;
@group(1) @binding(6)
var<storage, read> meshlets: Meshlets;

// Group 2: Materials, Textures, Lights
@group(2) @binding(0)
var<uniform> materials: Materials;
@group(2) @binding(1)
var<uniform> textures: Textures;
@group(2) @binding(2)
var<uniform> lights: Lights;

#import "texture_utils.inc"

const WORKGROUP_SIZE: u32 = 8u;

@compute @workgroup_size(WORKGROUP_SIZE, WORKGROUP_SIZE, 1)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let dimensions = vec2<u32>(DEFAULT_WIDTH, DEFAULT_HEIGHT);
    let pixel = vec2<u32>(global_invocation_id.x, global_invocation_id.y);
    if (pixel.x >= dimensions.x || pixel.y >= dimensions.y) {
        return;
    }
    
    // Clear indirect lighting textures at frame start (prevents ghosting)
    textureStore(indirect_diffuse_texture, pixel, vec4<u32>(0u));
    textureStore(indirect_specular_texture, pixel, vec4<u32>(0u));

    var direct_light = vec3<f32>(0.0);

    let visibility_dimensions = textureDimensions(visibility_texture);
    let visibility_scale = vec2<f32>(visibility_dimensions) / vec2<f32>(dimensions);
    let visibility_pixel = vec2<u32>(vec2<f32>(pixel) * visibility_scale);
    let visibility_value = textureLoad(visibility_texture, visibility_pixel, 0);
    let visibility_id = visibility_value.r;

    if (visibility_id != 0u && (visibility_id & 0xFFFFFFFFu) != 0xFF000000u) {
        let depth_dimensions = textureDimensions(depth_texture);
        let depth_scale = vec2<f32>(depth_dimensions) / vec2<f32>(dimensions);
        let depth_pixel = vec2<u32>(vec2<f32>(pixel) * depth_scale);
        let depth = textureLoad(depth_texture, depth_pixel, 0);
        
        let hit_point = pixel_to_world(depth_pixel, depth_dimensions, depth);
        
        // Reconstruct pixel data from visibility
        var pixel_data = visibility_to_gbuffer(visibility_id, hit_point);
        
        // Compute PBR lighting (direct lighting only, no IBL)
        let material_info_direct = compute_direct_lighting_only(pixel_data.material_id, &pixel_data);
        direct_light = material_info_direct.f_color.rgb;
        
        // Generate bounce ray for path tracing
        
        // Use robust material preparation
        var material = materials.data[pixel_data.material_id];
        var v: vec3<f32>; 
        var tbn: TBN;
        var material_info = prepare_material(&material, &pixel_data, &tbn, &v);
        
        let c_diff = material_info.c_diff;
        let f0 = material_info.f0;
        let diffuse_weight = length(c_diff);
        let specular_weight_len = length(f0);
        let total_weight = diffuse_weight + specular_weight_len;
        
        // Half-Resolution GI optimization.
        // We only compute indirect lighting for every 2x2 block (High-Res pixels x%2==0 && y%2==0).
        // This reduces Ray Buffer usage by 4x, allowing 2556x1490 resoluion to fit in a 2M buffer.
        if (pixel.x % 2u == 0u && pixel.y % 2u == 0u) {
            let screen_width = u32(constant_data.screen_width);
            
            // Force Dense Packing: Stride = Number of Even Pixels
            let half_width = (screen_width + 1u) / 2u;
            
            let ray_index = (pixel.y / 2u) * half_width + (pixel.x / 2u);
            
            if (ray_index < arrayLength(&rays.data)) {
                 // Robustness Fix: Pack X and Y into pixel_index (16 bits each)
                let packed_coord = (pixel.y << 16u) | (pixel.x & 0xFFFFu);
                rays.data[ray_index].pixel_index = packed_coord;
                
                var seed = vec2<u32>(ray_index, constant_data.frame_index);
                let rnd = get_random_numbers(&seed);
                let rnd_dir = get_random_numbers(&seed);
                
                var next_dir: vec3<f32>;
                var throughput_adj: vec3<f32>;
                var next_ray_type: u32;
                
                // Probability of choosing specular bounce
                let p_specular = select(0.0, specular_weight_len / total_weight, total_weight > 0.001);
                
                if (rnd.x < p_specular) {
                    // Specular Bounce (GGX)
                    let local_H = importance_sample_ggx(material_info.perceptual_roughness, rnd_dir);
                    let H = normalize(mat3x3<f32>(tbn.tangent, tbn.binormal, tbn.normal) * local_H);
                    next_dir = normalize(reflect(-v, H));
                    next_ray_type = RAY_TYPE_SPECULAR_BOUNCE;
                    
                    let NdotL_next = clamp(dot(tbn.normal, next_dir), 0.001, 1.0);
                    let NdotV = clamp(dot(tbn.normal, v), 0.001, 1.0);
                    let NdotH = clamp(dot(tbn.normal, H), 0.001, 1.0);
                    let VdotH = clamp(dot(v, H), 0.001, 1.0);
                    
                    let F = f_schlick_vec3_vec3(f0, material_info.f90, VdotH); 
                    let Vis = V_GGX(NdotL_next, NdotV, material_info.alpha_roughness); // Vis = G / (4*NL*NV)
                    let G = Vis * 4.0 * NdotL_next * NdotV; 

                    // PDF = D * NH / (4 * VH)
                    // BRDF = F * G * D / (4 * NL * NV)
                    // Weight = BRDF * NL / PDF 
                    //        = (F * G * D * NL / (4 * NL * NV)) / (D * NH / (4 * VH))
                    //        = F * G * VH / (NV * NH)

                    throughput_adj = (F * G * VdotH) / (NdotV * NdotH * p_specular);

                } else {
                    // Diffuse Bounce (Cosine Weighted)
                    let local_dir = sample_cosine_weighted_hemisphere(rnd_dir);
                    next_dir = normalize(mat3x3<f32>(tbn.tangent, tbn.binormal, tbn.normal) * local_dir);
                    next_ray_type = RAY_TYPE_DIFFUSE_BOUNCE;
                    
                    // Weight = (1 - k_spec * F) * Albedo
                     let H_diff = normalize(v + next_dir);
                    let VdotH_diff = clamp(dot(v, H_diff), 0.0, 1.0);
                    let k_spec = unpack2x16float(material_info.specular_weight_and_anisotropy_strength).x;
                    let F_diff = f_schlick_vec3_vec3(f0, material_info.f90, VdotH_diff);
                    
                    throughput_adj = ((1.0 - k_spec * F_diff) * c_diff) / (1.0 - p_specular);
                }
                
                rays.data[ray_index].origin = pixel_data.world_pos + next_dir * 0.01;
                rays.data[ray_index].direction = next_dir;
                rays.data[ray_index].throughput = throughput_adj; 
                rays.data[ray_index].t_max = MAX_TRACING_DISTANCE;
                rays.data[ray_index].t_min = 0.001;
                rays.data[ray_index].pixel_index = packed_coord; // Use packed coord (Wait, I used ray_index before, need packed!)
                // Wait, I defined packed_coord above.
                // Re-using packed_coord here.
                rays.data[ray_index].ray_type = next_ray_type;
                rays.data[ray_index].bounce_count = 0u;
                rays.data[ray_index].flags = RAY_FLAG_ACTIVE;
            }
        }
    } else {
        // No hit - mark ray as terminated (Only if Even)
        if (pixel.x % 2u == 0u && pixel.y % 2u == 0u) {
            let screen_width = u32(constant_data.screen_width);
            let half_width = (screen_width + 1u) / 2u;
            let ray_index = (pixel.y / 2u) * half_width + (pixel.x / 2u);
            
             if (ray_index < arrayLength(&rays.data)) {
                rays.data[ray_index].t_max = -1.0;
                rays.data[ray_index].flags = RAY_FLAG_TERMINATED;
             }
        }
    }

    // Write direct lighting
    textureStore(direct_lighting_texture, pixel, vec4<f32>(direct_light, 1.0));
}
