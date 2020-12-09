
use std::io::{Read, Seek};
use crate::read_bytes;

#[inline]
pub fn read_u32_be<R: Read + Seek>(reader: &mut R) -> u32 {
    read_bytes!(u32, 4, reader, to_be)
}

#[inline]
pub fn read_u32_le<R: Read + Seek>(reader: &mut R) -> u32 {
    read_bytes!(u32, 4, reader, to_le)
}

#[inline]
pub fn read_i32_be<R: Read + Seek>(reader: &mut R) -> i32 {
    read_bytes!(i32, 4, reader, to_be)
}

#[inline]
pub fn read_i32_le<R: Read + Seek>(reader: &mut R) -> i32 {
    read_bytes!(i32, 4, reader, to_le)
}

#[inline]
pub fn read_u16_be<R: Read + Seek>(reader: &mut R) -> u16 {
    read_bytes!(u16, 2, reader, to_be)
}

#[inline]
pub fn read_u16_le<R: Read + Seek>(reader: &mut R) -> u16 { 
    read_bytes!(u16, 2, reader, to_le)
}

#[inline]
pub fn read_u8<R: Read>(reader: &mut R) -> u8 {
    read_bytes!(u8, 1, reader, to_le)
}