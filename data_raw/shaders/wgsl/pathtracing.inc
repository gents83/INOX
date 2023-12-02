const MAX_PATH_BOUNCES: u32 = 8u;
const NUM_SAMPLES_PER_PIXEL: u32 = 4u;


struct RadianceData {
    radiance: vec3<f32>, 
    throughput_weight: vec3<f32>, 
    seed: vec2<u32>,
    ray: Ray,
}

fn sample_sphere(random_numbers: vec2<f32>) -> vec3<f32> {
    let z = 2. * random_numbers.y - 1.;
    let phi = 2. * PI * random_numbers.x;
    let s = sqrt(1. - z * z);
    let x = cos(phi) * s;
    let y = sin(phi) * s;
    return vec3<f32>(x, y, z);
}

fn sample_hemisphere(random_numbers: vec2<f32>, normal: vec3<f32>) -> vec3<f32> {
    var direction = sample_sphere(random_numbers);
    let d = dot(normal, direction);
    if (d < 0.) {
        direction -= 2. * d * normal;
    }
    return direction;
}

// Need constant_data, meshlets, meshes, indices, runtime_vertices, vertices_attributes
fn compute_radiance_from_visibility(visibility_id: u32, screen_uv: vec2<f32>, data: RadianceData) -> RadianceData { 
    let meshlet_id = (visibility_id >> 8u) - 1u; 
    let primitive_id = visibility_id & 255u;
    
    let meshlet = &meshlets.data[meshlet_id];
    let index_offset = (*meshlet).indices_offset + (primitive_id * 3u);

    let mesh_id = u32((*meshlet).mesh_index);
    let mesh = &meshes.data[mesh_id];
    let position_offset = (*mesh).vertices_position_offset;
    let attributes_offset = (*mesh).vertices_attribute_offset;
    let vertex_layout = (*mesh).vertices_attribute_layout;
    let vertex_attribute_stride = vertex_layout_stride(vertex_layout);   
    let offset_color = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_COLOR);
    let offset_normal = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_NORMAL);
        
    var normal = vec3<f32>(0.);
    var emissive_color = vec3<f32>(0.25);
    var color = vec4<f32>(1.);

    let vert_indices = vec3<u32>(indices.data[index_offset], indices.data[index_offset + 1u], indices.data[index_offset + 2u]);
    let pos_indices = vert_indices + vec3<u32>(position_offset, position_offset, position_offset);
    let attr_indices = vec3<u32>(attributes_offset + vert_indices.x * vertex_attribute_stride, 
                                 attributes_offset + vert_indices.y * vertex_attribute_stride,
                                 attributes_offset + vert_indices.z * vertex_attribute_stride);
    
    let v1 = runtime_vertices.data[pos_indices.x].world_pos;
    let v2 = runtime_vertices.data[pos_indices.y].world_pos;
    let v3 = runtime_vertices.data[pos_indices.z].world_pos;
    let mvp = constant_data.proj * constant_data.view;
    var p1 = mvp * vec4<f32>(v1, 1.);
    var p2 = mvp * vec4<f32>(v2, 1.);
    var p3 = mvp * vec4<f32>(v3, 1.);

    // Calculate the inverse of w, since it's going to be used several times
    let one_over_w = 1. / vec3<f32>(p1.w, p2.w, p3.w);
    p1 = (p1 * one_over_w.x + 1.) * 0.5;
    p2 = (p2 * one_over_w.y + 1.) * 0.5;
    p3 = (p3 * one_over_w.z + 1.) * 0.5;
    
    // Get delta vector that describes current screen point relative to vertex 0
    var screen_pixel = screen_uv.xy;
    screen_pixel.y = 1. - screen_pixel.y;
    let barycentrics = compute_barycentrics(p1.xy, p2.xy, p3.xy, screen_pixel);
    
    if (offset_color >= 0) {
        let a1 = vertices_attributes.data[attr_indices.x + u32(offset_color)];
        let a2 = vertices_attributes.data[attr_indices.y + u32(offset_color)];
        let a3 = vertices_attributes.data[attr_indices.z + u32(offset_color)];
        let c1 = unpack_unorm_to_4_f32(a1);
        let c2 = unpack_unorm_to_4_f32(a2);
        let c3 = unpack_unorm_to_4_f32(a3);
        color = barycentrics.x * c1 + barycentrics.y * c2 + barycentrics.z * c3;    
    }
    if (offset_normal >= 0) {
        let a1 = vertices_attributes.data[attr_indices.x + u32(offset_normal)];
        let a2 = vertices_attributes.data[attr_indices.y + u32(offset_normal)];
        let a3 = vertices_attributes.data[attr_indices.z + u32(offset_normal)];
        let n1 = decode_as_vec3(a1);
        let n2 = decode_as_vec3(a2);
        let n3 = decode_as_vec3(a3);
        normal = barycentrics.x * n1 + barycentrics.y * n2 + barycentrics.z * n3;
        normal = rotate_vector(normal, (*mesh).orientation); 
    }

    let seed = get_random_numbers(data.seed);    
    let rnd = vec2<f32>(seed) * 2.32830643654e-10;
    let direction = sample_hemisphere(rnd, normal);
    let d = dot(normal, direction);

    let new_ray = Ray(data.ray.origin + (data.ray.direction * data.ray.t_max) + (normal * HIT_EPSILON), 0., direction, MAX_FLOAT);
    let rad = data.radiance + data.throughput_weight * emissive_color;
    let weight = data.throughput_weight * (color.rgb * 2. * d);
    return RadianceData(rad, weight, seed, new_ray);
}