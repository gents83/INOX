const HIT_DATA_TLAS = 0u;
const HIT_DATA_BLAS = 1u;
const HIT_DATA_TRIANGLE = 2u;
const HIT_DATA_MISS = 4u;

struct Result {
    distance: f32,
    visibility_id: u32,
}

fn intersect_aabb(ray: ptr<function, Ray>, aabb_min: vec3<f32>, aabb_max: vec3<f32>) -> f32 
{     
    let r_min = (*ray).t_min;
    let r_max = (*ray).t_max;
    let inverse_dir = 1. / (*ray).direction;
    let v_min = (aabb_min - (*ray).origin) * inverse_dir;
    let v_max = (aabb_max - (*ray).origin) * inverse_dir;

    let b_min = min(v_min, v_max);
    let b_max = max(v_min, v_max);

    let t_near = max(max(b_min.x, b_min.y), max(b_min.x, b_min.z));
    let t_far = min(min(b_max.x, b_max.y), min(b_max.x, b_max.z));
    
    let r = select(t_near, r_max, t_near > t_far || t_far < 0.);
    return r; 
}

fn intersect_triangle(ray: Ray, v0: vec3<f32>, v1: vec3<f32>, v2: vec3<f32>) -> f32
{
    let e1 = v1 - v0;
    let e2 = v2 - v0;

    let h = cross(ray.direction, e2);
    let a = dot(e1, h);    
    let inv_a = 1.0 / a;    
    let s = ray.origin - v0;    
    let u = inv_a * dot(s, h);    
    let mask_u = u >= 0.0 && u <= 1.0;
    let q = cross(s, e1);    
    let v = inv_a * dot(ray.direction, q);    
    let mask_v = v >= 0.0 && u + v <= 1.0;
    let t = inv_a * dot(e2, q);    
    let mask_t = t > 0.0 && t < ray.t_max;

    return select(ray.t_max, t, (mask_u && mask_v && mask_t));
}

fn intersect_meshlet_primitive(ray: Ray, position_offset: u32, meshlet_id: u32, index_offset: u32) -> f32 {
    let v1 = runtime_vertices.data[position_offset + indices.data[index_offset]].world_pos;
    let v2 = runtime_vertices.data[position_offset + indices.data[index_offset + 1u]].world_pos;
    let v3 = runtime_vertices.data[position_offset + indices.data[index_offset + 2u]].world_pos;
        
    return intersect_triangle(ray, v1, v2, v3);
}

fn intersect_meshlet(ray: Ray, position_offset: u32, meshlet_id: u32, far_plane: f32) -> Result {
    var nearest = far_plane;  
    var visibility_id = 0u;
      
    let meshlet = &meshlets.data[meshlet_id];
    let index_count = (*meshlet).indices_count;
    let index_offset = (*meshlet).indices_offset;
    for(var index = 0u; index < index_count; index = index + 3u)
    {       
        let hit = intersect_meshlet_primitive(ray, position_offset, meshlet_id, index_offset + index);
        visibility_id = select(visibility_id, ((meshlet_id + 1u) << 8u) | (index / 3u), hit < nearest);
        nearest = min(nearest, hit);
    }
    return Result(nearest, visibility_id);
}

fn traverse_bhv_of_meshlets(world_ray: ptr<function, Ray>, local_ray: ptr<function, Ray>, mesh_id: u32, far_plane: f32) -> Result {
    let mesh = &meshes.data[mesh_id];    
    let position_offset = (*mesh).vertices_position_offset;
    var blas_index = i32((*mesh).blas_index);    
    let mesh_blas_index = blas_index;
    var nearest = far_plane;  
    var visibility_id = 0u;

    while (blas_index >= 0)
    { 
        let node = &bhv.data[u32(blas_index)];   
        let intersection = intersect_aabb(local_ray, (*node).min, (*node).max);
        if (intersection > nearest) {
            blas_index = select((*node).miss, (*node).miss + mesh_blas_index, (*node).miss >= 0);
            continue;
        }
        if ((*node).reference < 0) {
            //inner node
            blas_index = blas_index + 1;
            continue;  
        }
        //leaf node
        let meshlet_id = (*mesh).meshlets_offset + u32((*node).reference);   
        
        let index = meshlet_id / 32u;
        let offset = meshlet_id - (index * 32u);
        let bits = culling_result[index];
        let is_meshlet_visible =  (bits & (1u << offset)) > 0u;   

        if (!is_meshlet_visible) {
            blas_index = select((*node).miss, (*node).miss + mesh_blas_index, (*node).miss >= 0);
            continue;
        }
        let hit = intersect_meshlet((*world_ray), position_offset, meshlet_id, nearest);
        visibility_id = select(visibility_id, hit.visibility_id, hit.distance < nearest);
        nearest = min(nearest, hit.distance);
        blas_index = select((*node).miss, (*node).miss + mesh_blas_index, (*node).miss >= 0);
    }
    return Result(nearest, visibility_id);
}


//starting index = tlas_starting_index
fn traverse_bvh(r: Ray, tlas_starting_index: i32) -> Result {
    var world_ray = r;
    var ray = world_ray;
    var node_index = tlas_starting_index;
    var visibility_id = 0u;
    var hit_type = HIT_DATA_TLAS;
    var meshlets_offset = 0u;
    var meshlet_indices_offset = 0u;
    var vertices_position_offset = 0u;
    var meshlet_id = 0u;
    var blas_sibling = 0;
    var tlas_sibling = 0;
    var hit_distance = MAX_FLOAT;
    
    while(hit_type != HIT_DATA_MISS)
    {
        if(node_index < 0) {
            let is_tlas = hit_type == HIT_DATA_TLAS;
            let is_triangle = hit_type == HIT_DATA_TRIANGLE;
            node_index = select(tlas_sibling, blas_sibling, is_triangle);
            hit_type = select(HIT_DATA_TLAS, HIT_DATA_BLAS, is_triangle);
            hit_type = select(hit_type, HIT_DATA_MISS, is_tlas);
            ray.origin = select(world_ray.origin, ray.origin, is_triangle);
            ray.direction = select(world_ray.direction, ray.direction, is_triangle);
            continue;
        }
        
        let node = bhv.data[u32(node_index)]; 
        let intersection = intersect_aabb(&ray, node.min, node.max);        
        let is_miss = intersection > ray.t_max;
        let is_inner_node = node.reference < 0;
        node_index = select(node_index, node_index + 1, is_inner_node);
        node_index = select(node_index, node.miss, is_miss);
        if (is_miss || is_inner_node) {
            continue;
        }
        //leaf node
        if(hit_type == HIT_DATA_TRIANGLE) {     
            node_index = node.miss;        
            
            let triangle_id = u32(node.reference);
            let index_offset = meshlet_indices_offset + (triangle_id * 3u);
            let v1 = runtime_vertices.data[vertices_position_offset + indices.data[index_offset]].world_pos;
            let v2 = runtime_vertices.data[vertices_position_offset + indices.data[index_offset + 1u]].world_pos;
            let v3 = runtime_vertices.data[vertices_position_offset + indices.data[index_offset + 2u]].world_pos;
            let distance = intersect_triangle(world_ray, v1, v2, v3);
            visibility_id = select(visibility_id, ((meshlet_id + 1u) << 8u) | triangle_id, distance < hit_distance);
            hit_distance = min(hit_distance, distance);
        }
        else if(hit_type == HIT_DATA_BLAS) {  
            hit_type = HIT_DATA_TRIANGLE;
            blas_sibling = node.miss;
            meshlet_id = meshlets_offset + u32(node.reference);  
            let index = meshlet_id / 32u;
            let offset = meshlet_id - (index * 32u);
            let is_meshlet_visible =  (culling_result[index] & (1u << offset)) != 0u;   
            let meshlet = meshlets.data[meshlet_id];
            meshlet_indices_offset = meshlet.indices_offset;
            node_index = select(blas_sibling, i32(meshlet.triangles_bhv_index), is_meshlet_visible); 
        }
        else {  
            hit_type = HIT_DATA_BLAS;
            tlas_sibling = node.miss;
            let mesh_id = u32(node.reference);
            let mesh = meshes.data[mesh_id];  
            meshlets_offset = mesh.meshlets_offset;
            vertices_position_offset = mesh.vertices_position_offset;
            node_index = i32(mesh.blas_index); 

            let matrix = transform_matrix(mesh.position, mesh.orientation, mesh.scale);    
            let inverse_matrix = matrix_inverse(matrix);
            let local_ray_origin = inverse_matrix * vec4<f32>(world_ray.origin, 1.);
            let local_ray_direction = inverse_matrix * vec4<f32>(world_ray.direction, 0.);
            ray.origin = local_ray_origin.xyz;
            ray.direction = local_ray_direction.xyz;
        }
    }
    return Result(hit_distance, visibility_id);
}