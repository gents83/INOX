use nrg_math::{VecBase, Vector4};
use nrg_serialize::Uid;

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

#[inline]
pub fn compute_color_from_id(id: Uid) -> Vector4 {
    let id_as_bytes = id.as_bytes();
    let mut color = Vector4::default_zero();
    color.x = ((id_as_bytes[0] as u32) << 24
        | (id_as_bytes[1] as u32) << 16
        | (id_as_bytes[2] as u32) << 8
        | (id_as_bytes[3] as u32)) as _;
    color.y = ((id_as_bytes[4] as u32) << 24
        | (id_as_bytes[5] as u32) << 16
        | (id_as_bytes[6] as u32) << 8
        | (id_as_bytes[7] as u32)) as _;
    color.z = ((id_as_bytes[8] as u32) << 24
        | (id_as_bytes[9] as u32) << 16
        | (id_as_bytes[10] as u32) << 8
        | (id_as_bytes[11] as u32)) as _;
    color.w = ((id_as_bytes[12] as u32) << 24
        | (id_as_bytes[13] as u32) << 16
        | (id_as_bytes[14] as u32) << 8
        | (id_as_bytes[15] as u32)) as _;
    color
}

#[inline]
pub fn compute_id_from_color(color: Vector4) -> Uid {
    let mut id_as_bytes: [u8; 16] = [0; 16];
    id_as_bytes[0] = (color.x as u32 >> 24) as u8;
    id_as_bytes[1] = (color.x as u32 >> 16) as u8;
    id_as_bytes[2] = (color.x as u32 >> 8) as u8;
    id_as_bytes[3] = color.x as u32 as u8;
    id_as_bytes[4] = (color.y as u32 >> 24) as u8;
    id_as_bytes[5] = (color.y as u32 >> 16) as u8;
    id_as_bytes[6] = (color.y as u32 >> 8) as u8;
    id_as_bytes[7] = color.y as u32 as u8;
    id_as_bytes[8] = (color.z as u32 >> 24) as u8;
    id_as_bytes[9] = (color.z as u32 >> 16) as u8;
    id_as_bytes[10] = (color.z as u32 >> 8) as u8;
    id_as_bytes[11] = color.z as u32 as u8;
    id_as_bytes[12] = (color.w as u32 >> 24) as u8;
    id_as_bytes[13] = (color.w as u32 >> 16) as u8;
    id_as_bytes[14] = (color.w as u32 >> 8) as u8;
    id_as_bytes[15] = color.w as u32 as u8;
    Uid::from_bytes(id_as_bytes)
}
