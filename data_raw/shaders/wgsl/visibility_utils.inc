// Need constant_data, meshlets, meshes, indices, runtime_vertices, vertices_attributes

fn visibility_to_gbuffer(visibility_id: u32, hit_point: vec3<f32>) -> PixelData 
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
    let scale = (*mesh).scale;
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
    
    let p1 = runtime_vertices.data[vert_indices.x + position_offset].world_pos;
    let p2 = runtime_vertices.data[vert_indices.y + position_offset].world_pos;
    let p3 = runtime_vertices.data[vert_indices.z + position_offset].world_pos;
    
    let barycentrics = compute_barycentrics_3d(p1,p2,p3,hit_point); 

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
        uv_set[0] = barycentrics.x * uv1 + barycentrics.y * uv2 + barycentrics.z * uv3;
    }
    if(offset_uv1 >= 0) {
        let a1 = vertices_attributes.data[attr_indices.x + u32(offset_uv1)];
        let a2 = vertices_attributes.data[attr_indices.y + u32(offset_uv1)];
        let a3 = vertices_attributes.data[attr_indices.z + u32(offset_uv1)];
        let uv1 = unpack2x16float(a1);
        let uv2 = unpack2x16float(a2);
        let uv3 = unpack2x16float(a3);
        uv_set[1] = barycentrics.x * uv1 + barycentrics.y * uv2 + barycentrics.z * uv3;
    }
    if(offset_uv2 >= 0) {
        let a1 = vertices_attributes.data[attr_indices.x + u32(offset_uv2)];
        let a2 = vertices_attributes.data[attr_indices.y + u32(offset_uv2)];
        let a3 = vertices_attributes.data[attr_indices.z + u32(offset_uv2)];
        let uv1 = unpack2x16float(a1);
        let uv2 = unpack2x16float(a2);
        let uv3 = unpack2x16float(a3);
        uv_set[2] = barycentrics.x * uv1 + barycentrics.y * uv2 + barycentrics.z * uv3;
    }
    if(offset_uv3 >= 0) {
        let a1 = vertices_attributes.data[attr_indices.x + u32(offset_uv3)];
        let a2 = vertices_attributes.data[attr_indices.y + u32(offset_uv3)];
        let a3 = vertices_attributes.data[attr_indices.z + u32(offset_uv3)];
        let uv1 = unpack2x16float(a1);
        let uv2 = unpack2x16float(a2);
        let uv3 = unpack2x16float(a3);
        uv_set[3] = barycentrics.x * uv1 + barycentrics.y * uv2 + barycentrics.z * uv3;
    }    

    return PixelData(hit_point, material_id, color, normal, scale.x, uv_set);
}