#import "common.inc"
#import "utils.inc"
#import "vertex_utils.inc"
#import "material_utils.inc"
#import "pbr_utils.inc"
#import "ray_data.inc"
#import "ray_types.inc"
#import "sampling.inc"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;

// Read-write textures for accumulating indirect lighting (using integer format for atomic operations)
@group(0) @binding(1)
var indirect_diffuse_texture: texture_storage_2d<rgba32uint, read_write>;
@group(0) @binding(2)
var indirect_specular_texture: texture_storage_2d<rgba32uint, read_write>;

// Group 0: Ray Data (bindings 3-5 adjusted for fewer texture bindings)
@group(0) @binding(3)
var<storage, read> rays: Rays;
@group(0) @binding(4)
var<storage, read> intersections: Intersections;
@group(0) @binding(5)
var<storage, read_write> rays_next: Rays;

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

// Materials and Lights
// Group 2: Materials, Textures, Lights
@group(2) @binding(0)
var<uniform> materials: Materials;
@group(2) @binding(1)
var<uniform> textures: Textures;
@group(2) @binding(2)
var<uniform> lights: Lights;

#import "texture_utils.inc"

const WORKGROUP_SIZE: u32 = 64u;

fn get_pixel_data_from_intersection(ray: PathRay, intersection: Intersection) -> PixelData {
    let hit_point = ray.origin + ray.direction * intersection.t;
    let u = intersection.u;
    let v = intersection.v;
    let w = 1.0 - u - v;
    let barycentrics = vec3<f32>(w, u, v);
    
    let primitive_index = u32(intersection.primitive_index);
    let index_offset = primitive_index * 3u;
    
    return get_pixel_data_from_triangle(
        u32(intersection.instance_id),
        index_offset,
        hit_point,
        barycentrics,
        true
    );
}

@compute
@workgroup_size(WORKGROUP_SIZE, 1, 1)
fn main(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let ray_index = global_invocation_id.x;
    if (ray_index >= arrayLength(&rays.data)) {
        return;
    }

    let ray = rays.data[ray_index];
    let intersection = intersections.data[ray_index];
    
    // Skip inactive or terminated rays
    if ((ray.flags & RAY_FLAG_ACTIVE) == 0u || (ray.flags & RAY_FLAG_TERMINATED) != 0u) {
        return;
    }
    
    // Check for miss
    if (intersection.instance_id < 0) {
        // Environment contribution via IBL
        let pixel_index = ray.pixel_index;
        let dimensions = vec2<u32>(DEFAULT_WIDTH, DEFAULT_HEIGHT);
        let pixel = vec2<u32>(pixel_index % dimensions.x, pixel_index / dimensions.x);
        
        // Sample environment map in ray direction for indirect lighting
        if ((constant_data.flags & CONSTANT_DATA_FLAGS_USE_IBL) != 0) {
            
            // Verify IBL Contribution
            // Use ray_index to reconstruct coord, ensuring match with generation logic
            let half_width = (DEFAULT_WIDTH + 1u) / 2u;
            let write_x = ray_index % half_width;
            let write_y = ray_index / half_width;
            let write_coord = vec2<u32>(write_x, write_y);

            // Bounds Check
            if (write_x * 2u >= u32(constant_data.screen_width) || write_y * 2u >= u32(constant_data.screen_height)) {
               return; // Skip inactive rays in fixed buffer
            }
            
            let env_radiance = sample_environment_ibl(ray.direction);
            let contribution = env_radiance * ray.throughput;
            
            // Accumulate IBL contributions from all bounces
            if (ray.ray_type == RAY_TYPE_DIFFUSE_BOUNCE) {
                let prev_encoded = textureLoad(indirect_diffuse_texture, write_coord).rgb;
                let prev_value = decode_uvec3_to_vec3(prev_encoded);
                let accumulated = prev_value + contribution;
                let encoded = encode_vec3_to_uvec3(accumulated);
                textureStore(indirect_diffuse_texture, write_coord, vec4<u32>(encoded, 0u));
            } else {
                let prev_encoded = textureLoad(indirect_specular_texture, write_coord).rgb;
                let prev_value = decode_uvec3_to_vec3(prev_encoded);
                let accumulated = prev_value + contribution;
                let encoded = encode_vec3_to_uvec3(accumulated);
                textureStore(indirect_specular_texture, write_coord, vec4<u32>(encoded, 0u));
            }
        }
        
        // Terminate ray after environment contribution
        rays_next.data[ray_index].t_max = -1.0;
        rays_next.data[ray_index].flags = RAY_FLAG_TERMINATED;
        return;
    }

    // Hit processing
    var pixel_data = get_pixel_data_from_intersection(ray, intersection);
    
    // Use robust material preparation (matches direct lighting pass)
    var material = materials.data[pixel_data.material_id];
    var v: vec3<f32>; // View vector (will be computed by prepare_material)
    var tbn: TBN;
    var material_info = prepare_material(&material, &pixel_data, &tbn, &v);
    
    // Extract properties from standardized material_info
    let c_diff = material_info.c_diff;
    let f0 = material_info.f0;
    
    // Explicitly compute emissive using the same logic as direct lighting
    // (prepare_material doesn't fully bake emissive * strength into a single float until combine, so we do it here)
    var emissive = material.emissive_color.rgb * material.emissive_strength;
    if (has_texture(&material, TEXTURE_TYPE_EMISSIVE)) {
        let uv = material_texture_uv(&material, &pixel_data, TEXTURE_TYPE_EMISSIVE);
        let texture_color = sample_texture(uv);
        emissive *= texture_color.rgb;
    }
    
    // Prevent double counting: 
    // Ideally we should check if THIS specific instance is in the light list.
    // For now, we allow it to ensure Emissive Meshes appear even if NEE is active.
    // if (constant_data.num_lights > 0u) {
    //     emissive = vec3(0.0);
    // }
    
    // Evaluate direct lighting at bounce hit point for indirect illumination contribution
    let pixel_index = ray.pixel_index;
    // Generate random seed early for light sampling
    // Mix in bounce_count and ray origin to ensure unique random sequence per bounce
    let origin_hash = bitcast<u32>(pixel_data.world_pos.x) ^ bitcast<u32>(pixel_data.world_pos.y) ^ bitcast<u32>(pixel_data.world_pos.z);
    var seed = vec2<u32>(pixel_index ^ origin_hash, constant_data.frame_index + ray_index + (ray.bounce_count * 15485863u));

    var light_contribution = vec3(0.0);
    
    // Only sample lights if we actually have them!
    if (constant_data.num_lights > 0u) {
        // Pick one random light (same approach as direct lighting pass)
        let light_index = hash(constant_data.frame_index + ray_index) % constant_data.num_lights;
        let light = lights.data[light_index];
        
        if (light.light_type != LIGHT_TYPE_INVALID) {
             var point_to_light: vec3<f32>;
            if (light.light_type == LIGHT_TYPE_DIRECTIONAL) { 
                 point_to_light = -light.direction;
            } else {
                 point_to_light = light.position - pixel_data.world_pos;
                 
                 // Area Light Sampling
                 if (light.light_type == LIGHT_TYPE_AREA) {
                    let rnd_light = get_random_numbers(&seed);
                    // Construct Basis
                    let up = select(vec3(0., 1., 0.), vec3(1., 0., 0.), abs(light.direction.y) > 0.999);
                    let tangent = normalize(cross(up, light.direction));
                    let bitangent = cross(light.direction, tangent);
                    
                    // Dimensions from cone angles (Hack)
                    let width = light.inner_cone_angle;
                    let height = light.outer_cone_angle;
                    
                    var offset = vec3(0.0);
                    
                    // Decode Shape from _padding1 (f32 -> u32)
                    let shape_type = bitcast<u32>(light._padding1);
                    
                    if (shape_type == LIGHT_AREA_SHAPE_DISK) {
                        // Uniform Disk Sampling
                        let r = sqrt(rnd_light.x);            // Square root for uniform area distribution
                        let theta = 2.0 * MATH_PI * rnd_light.y;
                        let radius_x = width * 0.5;
                        let radius_z = height * 0.5;
                        
                        let u = r * cos(theta) * radius_x;
                        let v = r * sin(theta) * radius_z;
                        
                        offset = u * tangent + v * bitangent;
                    } else {
                        // Rectangle Sampling
                        offset = (rnd_light.x - 0.5) * width * tangent + (rnd_light.y - 0.5) * height * bitangent;
                    }
                    
                    point_to_light = (light.position + offset) - pixel_data.world_pos;
                 }
            }
            
            let L = normalize(point_to_light);
            let N = normalize(pixel_data.normal);
            let NdotL = clamp(dot(N, L), 0.0, 1.0);
            
            if (NdotL > 0.0) {
                var light_nee: LightData = light;
                if (light.light_type == LIGHT_TYPE_DIRECTIONAL) { 
                     light_nee.position = pixel_data.world_pos - light.direction; // Fake position for punctual eval
                }
                if (light.light_type == LIGHT_TYPE_AREA) {
                     light_nee.position = pixel_data.world_pos + point_to_light; // Evaluated position
                }
                // Use eval_punctual_light which handles full PBR (diffuse + spec + sheen + clearcoat)
                // It accumulates into material_info (which starts at 0 for light accumulators)
                eval_punctual_light(&light_nee, material, &material_info, &pixel_data, tbn, v);
                
                // Extract the accumulated light
                // Note: eval_punctual_light does: material_info.f_diffuse += contribution...
                // So we sum them up.
                light_contribution = material_info.f_diffuse + material_info.f_specular + material_info.f_sheen + material_info.f_clearcoat + material_info.f_transmission;
            }
        }
    }
    
    // Radiance = emissive + lights (NO AMBIENT in bounces to avoid energy explosion)
    // Ambient is applied only in Direct pass or Finalize
    // Radiance = emissive + lights (NO AMBIENT in bounces to avoid energy explosion)
    // Ambient is applied only in Direct pass or Finalize
    let radiance = emissive + light_contribution;

    // Apply throughput
    // Apply throughput
    let contribution = radiance * ray.throughput;
    
    
    // Robustness Fix: Unpack X and Y from pixel_index
    let packed_coord = ray.pixel_index;
    // Reconstruct pixel coordinates from ray_index to ensure stride alignment
    // This matches the dispatch logic: ray_index = y * (DEFAULT_WIDTH/2) + x
    let half_width = (DEFAULT_WIDTH + 1u) / 2u;
    let px = (ray_index % half_width) * 2u;
    let py = (ray_index / half_width) * 2u;
    let pixel = vec2<u32>(px, py);

    // Bounds Check: Since buffer is fixed size (DEFAULT), we might be processing 
    // a ray that is outside the current window size (hole/padded area).
    if (px >= u32(constant_data.screen_width) || py >= u32(constant_data.screen_height)) {
        return;
    }
    
    // Write contribution to indirect textures
    if (ray.ray_type == RAY_TYPE_DIFFUSE_BOUNCE) {
        // Reduced Resolution: Write to pixel / 2
        let write_coord = pixel / 2u;
        
        let prev_data = textureLoad(indirect_diffuse_texture, write_coord);
        let prev_encoded = prev_data.rgb;
        let prev_value = decode_uvec3_to_vec3(prev_encoded);
        
        // Accumulate radiance contribution
        let accumulated = prev_value + contribution;
        
        let encoded = encode_vec3_to_uvec3(accumulated);
        textureStore(indirect_diffuse_texture, write_coord, vec4<u32>(encoded, 0u));
    } else {
        let write_coord = pixel / 2u;
        
        let prev_data = textureLoad(indirect_specular_texture, write_coord);
        let prev_encoded = prev_data.rgb;
        let prev_value = decode_uvec3_to_vec3(prev_encoded);
        
        let accumulated = prev_value + contribution;
        
        let encoded = encode_vec3_to_uvec3(accumulated);
        textureStore(indirect_specular_texture, write_coord, vec4<u32>(encoded, 0u));
    }
    
    // Check if we hit a strong emissive surface (acts like hitting a light)
    // If so, terminate the path after accumulating its contribution
    let emissive_luminance = dot(emissive, vec3<f32>(0.299, 0.587, 0.114));
    if (emissive_luminance > 0.1) {
        // Hit an emissive surface - terminate path after contribution
        rays_next.data[ray_index].t_max = -1.0;
        rays_next.data[ray_index].flags = RAY_FLAG_TERMINATED;
        return;
    }
    
    // Generate next bounce
    
    let rnd = get_random_numbers(&seed);
    let rnd_dir = get_random_numbers(&seed);
    
    let cam_pos = constant_data.inv_view[3].xyz;
    let V = normalize(cam_pos - pixel_data.world_pos);
    
    let diffuse_weight = length(c_diff);
    let specular_weight = length(f0);
    let total_weight = diffuse_weight + specular_weight;
    
    if (total_weight > 0.001 && length(ray.throughput) > 0.01) {
        let p_specular = specular_weight / total_weight;
        
        let N = normalize(pixel_data.normal);
        let up = select(vec3(1., 0., 0.), vec3(0., 1., 0.), abs(N.z) < 0.999);
        let tangent = normalize(cross(up, N));
        let bitangent = cross(N, tangent);
        let tbn_sample = mat3x3<f32>(tangent, bitangent, N);
        
        var next_dir: vec3<f32>;
        var throughput_adj: vec3<f32>;
        var next_ray_type: u32;
        
        if (rnd.x < p_specular) {
            let local_H = importance_sample_ggx(material.roughness_factor, rnd_dir);
            let H = normalize(tbn_sample * local_H);
            next_dir = normalize(reflect(-V, H));
            next_ray_type = RAY_TYPE_SPECULAR_BOUNCE;
            
            let NdotL_next = clamp(dot(N, next_dir), 0.001, 1.0);
            let VdotH = clamp(dot(V, H), 0.001, 1.0);
            let NdotH = clamp(dot(N, H), 0.001, 1.0);
            let NdotV = clamp(dot(N, V), 0.001, 1.0);
            
            // Weight = BRDF * NdotL / PDF
            // PDF(L) = D * NH / (4 * VH)
            // BRDF = F * G * D / (4 * NL * NV)
            // Weight = (F * G * D * NL / (4 * NL * NV)) / (D * NH / (4 * VH))
            //        = F * G * VH / (NV * NH)
            
            let F = f_schlick_vec3_vec3(f0, material_info.f90, VdotH);
            let Vis = V_GGX(NdotL_next, NdotV, material_info.alpha_roughness);
            // Vis = G / (4 * NL * NV) => G = Vis * 4 * NL * NV
            let G = Vis * 4.0 * NdotL_next * NdotV;
            
            throughput_adj = (F * G * VdotH) / (NdotV * NdotH * p_specular);
            
        } else {
            let local_dir = sample_cosine_weighted_hemisphere(rnd_dir);
            next_dir = normalize(tbn_sample * local_dir);
            next_ray_type = RAY_TYPE_DIFFUSE_BOUNCE;
            
            // Diffuse BRDF weight for path tracing
            // PDF = cos(theta) / pi (cosine-weighted hemisphere sampling)
            // BRDF_diffuse = c_diff / pi (Lambertian)
            // Weight = BRDF * cos(theta) / PDF = c_diff
            // Then scale by 1/(1-p_specular) to account for MIS probability
            
            throughput_adj = c_diff / (1.0 - p_specular);
        }
        
        rays_next.data[ray_index].origin = pixel_data.world_pos + next_dir * 0.01;
        rays_next.data[ray_index].direction = next_dir;
        rays_next.data[ray_index].throughput = ray.throughput * throughput_adj;
        rays_next.data[ray_index].t_max = MAX_TRACING_DISTANCE;
        rays_next.data[ray_index].t_min = 0.001;
        rays_next.data[ray_index].pixel_index = pixel_index;
        rays_next.data[ray_index].ray_type = next_ray_type;
        rays_next.data[ray_index].bounce_count = ray.bounce_count + 1u;
        rays_next.data[ray_index].flags = RAY_FLAG_ACTIVE;
    } else {
        // Terminate ray
        rays_next.data[ray_index].t_max = -1.0;
        rays_next.data[ray_index].flags = RAY_FLAG_TERMINATED;
    }
}
