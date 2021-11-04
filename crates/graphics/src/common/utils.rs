use nrg_math::Vector4;

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
pub fn compute_color_from_id(id: u32) -> Vector4 {
    let r = ((id & 0xFF) as f32) / 255.;
    let g = ((id >> 8) & 0xFF) as f32 / 255.;
    let b = ((id >> 16) & 0xFF) as f32 / 255.;
    let a = ((id >> 24) & 0xFF) as f32 / 255.;
    Vector4::new(r, g, b, a)
}

#[inline]
pub fn compute_id_from_color(color: Vector4) -> u32 {
    let color = color * 255.;
    (color.x as u32) | (color.y as u32) << 8 | (color.z as u32) << 16 | (color.w as u32) << 24
}
