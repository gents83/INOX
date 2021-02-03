
pub fn read_spirv_from_bytes<Data: ::std::io::Read + ::std::io::Seek>(data: &mut Data) -> ::std::vec::Vec<u32> {
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
        )).unwrap();
        result.set_len(words);
    }
    const MAGIC_NUMBER: u32 = 0x0723_0203;
    if !result.is_empty() && result[0] == MAGIC_NUMBER.swap_bytes() {
        for word in &mut result {
            *word = word.swap_bytes();
        }
    }
    if result.is_empty() || result[0] != MAGIC_NUMBER {
        panic!("Input data is missing SPIR-V magic number");
    }
    result
}