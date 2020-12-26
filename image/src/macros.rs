
#[macro_export]
macro_rules! read_bytes_from_reader {
    ($ty:ty, $size:expr, $reader:expr, $which:ident) => ({
        let mut buf = [0; $size];
        let _res = $reader.read_exact(&mut buf); 
        read_bytes!($ty, $size, buf, $which)
    });
}

#[macro_export]
macro_rules! read_bytes {
    ($ty:ty, $size:expr, $buf:expr, $which:ident) => ({
        assert!($size == ::core::mem::size_of::<$ty>());
        assert!($size <= $buf.len());
        let mut data: $ty = 0;
        unsafe {
            core::ptr::copy_nonoverlapping(
                $buf.as_ptr(),
                &mut data as *mut $ty as *mut u8,
                $size);
        }
        data.$which()
    });
}


#[macro_export]
macro_rules! write_bytes {
    ($ty:ty, $size:expr, $n:expr, $dst:expr, $which:ident) => ({
        assert!($size <= $dst.len());
        unsafe {
            let bytes = *(&$n.$which() as *const _ as *const [u8; $size]);
            core::ptr::copy_nonoverlapping((&bytes).as_ptr(), $dst.as_mut_ptr(), $size);
        }
    });
}