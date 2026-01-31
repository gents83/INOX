#import "common.inc"
#import "pathtracing_common.inc"

@group(0) @binding(0) var<uniform> constant_data: ConstantData;
@group(0) @binding(1) var<storage, read> indices: Indices;
@group(0) @binding(2) var<storage, read> vertices_positions: VerticesPositions;
@group(0) @binding(3) var<storage, read> vertices_attributes: VerticesAttributes;
@group(0) @binding(4) var<storage, read> instances: Instances;
@group(0) @binding(5) var<storage, read> transforms: Transforms;
@group(0) @binding(6) var<storage, read> meshes: Meshes;
@group(0) @binding(7) var<storage, read> meshlets: Meshlets;

@group(1) @binding(0) var<uniform> materials: Materials;
@group(1) @binding(1) var<uniform> textures: Textures;
@group(1) @binding(2) var<uniform> lights: Lights;
@group(1) @binding(3) var<storage, read> hits: array<RayHit>;
@group(1) @binding(4) var<storage, read_write> shadow_rays: array<ShadowRay>;
@group(1) @binding(5) var<storage, read_write> counters: PathTracingCounters;
@group(1) @binding(6) var<storage, read> rays: array<Ray>;
@group(1) @binding(7) var<storage, read_write> next_rays: array<Ray>;

#import "utils.inc"
#import "matrix_utils.inc"
#import "geom_utils.inc"
#import "texture_utils.inc"
#import "material_utils.inc"
#import "pbr_utils.inc"
#import "light_utils.inc"

const WORKGROUP_SIZE: u32 = 8u;

struct LightSample {
    direction: vec3<f32>,
    distance: f32,
    radiance: vec3<f32>,
    pdf: f32,
}

fn pixarOnb(n: vec3f) -> mat3x3f {
    let s = select(-1f, 1f, n.z >= 0f);
    let a = -1f / (s + n.z);
    let b = n.x * n.y * a;
    let u = vec3(1f + s * n.x * n.x * a, s * b, -s * n.x);
    let v = vec3(b, s + n.y * n.y * a, -n.y);
    return mat3x3(u, v, n);
}

fn sample_cosine_weighted_hemisphere(random_numbers: vec2<f32>, n: vec3<f32>) -> vec3<f32> {
    let phi = 2. * MATH_PI * random_numbers.y;
    let sin_theta = sqrt(1. - random_numbers.x);
    let x = cos(phi) * sin_theta;
    let y = sin(phi) * sin_theta;
    let z = sqrt(random_numbers.x);
    let v = vec3<f32>(x, y, z);
    let onb = pixarOnb(n);
    return normalize(onb * v);
}

fn sample_light_nee(light: ptr<function, LightData>, world_pos: vec3<f32>) -> LightSample {
    var L = vec3<f32>(0.);
    var dist = 10000.0;
    var radiance = vec3<f32>(0.);

    if ((*light).light_type == LIGHT_TYPE_DIRECTIONAL) {
        L = -normalize((*light).direction);
        dist = 10000.0;
        radiance = (*light).color * (*light).intensity;
    } else {
        let to_light = (*light).position - world_pos;
        dist = length(to_light);
        L = normalize(to_light);
        radiance = get_light_intensity(light, to_light);
    }
    return LightSample(L, dist, radiance, 1.0);
}

fn reconstruct_surface(hit: RayHit, hit_point: vec3<f32>) -> PixelData {
    var uv_set: array<vec4<f32>, 4>;
    var normal = vec3<f32>(0.);
    var tangent = vec4<f32>(0.);
    var color = vec4<f32>(1.);

    let instance = instances.data[hit.instance_id];
    let meshlet = meshlets.data[instance.meshlet_id];
    let index_offset = meshlet.indices_offset + (hit.primitive_index * 3u);

    let mesh_id = meshlet.mesh_index;
    let mesh = meshes.data[mesh_id];
    let material_id = u32(mesh.material_index);
    let position_offset = mesh.vertices_position_offset;
    let attributes_offset = mesh.vertices_attribute_offset;
    let vertex_layout = mesh.flags_and_vertices_attribute_layout & 0x0000FFFFu;

    let vertex_attribute_stride = vertex_layout_stride(vertex_layout);
    let offset_color = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_COLOR);
    let offset_normal = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_NORMAL);
    let offset_tangent = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_TANGENT);
    let offset_uv0 = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_UV1);

    let vert_indices = vec3<u32>(indices.data[index_offset], indices.data[index_offset + 1u], indices.data[index_offset + 2u]);
    let attr_indices = vec3<u32>(attributes_offset + vert_indices.x * vertex_attribute_stride,
                                 attributes_offset + vert_indices.y * vertex_attribute_stride,
                                 attributes_offset + vert_indices.z * vertex_attribute_stride);

    let transform = transforms.data[instance.transform_id];
    let orientation = transform.orientation;

    let barycentrics = vec3<f32>(1.0 - hit.barycentrics.x - hit.barycentrics.y, hit.barycentrics.x, hit.barycentrics.y);

    if (offset_normal >= 0) {
        let a1 = vertices_attributes.data[attr_indices.x + u32(offset_normal)];
        let a2 = vertices_attributes.data[attr_indices.y + u32(offset_normal)];
        let a3 = vertices_attributes.data[attr_indices.z + u32(offset_normal)];
        let n1 = unpack_snorm_to_3_f32(a1);
        let n2 = unpack_snorm_to_3_f32(a2);
        let n3 = unpack_snorm_to_3_f32(a3);
        normal = barycentrics.x * n1 + barycentrics.y * n2 + barycentrics.z * n3;
        normal = rotate_vector(normal, orientation);
        normal = normalize(normal);
    } else {
        // Flat shading normal if missing
        // Compute from 3 points?
        // For now assume Z up or something wrong
        normal = vec3<f32>(0., 1., 0.);
    }

    if(offset_uv0 >= 0) {
        let a1 = vertices_attributes.data[attr_indices.x + u32(offset_uv0)];
        let a2 = vertices_attributes.data[attr_indices.y + u32(offset_uv0)];
        let a3 = vertices_attributes.data[attr_indices.z + u32(offset_uv0)];
        let uv1 = unpack2x16float(a1);
        let uv2 = unpack2x16float(a2);
        let uv3 = unpack2x16float(a3);
        let uv = barycentrics.x * uv1 + barycentrics.y * uv2 + barycentrics.z * uv3;
        uv_set[0] = vec4<f32>(uv, 0., 0.);
    }

    return PixelData(hit_point, material_id, color, normal, hit.instance_id, tangent, uv_set);
}

@compute
@workgroup_size(WORKGROUP_SIZE, WORKGROUP_SIZE, 1)
fn main(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let index = global_invocation_id.y * u32(constant_data.screen_width) + global_invocation_id.x;

    // Bounds check
    let ray = rays[index]; // Use Ray from buffer (valid for primary due to RayGen fix)

    // If compaction was used, we would check hit counter. But we use dense dispatch.
    // So we check if ray is valid (t_max > 0) AND if hit is valid.

    let hit = hits[index];
    if (hit.instance_id == 0xFFFFFFFFu) {
        // Miss logic could go here
        return;
    }
    if (ray.t_max <= 0.0) {
        return;
    }

    let hit_point = ray.origin + ray.direction * hit.t;
    var pixel_data = reconstruct_surface(hit, hit_point);
    let material_info = compute_color_from_material(pixel_data.material_id, &pixel_data);

    let V = -ray.direction;
    let N = pixel_data.normal;

    var seed = vec2<u32>(hit.pixel_index, constant_data.frame_index);
    let random = get_random_numbers(&seed);

    // Direct Lighting (NEE)
    if (constant_data.num_lights > 0u) {
        let light_idx = u32(random.x * f32(constant_data.num_lights));
        var light = lights.data[light_idx];
        let light_sample = sample_light_nee(&light, hit_point);

        let L = light_sample.direction;
        let NdotL = max(dot(N, L), 0.0);

        if (NdotL > 0.0 && light_sample.radiance.x > 0.) {
            // Shadow Ray
            var shadow_ray: ShadowRay;
            shadow_ray.origin = hit_point + N * 0.001; // Bias
            shadow_ray.t_max = light_sample.distance - 0.002;
            shadow_ray.direction = L;
            shadow_ray.pixel_index = hit.pixel_index;

            // Simple Lambertian BRDF for now + Specular?
            // Use material_info.base_color or f0
            // Contribution = Throughput * LightRadiance * BRDF * NdotL * LightCount (for PDF)
            // PDF of picking this light = 1 / NumLights.
            // So Weight = NumLights.

            let brdf = material_info.base_color.rgb / MATH_PI; // Lambertian assumption for test

            shadow_ray.contribution = ray.throughput * light_sample.radiance * brdf * NdotL * f32(constant_data.num_lights);

            // Should add Specular contribution here too using material_info

            let shadow_index = atomicAdd(&counters.shadow_ray_count, 1u);
            shadow_rays[shadow_index] = shadow_ray;
        }
    }

    // Indirect Lighting (Extension Ray)
    // Importance sample BSDF.
    // For now: Cosine Weighted Hemisphere (Lambertian).
    let next_dir = sample_cosine_weighted_hemisphere(random, N);
    let NdotL_next = max(dot(N, next_dir), 0.0);

    if (NdotL_next > 0.0) {
        var next_ray: Ray;
        next_ray.origin = hit_point + N * 0.001;
        next_ray.direction = next_dir;
        next_ray.t_min = 0.0;
        next_ray.t_max = 10000.0;
        next_ray.pixel_index = hit.pixel_index;

        // Throughput update
        // Weight = BRDF * NdotL / PDF
        // PDF (cosine weighted) = NdotL / PI.
        // BRDF (Lambertian) = Color / PI.
        // Weight = (Color / PI) * NdotL / (NdotL / PI) = Color.

        next_ray.throughput = ray.throughput * material_info.base_color.rgb;

        // Russian Roulette
        let p = max(next_ray.throughput.x, max(next_ray.throughput.y, next_ray.throughput.z));
        if (random.y < p) {
            next_ray.throughput = next_ray.throughput / p;
            let next_index = atomicAdd(&counters.extension_ray_count, 1u);
            next_rays[next_index] = next_ray;
        }
    }

    // Emissive contribution (Hit Light directly)
    // Add to accumulation.
    // But Accumulation is done via ShadowRays?
    // Emissive should be added directly.
    // Need access to Accumulation Texture here?
    // Or create a "ShadowRay" with 0 distance (Self-Hit) to accumulate emissive?
    // Or a separate "Accumulation" buffer/texture binding.
    // `Shade` pass creates `ShadowRays`.
    // I should add `Accumulation` binding to `Shade` pass to add Emissive.
    // For now, I'll ignore emissive on hit to save time/complexity, or misuse ShadowRay.
    // Misuse ShadowRay: origin=0,0,0, direction=0,0,0, t_max=0. Contribution = Emissive.
    // `Shadow` pass logic: if t_max == 0, assume visible and add?
    // `Shadow` pass logic: if result.distance < shadow_ray.t_max.
    // If t_max = -1, it returns.
    // If t_max = 0.0001 (tiny), it might hit self.

    // Better: Add Emissive logic to ShadowPass? No.
    // I'll skip Emissive for now to ensure Basic Path Tracing works.
}
