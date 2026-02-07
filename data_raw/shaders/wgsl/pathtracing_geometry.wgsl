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

@group(1) @binding(0) var<storage, read> hits: array<RayHit>;
@group(1) @binding(1) var<storage, read_write> surface_data: array<SurfaceData>;
@group(1) @binding(2) var<storage, read_write> counters: PathTracingCounters;
@group(1) @binding(3) var<storage, read> rays: array<Ray>;

#import "utils.inc"
#import "matrix_utils.inc"
#import "geom_utils.inc"

const WORKGROUP_SIZE: u32 = 8u;

fn reconstruct_surface(hit: RayHit, hit_point: vec3<f32>) -> SurfaceData {
    var s: SurfaceData;
    s.position = hit_point;
    s.albedo = vec3<f32>(1.);
    s.normal = vec3<f32>(0., 1., 0.);
    s.uv = vec2<f32>(0.);
    s.roughness = 0.5;
    s.metallic = 0.0;
    s.flags = 0u;
    s.material_index = -1;

    let instance = instances.data[hit.instance_id];
    let meshlet = meshlets.data[instance.meshlet_id];
    let index_offset = meshlet.indices_offset + (hit.primitive_index * 3u);

    let mesh_id = meshlet.mesh_index;
    let mesh = meshes.data[mesh_id];
    s.material_index = mesh.material_index;
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
        var normal = barycentrics.x * n1 + barycentrics.y * n2 + barycentrics.z * n3;
        normal = rotate_vector(normal, orientation);
        s.normal = normalize(normal);
    }

    if (offset_tangent >= 0) {
        let a1 = vertices_attributes.data[attr_indices.x + u32(offset_tangent)];
        let a2 = vertices_attributes.data[attr_indices.y + u32(offset_tangent)];
        let a3 = vertices_attributes.data[attr_indices.z + u32(offset_tangent)];
        let t1 = unpack_snorm_to_4_f32(a1);
        let t2 = unpack_snorm_to_4_f32(a2);
        let t3 = unpack_snorm_to_4_f32(a3);
        var tangent_xyz = barycentrics.x * t1.xyz + barycentrics.y * t2.xyz + barycentrics.z * t3.xyz;
        tangent_xyz = rotate_vector(tangent_xyz, orientation);
        s.tangent = vec4<f32>(normalize(tangent_xyz), t1.w);
    } else {
        s.tangent = vec4<f32>(1., 0., 0., 1.);
    }

    if(offset_uv0 >= 0) {
        let a1 = vertices_attributes.data[attr_indices.x + u32(offset_uv0)];
        let a2 = vertices_attributes.data[attr_indices.y + u32(offset_uv0)];
        let a3 = vertices_attributes.data[attr_indices.z + u32(offset_uv0)];
        let uv1 = unpack2x16float(a1);
        let uv2 = unpack2x16float(a2);
        let uv3 = unpack2x16float(a3);
        s.uv = barycentrics.x * uv1 + barycentrics.y * uv2 + barycentrics.z * uv3;
    }

    return s;
}

@compute
@workgroup_size(WORKGROUP_SIZE, WORKGROUP_SIZE, 1)
fn main(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
) {
    let index = global_invocation_id.y * u32(constant_data.screen_width) + global_invocation_id.x;

    let hit = hits[index];
    if (hit.instance_id == 0xFFFFFFFFu) {
        return;
    }

    let ray = rays[index];
    let hit_point = ray.origin + ray.direction * hit.t;

    surface_data[index] = reconstruct_surface(hit, hit_point);
}
