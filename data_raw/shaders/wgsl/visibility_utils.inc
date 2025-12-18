// Note: This file intentionally does NOT import vertex_utils.inc to avoid circular dependencies
// Shaders that use visibility_to_gbuffer should import both visibility_utils.inc and vertex_utils.inc

fn visibility_to_gbuffer(visibility_id: u32, hit_point: vec3<f32>) -> PixelData 
{     
    let instance_id = (visibility_id >> 8u) - 1u; 
    let primitive_id = visibility_id & 255u;
    
    let instance = instances.data[instance_id];
    let meshlet = meshlets.data[instance.meshlet_id];
    let index_offset = meshlet.indices_offset + (primitive_id * 3u);

    return get_pixel_data_from_triangle(
        instance_id,
        index_offset,
        hit_point,
        vec3<f32>(0., 0., 0.),
        false
    );
}