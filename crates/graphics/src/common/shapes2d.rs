use std::f32::consts::PI;

use nrg_math::{VecBase, Vector2, Vector3, Vector4};

use crate::{VertexData, MAX_TEXTURE_COORDS_SETS};

pub fn create_quad(
    rect: Vector4,
    z: f32,
    tex_coords: Vector4,
    index_start: Option<usize>,
) -> ([VertexData; 4], [u32; 6]) {
    let vertices = [
        VertexData {
            pos: [rect.x, rect.y, z].into(),
            normal: [-1., -1., 0.].into(),
            tex_coord: [[tex_coords.x, tex_coords.y].into(); MAX_TEXTURE_COORDS_SETS],
            ..Default::default()
        },
        VertexData {
            pos: [rect.x, rect.w, z].into(),
            normal: [-1., 1., 0.].into(),
            tex_coord: [[tex_coords.x, tex_coords.w].into(); MAX_TEXTURE_COORDS_SETS],
            ..Default::default()
        },
        VertexData {
            pos: [rect.z, rect.w, z].into(),
            normal: [1., 1., 0.].into(),
            tex_coord: [[tex_coords.z, tex_coords.w].into(); MAX_TEXTURE_COORDS_SETS],
            ..Default::default()
        },
        VertexData {
            pos: [rect.z, rect.y, z].into(),
            normal: [1., -1., 0.].into(),
            tex_coord: [[tex_coords.z, tex_coords.y].into(); MAX_TEXTURE_COORDS_SETS],
            ..Default::default()
        },
    ];
    let index_offset: u32 = index_start.unwrap_or(0) as _;
    let indices: [u32; 6] = [
        index_offset,
        2 + index_offset,
        1 + index_offset,
        3 + index_offset,
        2 + index_offset,
        index_offset,
    ];
    (vertices, indices)
}
pub fn create_colored_quad(rect: Vector4, z: f32, color: Vector4) -> ([VertexData; 4], [u32; 6]) {
    let vertices = [
        VertexData {
            pos: [rect.x, rect.y, z].into(),
            normal: [-1., -1., 0.].into(),
            color,
            tex_coord: [[0., 0.].into(); MAX_TEXTURE_COORDS_SETS],
            ..Default::default()
        },
        VertexData {
            pos: [rect.x, rect.w, z].into(),
            normal: [-1., 1., 0.].into(),
            color,
            tex_coord: [[0., 1.].into(); MAX_TEXTURE_COORDS_SETS],
            ..Default::default()
        },
        VertexData {
            pos: [rect.z, rect.w, z].into(),
            normal: [1., 1., 0.].into(),
            color,
            tex_coord: [[1., 1.].into(); MAX_TEXTURE_COORDS_SETS],
            ..Default::default()
        },
        VertexData {
            pos: [rect.z, rect.y, z].into(),
            normal: [1., -1., 0.].into(),
            color,
            tex_coord: [[1., 0.].into(); MAX_TEXTURE_COORDS_SETS],
            ..Default::default()
        },
    ];
    let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];
    (vertices, indices)
}

pub fn create_triangle_up() -> ([VertexData; 3], [u32; 3]) {
    let mut vertices = [VertexData::default(); 3];
    vertices[0].pos = [0., 1., 0.].into();
    vertices[1].pos = [1., 1., 0.].into();
    vertices[2].pos = [0.5, 0., 0.].into();
    vertices[0].normal = [-1., -1., 0.].into();
    vertices[1].normal = [1., -1., 0.].into();
    vertices[2].normal = [0., 1., 0.].into();
    vertices[0].tex_coord = [[0., 1.].into(); MAX_TEXTURE_COORDS_SETS];
    vertices[1].tex_coord = [[1., 1.].into(); MAX_TEXTURE_COORDS_SETS];
    vertices[2].tex_coord = [[0.5, 0.].into(); MAX_TEXTURE_COORDS_SETS];
    vertices[0].color = [1., 1., 1., 1.].into();
    vertices[1].color = [1., 1., 1., 1.].into();
    vertices[2].color = [1., 1., 1., 1.].into();
    let indices = [0u32, 2, 1];
    (vertices, indices)
}

pub fn create_triangle_down() -> ([VertexData; 3], [u32; 3]) {
    let mut vertices = [VertexData::default(); 3];
    vertices[0].pos = [0., 0., 0.].into();
    vertices[1].pos = [1., 0., 0.].into();
    vertices[2].pos = [0.5, 1., 0.].into();
    vertices[0].normal = [-1., 1., 0.].into();
    vertices[1].normal = [1., 1., 0.].into();
    vertices[2].normal = [0., -1., 0.].into();
    vertices[0].tex_coord = [[0., 0.].into(); MAX_TEXTURE_COORDS_SETS];
    vertices[1].tex_coord = [[1., 0.].into(); MAX_TEXTURE_COORDS_SETS];
    vertices[2].tex_coord = [[0.5, 1.].into(); MAX_TEXTURE_COORDS_SETS];
    vertices[0].color = [1., 1., 1., 1.].into();
    vertices[1].color = [1., 1., 1., 1.].into();
    vertices[2].color = [1., 1., 1., 1.].into();
    let indices = [0u32, 2, 1];
    (vertices, indices)
}

pub fn create_triangle_right() -> ([VertexData; 3], [u32; 3]) {
    let mut vertices = [VertexData::default(); 3];
    vertices[0].pos = [0., 0., 0.].into();
    vertices[1].pos = [1., 0.5, 0.].into();
    vertices[2].pos = [0., 1., 0.].into();
    vertices[0].normal = [-1., 1., 0.].into();
    vertices[1].normal = [1., 0., 0.].into();
    vertices[2].normal = [-1., -1., 0.].into();
    vertices[0].tex_coord = [[0., 0.].into(); MAX_TEXTURE_COORDS_SETS];
    vertices[1].tex_coord = [[1., 0.5].into(); MAX_TEXTURE_COORDS_SETS];
    vertices[2].tex_coord = [[0., 1.].into(); MAX_TEXTURE_COORDS_SETS];
    vertices[0].color = [1., 1., 1., 1.].into();
    vertices[1].color = [1., 1., 1., 1.].into();
    vertices[2].color = [1., 1., 1., 1.].into();
    let indices = [0u32, 2, 1];
    (vertices, indices)
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
) -> (Vec<VertexData>, Vec<u32>) {
    let center = VertexData {
        pos: [
            rect.x + (rect.z - rect.x) * 0.5,
            rect.y + (rect.w - rect.y) * 0.5,
            0.,
        ]
        .into(),
        tex_coord: [[0.5, 0.5].into(); MAX_TEXTURE_COORDS_SETS],
        ..Default::default()
    };

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

    let mut vertices = vec![center];
    for v in positions.iter() {
        let pos: Vector3 = [v.x, v.y, 0.].into();
        vertices.push(VertexData {
            pos,
            tex_coord: [[rect.z / v.x, rect.w / v.y].into(); MAX_TEXTURE_COORDS_SETS],
            normal: (pos - center.pos).normalized(),
            ..Default::default()
        });
    }
    let mut indices = Vec::new();

    for i in 1..vertices.len() - 1 {
        indices.push(i as u32 + 1u32);
        indices.push(i as u32);
        indices.push(0u32);
    }

    indices.push(1u32);
    indices.push((vertices.len() - 1) as u32);
    indices.push(0u32);

    (vertices, indices)
}
