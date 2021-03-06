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
    rect: Vector4f,
    z: f32,
    tex_coords: Vector4f,
    index_start: Option<usize>,
) -> ([VertexData; 4], [u32; 6]) {
    let vertices = [
        VertexData {
            pos: [rect.x, rect.y, z].into(),
            normal: [0., 0., 1.].into(),
            color: [1., 1., 1., 1.].into(),
            tex_coord: [tex_coords.z, tex_coords.y].into(),
        },
        VertexData {
            pos: [rect.z, rect.y, z].into(),
            normal: [0., 0., 1.].into(),
            color: [1., 1., 1., 1.].into(),
            tex_coord: [tex_coords.x, tex_coords.y].into(),
        },
        VertexData {
            pos: [rect.z, rect.w, z].into(),
            normal: [0., 0., 1.].into(),
            color: [1., 1., 1., 1.].into(),
            tex_coord: [tex_coords.x, tex_coords.w].into(),
        },
        VertexData {
            pos: [rect.x, rect.w, z].into(),
            normal: [0., 0., 1.].into(),
            color: [1., 1., 1., 1.].into(),
            tex_coord: [tex_coords.z, tex_coords.w].into(),
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
