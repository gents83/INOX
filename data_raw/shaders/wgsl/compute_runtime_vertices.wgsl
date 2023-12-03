#import "common.inc"
#import "utils.inc"

@group(0) @binding(0)
var<uniform> constant_data: ConstantData;
@group(0) @binding(1)
var<storage, read> bhv: BHV;
@group(0) @binding(2)
var<storage, read> meshes: Meshes;
@group(0) @binding(3)
var<storage, read> vertices_positions: VerticesPositions;
@group(0) @binding(4)
var<storage, read_write> runtime_vertices: RuntimeVertices;

#import "matrix_utils.inc"


@compute
@workgroup_size(256, 1, 1)
fn main(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>, 
) {
    let total = arrayLength(&runtime_vertices.data);
    let vertex_id = global_invocation_id.x;
    if (vertex_id >= total) {
        return;
    }
    
    let mesh_id = runtime_vertices.data[vertex_id].mesh_index;
    let mesh = &meshes.data[mesh_id];
    let bhv_id = (*mesh).blas_index;
    let bhv_node = &bhv.data[bhv_id];
    let size = (*bhv_node).max - (*bhv_node).min;
    let p = (*bhv_node).min + unpack_unorm_to_3_f32(vertices_positions.data[vertex_id]) * size;

    runtime_vertices.data[vertex_id].world_pos = transform_vector(p, (*mesh).position, (*mesh).orientation, (*mesh).scale);
}