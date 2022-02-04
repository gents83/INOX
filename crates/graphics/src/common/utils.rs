use inox_math::Vector4;

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

pub fn as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe {
        ::std::slice::from_raw_parts((p as *const T) as *const u8, ::std::mem::size_of::<T>())
    }
}
pub fn to_u8_slice<T: Sized>(a: &[T]) -> &[u8] {
    unsafe {
        let len = a.len() * ::std::mem::size_of::<T>();
        ::std::slice::from_raw_parts((&a[0] as *const T) as *const u8, len)
    }
}
