use std::f32::consts::PI;

use nrg_math::*;

use crate::common::data_formats::*;

pub fn read_spirv_from_bytes<Data: ::std::io::Read + ::std::io::Seek>(
    data: &mut Data,
) -> ::std::vec::Vec<u32> {
    let size = data.seek(::std::io::SeekFrom::End(0)).unwrap();
    if size % 4 != 0 {
        panic!("Input data length not divisible by 4");
    }
    if size > usize::max_value() as u64 {
        panic!("Input data too long");
    }
    let words = (size / 4) as usize;
    let mut result = Vec::<u32>::with_capacity(words);
    data.seek(::std::io::SeekFrom::Start(0)).unwrap();
    unsafe {
        data.read_exact(::std::slice::from_raw_parts_mut(
            result.as_mut_ptr() as *mut u8,
            words * 4,
        ))
        .unwrap();
        result.set_len(words);
    }
    const MAGIC_NUMBER: u32 = 0x0723_0203;
    if !result.is_empty() {
        if result[0] == MAGIC_NUMBER.swap_bytes() {
            for word in &mut result {
                *word = word.swap_bytes();
            }
        } else if result[0] != MAGIC_NUMBER {
            panic!("Input data is missing SPIR-V magic number");
        }
    } else {
        panic!("Input data is empty");
    }
    result
}

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
            color: [1., 1., 1., 1.].into(),
            tex_coord: [tex_coords.x, tex_coords.y].into(),
        },
        VertexData {
            pos: [rect.x, rect.w, z].into(),
            normal: [-1., 1., 0.].into(),
            color: [1., 1., 1., 1.].into(),
            tex_coord: [tex_coords.x, tex_coords.w].into(),
        },
        VertexData {
            pos: [rect.z, rect.w, z].into(),
            normal: [1., 1., 0.].into(),
            color: [1., 1., 1., 1.].into(),
            tex_coord: [tex_coords.z, tex_coords.w].into(),
        },
        VertexData {
            pos: [rect.z, rect.y, z].into(),
            normal: [1., -1., 0.].into(),
            color: [1., 1., 1., 1.].into(),
            tex_coord: [tex_coords.z, tex_coords.y].into(),
        },
    ];
    let index_offset: u32 = index_start.unwrap_or(0) as _;
    let indices: [u32; 6] = [
        index_offset,
        1 + index_offset,
        2 + index_offset,
        2 + index_offset,
        3 + index_offset,
        index_offset,
    ];
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
    vertices[0].tex_coord = [0., 1.].into();
    vertices[1].tex_coord = [1., 1.].into();
    vertices[2].tex_coord = [0.5, 0.].into();
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
    vertices[0].tex_coord = [0., 0.].into();
    vertices[1].tex_coord = [1., 0.].into();
    vertices[2].tex_coord = [0.5, 1.].into();
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
    vertices[0].tex_coord = [0., 0.].into();
    vertices[1].tex_coord = [1., 0.5].into();
    vertices[2].tex_coord = [0., 1.].into();
    vertices[0].color = [1., 1., 1., 1.].into();
    vertices[1].color = [1., 1., 1., 1.].into();
    vertices[2].color = [1., 1., 1., 1.].into();
    let indices = [0u32, 2, 1];
    (vertices, indices)
}

pub fn create_cube(size: Vector3) -> ([VertexData; 8], [u32; 36]) {
    let mut vertices = [VertexData::default(); 8];
    vertices[0].pos = [-size.x, -size.y, -size.z].into();
    vertices[1].pos = [size.x, -size.y, -size.z].into();
    vertices[2].pos = [size.x, size.y, -size.z].into();
    vertices[3].pos = [-size.x, size.y, -size.z].into();
    vertices[4].pos = [-size.x, -size.y, size.z].into();
    vertices[5].pos = [size.x, -size.y, size.z].into();
    vertices[6].pos = [size.x, size.y, size.z].into();
    vertices[7].pos = [-size.x, size.y, size.z].into();
    vertices[0].normal = [-1., -1., -1.].into();
    vertices[1].normal = [1., -1., -1.].into();
    vertices[2].normal = [1., 1., -1.].into();
    vertices[3].normal = [-1., 1., -1.].into();
    vertices[4].normal = [-1., -1., 1.].into();
    vertices[5].normal = [1., -1., 1.].into();
    vertices[6].normal = [1., 1., 1.].into();
    vertices[7].normal = [-1., 1., 1.].into();
    vertices[0].tex_coord = [0., 0.].into();
    vertices[1].tex_coord = [1., 0.].into();
    vertices[2].tex_coord = [1., 1.].into();
    vertices[3].tex_coord = [0., 1.].into();
    vertices[4].tex_coord = [0., 0.].into();
    vertices[5].tex_coord = [1., 0.].into();
    vertices[6].tex_coord = [1., 1.].into();
    vertices[7].tex_coord = [0., 1.].into();
    let indices = [
        0, 1, 3, 3, 1, 2, 1, 5, 2, 2, 5, 6, 5, 4, 6, 6, 4, 7, 4, 0, 7, 7, 0, 3, 3, 2, 7, 7, 2, 6,
        4, 5, 0, 0, 5, 1,
    ];
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
        color: [1., 1., 1., 1.].into(),
        tex_coord: [0.5, 0.5].into(),
        normal: [0., 0., 1.].into(),
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
            color: [1., 1., 1., 1.].into(),
            tex_coord: [rect.z / v.x, rect.w / v.y].into(),
            normal: (pos - center.pos).normalize(),
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
