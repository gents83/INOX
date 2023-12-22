// Need constant_data, meshlets, meshes, indices, runtime_vertices, vertices_attributes

fn visibility_to_gbuffer(visibility_id: u32, clip_coords: vec2<f32>, mvp: mat4x4<f32>) -> PixelData 
{     
    var uv_set: array<vec2<f32>, 4>;
    var normal = vec3<f32>(0.);
    var color = vec4<f32>(1.);

    let meshlet_id = (visibility_id >> 8u) - 1u; 
    let primitive_id = visibility_id & 255u;
    
    let meshlet = &meshlets.data[meshlet_id];
    let index_offset = (*meshlet).indices_offset + (primitive_id * 3u);

    let mesh_id = u32((*meshlet).mesh_index);
    let mesh = &meshes.data[mesh_id];
    let material_id = u32((*mesh).material_index);
    let position_offset = (*mesh).vertices_position_offset;
    let attributes_offset = (*mesh).vertices_attribute_offset;
    let vertex_layout = (*mesh).flags_and_vertices_attribute_layout;
    let orientation = (*mesh).orientation;
    let vertex_attribute_stride = vertex_layout_stride(vertex_layout);   
    let offset_color = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_COLOR);
    let offset_normal = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_NORMAL);
    let offset_uv0 = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_UV1);
    let offset_uv1 = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_UV2);
    let offset_uv2 = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_UV3);
    let offset_uv3 = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_UV4); 

    let vert_indices = vec3<u32>(indices.data[index_offset], indices.data[index_offset + 1u], indices.data[index_offset + 2u]);
    let attr_indices = vec3<u32>(attributes_offset + vert_indices.x * vertex_attribute_stride, 
                                 attributes_offset + vert_indices.y * vertex_attribute_stride,
                                 attributes_offset + vert_indices.z * vertex_attribute_stride);
    
    let v1 = runtime_vertices.data[vert_indices.x + position_offset].world_pos;
    let v2 = runtime_vertices.data[vert_indices.y + position_offset].world_pos;
    let v3 = runtime_vertices.data[vert_indices.z + position_offset].world_pos;
    
    var p1 = mvp * vec4<f32>(v1, 1.);
    var p2 = mvp * vec4<f32>(v2, 1.);
    var p3 = mvp * vec4<f32>(v3, 1.);

    // Calculate the inverse of w, since it's going to be used several times
    let one_over_w = 1. / vec3<f32>(p1.w, p2.w, p3.w);
    p1 = (p1 * one_over_w.x + 1.) * 0.5;
    p2 = (p2 * one_over_w.y + 1.) * 0.5;
    p3 = (p3 * one_over_w.z + 1.) * 0.5;
    
    // Get delta vector that describes current screen point relative to vertex 0
    var screen_pixel = clip_coords.xy;
    screen_pixel.y = 1. - screen_pixel.y;
    let delta = screen_pixel + -p1.xy;
    let barycentrics = compute_barycentrics(p1.xy, p2.xy, p3.xy, screen_pixel);
    let deriv = compute_partial_derivatives(p1.xy, p2.xy, p3.xy);   

    let world_pos = barycentrics.x * v1 + barycentrics.y * v2 + barycentrics.z * v3; 
    let depth = barycentrics.x * p1.z + barycentrics.y * p2.z + barycentrics.z * p3.z;  

    if (offset_color >= 0) {
        let a1 = vertices_attributes.data[attr_indices.x + u32(offset_color)];
        let a2 = vertices_attributes.data[attr_indices.y + u32(offset_color)];
        let a3 = vertices_attributes.data[attr_indices.z + u32(offset_color)];
        let c1 = unpack_unorm_to_4_f32(a1);
        let c2 = unpack_unorm_to_4_f32(a2);
        let c3 = unpack_unorm_to_4_f32(a3);
        color *= barycentrics.x * c1 + barycentrics.y * c2 + barycentrics.z * c3;    
    }
    if (offset_normal >= 0) {
        let a1 = vertices_attributes.data[attr_indices.x + u32(offset_normal)];
        let a2 = vertices_attributes.data[attr_indices.y + u32(offset_normal)];
        let a3 = vertices_attributes.data[attr_indices.z + u32(offset_normal)];
        let n1 = unpack_snorm_to_3_f32(a1);
        let n2 = unpack_snorm_to_3_f32(a2);
        let n3 = unpack_snorm_to_3_f32(a3);
        normal = barycentrics.x * n1 + barycentrics.y * n2 + barycentrics.z * n3;
        normal = rotate_vector(normal, orientation); 
    }
    if(offset_uv0 >= 0) {
        let a1 = vertices_attributes.data[attr_indices.x + u32(offset_uv0)];
        let a2 = vertices_attributes.data[attr_indices.y + u32(offset_uv0)];
        let a3 = vertices_attributes.data[attr_indices.z + u32(offset_uv0)];
        let uv1 = unpack2x16float(a1);
        let uv2 = unpack2x16float(a2);
        let uv3 = unpack2x16float(a3);
        uv_set[0] = interpolate_2d_attribute(uv1, uv2, uv3, deriv, delta);
    }
    if(offset_uv1 >= 0) {
        let a1 = vertices_attributes.data[attr_indices.x + u32(offset_uv1)];
        let a2 = vertices_attributes.data[attr_indices.y + u32(offset_uv1)];
        let a3 = vertices_attributes.data[attr_indices.z + u32(offset_uv1)];
        let uv1 = unpack2x16float(a1);
        let uv2 = unpack2x16float(a2);
        let uv3 = unpack2x16float(a3);
        uv_set[1] = interpolate_2d_attribute(uv1, uv2, uv3, deriv, delta);
    }
    if(offset_uv2 >= 0) {
        let a1 = vertices_attributes.data[attr_indices.x + u32(offset_uv2)];
        let a2 = vertices_attributes.data[attr_indices.y + u32(offset_uv2)];
        let a3 = vertices_attributes.data[attr_indices.z + u32(offset_uv2)];
        let uv1 = unpack2x16float(a1);
        let uv2 = unpack2x16float(a2);
        let uv3 = unpack2x16float(a3);
        uv_set[2] = interpolate_2d_attribute(uv1, uv2, uv3, deriv, delta);
    }
    if(offset_uv3 >= 0) {
        let a1 = vertices_attributes.data[attr_indices.x + u32(offset_uv3)];
        let a2 = vertices_attributes.data[attr_indices.y + u32(offset_uv3)];
        let a3 = vertices_attributes.data[attr_indices.z + u32(offset_uv3)];
        let uv1 = unpack2x16float(a1);
        let uv2 = unpack2x16float(a2);
        let uv3 = unpack2x16float(a3);
        uv_set[3] = interpolate_2d_attribute(uv1, uv2, uv3, deriv, delta);
    }    

    return PixelData(world_pos, depth, normal, material_id, color, uv_set);
}