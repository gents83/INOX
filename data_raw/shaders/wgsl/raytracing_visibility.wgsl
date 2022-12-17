#import "utils.inc"
#import "common.inc"

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
var<storage, read> vertices: Vertices;
@group(0) @binding(3)
var<storage, read> positions: Positions;
@group(0) @binding(4)
var<storage, read> meshes: Meshes;
@group(0) @binding(5)
var<storage, read> meshlets: Meshlets;
@group(0) @binding(6)
var<storage, read> bhv: BHV;

@group(1) @binding(0)
var render_target: texture_storage_2d<rgba8unorm, read_write>;

#import "matrix_utils.inc"
#import "raytracing.inc"

@compute
@workgroup_size(16, 16, 1)
fn main(
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>, 
    @builtin(local_invocation_index) local_invocation_index: u32, 
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    let dimensions = vec2<u32>(textureDimensions(render_target));
         
    let pixel = vec2<u32>(global_invocation_id.x, global_invocation_id.y);
    if (pixel.x >= dimensions.x || pixel.y >= dimensions.y)
    {
        return;
    }    
    // Create a ray with the current fragment as the origin.
    let ray = compute_ray(pixel, dimensions);
                    
    var nearest = MAX_FLOAT;
    var visibility_id = 0u;

    let mesh_count = 10u;//arrayLength(&meshes.data);    
    for (var mesh_id = 0u; mesh_id < mesh_count; mesh_id++) {
        let mesh = &meshes.data[mesh_id];
        let mesh_aabb = &bhv.data[(*mesh).bhv_index];        
        let mesh_oobb_min = vec4<f32>(transform_vector((*mesh_aabb).min, (*mesh).position, (*mesh).orientation, (*mesh).scale), 1.);
        let mesh_oobb_max = vec4<f32>(transform_vector((*mesh_aabb).max, (*mesh).position, (*mesh).orientation, (*mesh).scale), 1.);
        let mesh_intersection = intersect_oobb(ray, mesh_oobb_min.xyz, mesh_oobb_max.xyz);
        if (mesh_intersection < MAX_FLOAT && mesh_intersection < nearest)
        {            
            let mesh_aabb_size = abs((*mesh_aabb).max - (*mesh_aabb).min);
            for (var meshlet_index = 0u; meshlet_index < (*mesh).meshlets_count; meshlet_index++) {
                let meshlet_id = (*mesh).meshlets_offset + meshlet_index;
                let meshlet = &meshlets.data[meshlet_id];
                let meshlet_aabb = &bhv.data[(*meshlet).bhv_index];          
                let meshlet_oobb_min = vec4<f32>(transform_vector((*meshlet_aabb).min, (*mesh).position, (*mesh).orientation, (*mesh).scale), 1.);
                let meshlet_oobb_max = vec4<f32>(transform_vector((*meshlet_aabb).max, (*mesh).position, (*mesh).orientation, (*mesh).scale), 1.);
                let meshlet_intersection = intersect_oobb(ray, meshlet_oobb_min.xyz, meshlet_oobb_max.xyz);
                if (meshlet_intersection < MAX_FLOAT && meshlet_intersection < nearest)
                {
                    let triangle_count = ((*meshlet).indices_count - (*meshlet).indices_offset) / 3u;            
                    for (var primitive_id = 0u; primitive_id < triangle_count; primitive_id++) {
                        let index_offset = (*mesh).indices_offset + (*meshlet).indices_offset + primitive_id * 3u;
                        let i1 = indices.data[index_offset];
                        let i2 = indices.data[index_offset + 1u];
                        let i3 = indices.data[index_offset + 2u];

                        let v1 = &vertices.data[(*mesh).vertex_offset + i1];
                        let v2 = &vertices.data[(*mesh).vertex_offset + i2];
                        let v3 = &vertices.data[(*mesh).vertex_offset + i3];
                        
                        let vp1 = (*mesh_aabb).min + decode_as_vec3(positions.data[(*v1).position_and_color_offset]) * mesh_aabb_size;
                        let vp2 = (*mesh_aabb).min + decode_as_vec3(positions.data[(*v2).position_and_color_offset]) * mesh_aabb_size;
                        let vp3 = (*mesh_aabb).min + decode_as_vec3(positions.data[(*v3).position_and_color_offset]) * mesh_aabb_size;

                        var p1 = vec4<f32>(transform_vector(vp1, (*mesh).position, (*mesh).orientation, (*mesh).scale), 1.);
                        var p2 = vec4<f32>(transform_vector(vp2, (*mesh).position, (*mesh).orientation, (*mesh).scale), 1.);
                        var p3 = vec4<f32>(transform_vector(vp3, (*mesh).position, (*mesh).orientation, (*mesh).scale), 1.);

                        let hit_distance = intersect_triangle(ray, p1.xyz, p2.xyz, p3.xyz);
                        if (hit_distance < MAX_FLOAT && hit_distance < nearest) {
                            visibility_id = 0xFFFFFFFFu;//((meshlet_id + 1u) << 8u) + primitive_id;
                            nearest = hit_distance;
                        }
                        //Intersection found
                        //if (intersection.t >= 0. && intersection.t < nearest.t) {
                        //    nearest.visibility_id = ((meshlet_id + 1u) << 8u) + primitive_id;
                        //    nearest.t = intersection.t;
                        //}
                    }
                }
            }
        }
    }    

    textureStore(render_target, vec2<i32>(pixel), unpack4x8unorm(visibility_id));
}