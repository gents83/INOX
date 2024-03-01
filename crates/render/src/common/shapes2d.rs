use std::f32::consts::PI;

use inox_math::{VecBase, VecBaseFloat, Vector2, Vector3, Vector4};

use crate::{MeshData, MeshletData, VertexAttributeLayout};

pub fn create_quad(rect: Vector4, z: f32) -> MeshData {
    create_quad_with_texture(rect, z, [0., 0., 1., 1.].into())
}

pub fn create_quad_with_texture(rect: Vector4, z: f32, tex_coords: Vector4) -> MeshData {
    let mut mesh_data = MeshData {
        vertex_layout: VertexAttributeLayout::pos_color_normal_uv1(),
        aabb_min: Vector3::new(rect.x, rect.y, z),
        aabb_max: Vector3::new(rect.z, rect.w, z),
        ..Default::default()
    };
    mesh_data.add_vertex_pos_color_normal_uv(
        [rect.x, rect.y, z].into(),
        Vector4::default_one(),
        [-1., -1., 0.].into(),
        [tex_coords.x, tex_coords.y].into(),
    );
    mesh_data.add_vertex_pos_color_normal_uv(
        [rect.x, rect.w, z].into(),
        Vector4::default_one(),
        [-1., 1., 0.].into(),
        [tex_coords.x, tex_coords.w].into(),
    );
    mesh_data.add_vertex_pos_color_normal_uv(
        [rect.z, rect.w, z].into(),
        Vector4::default_one(),
        [1., 1., 0.].into(),
        [tex_coords.z, tex_coords.w].into(),
    );
    mesh_data.add_vertex_pos_color_normal_uv(
        [rect.z, rect.y, z].into(),
        Vector4::default_one(),
        [1., -1., 0.].into(),
        [tex_coords.z, tex_coords.y].into(),
    );
    mesh_data.indices = [0, 3, 2, 2, 1, 0].to_vec();

    let meshlet = MeshletData {
        indices_count: mesh_data.index_count() as _,
        aabb_min: mesh_data.aabb_min(),
        aabb_max: mesh_data.aabb_max(),
        ..Default::default()
    };
    mesh_data.meshlets[0].push(meshlet);
    mesh_data
}
pub fn create_colored_quad(rect: Vector4, z: f32, color: Vector4) -> MeshData {
    let mut mesh_data = MeshData {
        vertex_layout: VertexAttributeLayout::pos_color(),
        aabb_min: Vector3::new(rect.x, rect.y, z),
        aabb_max: Vector3::new(rect.z, rect.w, z),
        ..Default::default()
    };
    mesh_data.add_vertex_pos_color([rect.x, rect.y, z].into(), color);
    mesh_data.add_vertex_pos_color([rect.x, rect.w, z].into(), color);
    mesh_data.add_vertex_pos_color([rect.z, rect.w, z].into(), color);
    mesh_data.add_vertex_pos_color([rect.z, rect.y, z].into(), color);
    mesh_data.indices = [0, 2, 1, 3, 2, 0].to_vec();

    let meshlet = MeshletData {
        indices_count: mesh_data.index_count() as _,
        aabb_min: mesh_data.aabb_min(),
        aabb_max: mesh_data.aabb_max(),
        ..Default::default()
    };
    mesh_data.meshlets[0].push(meshlet);
    mesh_data
}

pub fn create_triangle_up() -> MeshData {
    let mut mesh_data = MeshData {
        vertex_layout: VertexAttributeLayout::pos_color_normal_uv1(),
        aabb_min: Vector3::new(0., 0., 0.),
        aabb_max: Vector3::new(1., 1., 0.),
        ..Default::default()
    };
    mesh_data.add_vertex_pos_color_normal_uv(
        [0., 1., 0.].into(),
        Vector4::default_one(),
        [-1., -1., 0.].into(),
        [0., 1.].into(),
    );
    mesh_data.add_vertex_pos_color_normal_uv(
        [1., 1., 0.].into(),
        Vector4::default_one(),
        [1., -1., 0.].into(),
        [1., 1.].into(),
    );
    mesh_data.add_vertex_pos_color_normal_uv(
        [0.5, 0., 0.].into(),
        Vector4::default_one(),
        [0., 1., 0.].into(),
        [0.5, 0.].into(),
    );

    mesh_data.indices = [0, 2, 1].to_vec();

    let meshlet = MeshletData {
        indices_count: mesh_data.index_count() as _,
        aabb_min: mesh_data.aabb_min(),
        aabb_max: mesh_data.aabb_max(),
        ..Default::default()
    };
    mesh_data.meshlets[0].push(meshlet);
    mesh_data
}

pub fn create_triangle_down() -> MeshData {
    let mut mesh_data = MeshData {
        vertex_layout: VertexAttributeLayout::pos_color_normal_uv1(),
        aabb_min: Vector3::new(0., 0., 0.),
        aabb_max: Vector3::new(1., 1., 0.),
        ..Default::default()
    };
    mesh_data.add_vertex_pos_color_normal_uv(
        [0., 0., 0.].into(),
        Vector4::default_one(),
        [-1., 1., 0.].into(),
        [0., 0.].into(),
    );
    mesh_data.add_vertex_pos_color_normal_uv(
        [1., 0., 0.].into(),
        Vector4::default_one(),
        [1., 1., 0.].into(),
        [1., 0.].into(),
    );
    mesh_data.add_vertex_pos_color_normal_uv(
        [0.5, 1., 0.].into(),
        Vector4::default_one(),
        [0., -1., 0.].into(),
        [0.5, 1.].into(),
    );

    mesh_data.indices = [0, 2, 1].to_vec();

    let meshlet = MeshletData {
        indices_count: mesh_data.index_count() as _,
        aabb_min: mesh_data.aabb_min(),
        aabb_max: mesh_data.aabb_max(),
        ..Default::default()
    };
    mesh_data.meshlets[0].push(meshlet);
    mesh_data
}

#[inline]
fn append_arc(
    vertices: &mut Vec<Vector2>,
    center_x: f32,
    center_y: f32,
    starting_angle: f32,
    arc: f32,
    radius: f32,
    num_slices: u32,
) {
    let n: u32 = (num_slices as f32 * arc / PI * 2.).ceil() as u32;
    for i in 0..n + 1 {
        let ang = starting_angle - arc * (i as f32) / (n as f32);
        let next_x = center_x + radius * ang.sin();
        let next_y = center_y + radius * ang.cos();
        vertices.push(Vector2::new(next_x, next_y));
    }
}

pub fn create_rounded_rect(
    rect: Vector4,
    corner_radius: f32,
    num_slices: u32,
    color: Vector4,
) -> MeshData {
    let mut mesh_data = MeshData {
        vertex_layout: VertexAttributeLayout::pos_color_normal_uv1(),
        aabb_min: Vector3::new(rect.x, rect.y, 0.),
        aabb_max: Vector3::new(rect.z, rect.w, 0.),
        ..Default::default()
    };
    mesh_data.add_vertex_pos_color_normal_uv(
        [
            rect.x + (rect.z - rect.x) * 0.5,
            rect.y + (rect.w - rect.y) * 0.5,
            0.,
        ]
        .into(),
        color,
        [0., 0., 1.].into(),
        [0.5, 0.5].into(),
    );

    let mut positions = Vec::new();

    // top-left corner
    append_arc(
        &mut positions,
        rect.x + corner_radius,
        rect.y + corner_radius,
        3. * PI / 2.,
        PI / 2.,
        corner_radius,
        num_slices,
    );

    // top-right
    append_arc(
        &mut positions,
        rect.z - corner_radius,
        rect.y + corner_radius,
        PI,
        PI / 2.,
        corner_radius,
        num_slices,
    );

    // bottom-right
    append_arc(
        &mut positions,
        rect.z - corner_radius,
        rect.w - corner_radius,
        PI / 2.,
        PI / 2.,
        corner_radius,
        num_slices,
    );

    // bottom-left
    append_arc(
        &mut positions,
        rect.x + corner_radius,
        rect.w - corner_radius,
        0.,
        PI / 2.,
        corner_radius,
        num_slices,
    );
    let center = mesh_data.position(0);
    for v in positions.iter() {
        let pos: Vector3 = [v.x, v.y, 0.].into();

        mesh_data.add_vertex_pos_color_normal_uv(
            pos,
            color,
            (pos - center).normalized(),
            [rect.z / v.x, rect.w / v.y].into(),
        );
    }

    for i in 1..mesh_data.vertex_count() - 1 {
        mesh_data.indices.push(i as u32 + 1u32);
        mesh_data.indices.push(i as u32);
        mesh_data.indices.push(0u32);
    }

    mesh_data.indices.push(1u32);
    mesh_data
        .indices
        .push((mesh_data.vertex_count() - 1) as u32);
    mesh_data.indices.push(0u32);

    let meshlet = MeshletData {
        indices_count: mesh_data.index_count() as _,
        aabb_min: mesh_data.aabb_min(),
        aabb_max: mesh_data.aabb_max(),
        ..Default::default()
    };
    mesh_data.meshlets[0].push(meshlet);
    mesh_data
}
