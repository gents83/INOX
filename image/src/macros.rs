
#[macro_export]
macro_rules! read_bytes {
    ($ty:ty, $size:expr, $reader:expr, $which:ident) => ({
        let mut buf = [0; $size];
        let _res = $reader.read_exact(&mut buf); 
        assert!($size == ::core::mem::size_of::<$ty>());
        assert!($size <= buf.len());
        let mut data: $ty = 0;
        unsafe {
            core::ptr::copy_nonoverlapping(
                buf.as_ptr(),
                &mut data as *mut $ty as *mut u8,
                $size);
        }
        data.$which()
    });
}
