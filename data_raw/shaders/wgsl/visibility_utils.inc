// Need constant_data, instances, meshlets, meshes, indices, vertices_positions, vertices_attributes

fn visibility_to_gbuffer(visibility_id: u32, instance_id: u32, hit_point: vec3<f32>) -> PixelData 
{     
    var uv_set: array<vec4<f32>, 4>;
    var normal = vec3<f32>(0.);
    var tangent = vec4<f32>(0.);
    var color = vec4<f32>(1.);

    let meshlet_id = (visibility_id >> 8u) - 1u; 
    let primitive_id = visibility_id & 255u;
    
    let meshlet = &meshlets.data[meshlet_id];
    let index_offset = (*meshlet).indices_offset + (primitive_id * 3u);

    let mesh_id = (*meshlet).mesh_index_and_lod_level >> 3u;
    let mesh = &meshes.data[mesh_id];
    let material_id = u32((*mesh).material_index);
    let position_offset = (*mesh).vertices_position_offset;
    let attributes_offset = (*mesh).vertices_attribute_offset;
    let vertex_layout = (*mesh).flags_and_vertices_attribute_layout & 0x0000FFFFu;

    let instance = &instances.data[instance_id];
    let orientation = (*instance).orientation;
    let position = (*instance).position;
    let scale = (*instance).scale;
    
    let bb_min = transform_vector((*instance).mesh_local_bb_min, position, orientation, scale);
    let bb_max = transform_vector((*instance).mesh_local_bb_max, position, orientation, scale);
    let size = bb_max - bb_min;

    let vertex_attribute_stride = vertex_layout_stride(vertex_layout);   
    let offset_color = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_COLOR);
    let offset_normal = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_NORMAL);
    let offset_tangent = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_TANGENT);
    let offset_uv0 = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_UV1);
    let offset_uv1 = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_UV2);
    let offset_uv2 = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_UV3);
    let offset_uv3 = vertex_attribute_offset(vertex_layout, VERTEX_ATTRIBUTE_HAS_UV4); 

    let vert_indices = vec3<u32>(indices.data[index_offset], indices.data[index_offset + 1u], indices.data[index_offset + 2u]);
    let attr_indices = vec3<u32>(attributes_offset + vert_indices.x * vertex_attribute_stride, 
                                 attributes_offset + vert_indices.y * vertex_attribute_stride,
                                 attributes_offset + vert_indices.z * vertex_attribute_stride);
    
    let v1 = bb_min + unpack_unorm_to_3_f32(vertices_positions.data[vert_indices.x + position_offset]) * size;
    let v2 = bb_min + unpack_unorm_to_3_f32(vertices_positions.data[vert_indices.y + position_offset]) * size;
    let v3 = bb_min + unpack_unorm_to_3_f32(vertices_positions.data[vert_indices.z + position_offset]) * size;
    let p1 = transform_vector(v1, position, orientation, scale);
    let p2 = transform_vector(v2, position, orientation, scale);
    let p3 = transform_vector(v3, position, orientation, scale);
    
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
        normal = normalize(normal);
    }
    if (offset_tangent >= 0) {
        let a1 = vertices_attributes.data[attr_indices.x + u32(offset_tangent)];
        let a2 = vertices_attributes.data[attr_indices.y + u32(offset_tangent)];
        let a3 = vertices_attributes.data[attr_indices.z + u32(offset_tangent)];
        let t1 = unpack_snorm_to_4_f32(a1);
        let t2 = unpack_snorm_to_4_f32(a2);
        let t3 = unpack_snorm_to_4_f32(a3);
        let t = barycentrics.x * t1 + barycentrics.y * t2 + barycentrics.z * t3;
        let rot_t = rotate_vector(t.xyz, orientation); 
        tangent = vec4<f32>(rot_t, t.w);
    } else {
        let s = select(-1., 1., normal.z >= 0.);
        let a = -1. / (s + normal.z);
        let b = normal.x * normal.y * a;
        tangent = vec4<f32>(1. + s * normal.x * normal.x * a, s * b, -s * normal.x, 1.);
    }
    if(offset_uv0 >= 0) {
        let a1 = vertices_attributes.data[attr_indices.x + u32(offset_uv0)];
        let a2 = vertices_attributes.data[attr_indices.y + u32(offset_uv0)];
        let a3 = vertices_attributes.data[attr_indices.z + u32(offset_uv0)];
        let uv1 = unpack2x16float(a1);
        let uv2 = unpack2x16float(a2);
        let uv3 = unpack2x16float(a3);
        let uv = barycentrics.x * uv1 + barycentrics.y * uv2 + barycentrics.z * uv3;
        var uv_dx = uv2 - uv1;
        var uv_dy = uv3 - uv1; 
        uv_dx = select(uv_dx, vec2<f32>(1., 0.), length(uv_dx) <= 0.01);
        uv_dy = select(uv_dy, vec2<f32>(0., 1.), length(uv_dy) <= 0.01);
        uv_set[0] = vec4<f32>(uv, f32(pack2x16float(uv_dx)), f32(pack2x16float(uv_dy)));
    }
    if(offset_uv1 >= 0) {
        let a1 = vertices_attributes.data[attr_indices.x + u32(offset_uv1)];
        let a2 = vertices_attributes.data[attr_indices.y + u32(offset_uv1)];
        let a3 = vertices_attributes.data[attr_indices.z + u32(offset_uv1)];
        let uv1 = unpack2x16float(a1);
        let uv2 = unpack2x16float(a2);
        let uv3 = unpack2x16float(a3);
        let uv = barycentrics.x * uv1 + barycentrics.y * uv2 + barycentrics.z * uv3;
        var uv_dx = uv2 - uv1;
        var uv_dy = uv3 - uv1; 
        uv_dx = select(uv_dx, vec2<f32>(1., 0.), length(uv_dx) <= 0.01);
        uv_dy = select(uv_dy, vec2<f32>(0., 1.), length(uv_dy) <= 0.01);
        uv_set[1] = vec4<f32>(uv, f32(pack2x16float(uv_dx)), f32(pack2x16float(uv_dy)));
    }
    if(offset_uv2 >= 0) {
        let a1 = vertices_attributes.data[attr_indices.x + u32(offset_uv2)];
        let a2 = vertices_attributes.data[attr_indices.y + u32(offset_uv2)];
        let a3 = vertices_attributes.data[attr_indices.z + u32(offset_uv2)];
        let uv1 = unpack2x16float(a1);
        let uv2 = unpack2x16float(a2);
        let uv3 = unpack2x16float(a3);
        let uv = barycentrics.x * uv1 + barycentrics.y * uv2 + barycentrics.z * uv3;
        var uv_dx = uv2 - uv1;
        var uv_dy = uv3 - uv1; 
        uv_dx = select(uv_dx, vec2<f32>(1., 0.), length(uv_dx) <= 0.01);
        uv_dy = select(uv_dy, vec2<f32>(0., 1.), length(uv_dy) <= 0.01);
        uv_set[2] = vec4<f32>(uv, f32(pack2x16float(uv_dx)), f32(pack2x16float(uv_dy)));
    }
    if(offset_uv3 >= 0) {
        let a1 = vertices_attributes.data[attr_indices.x + u32(offset_uv3)];
        let a2 = vertices_attributes.data[attr_indices.y + u32(offset_uv3)];
        let a3 = vertices_attributes.data[attr_indices.z + u32(offset_uv3)];
        let uv1 = unpack2x16float(a1);
        let uv2 = unpack2x16float(a2);
        let uv3 = unpack2x16float(a3);
        let uv = barycentrics.x * uv1 + barycentrics.y * uv2 + barycentrics.z * uv3;
        var uv_dx = uv2 - uv1;
        var uv_dy = uv3 - uv1; 
        uv_dx = select(uv_dx, vec2<f32>(1., 0.), length(uv_dx) <= 0.01);
        uv_dy = select(uv_dy, vec2<f32>(0., 1.), length(uv_dy) <= 0.01);
        uv_set[3] = vec4<f32>(uv, f32(pack2x16float(uv_dx)), f32(pack2x16float(uv_dy)));
    }  
    return PixelData(hit_point, material_id, color, normal, instance_id, tangent, uv_set);
}