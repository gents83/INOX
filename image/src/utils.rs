
use std::io::{Read, Seek};
use crate::{read_bytes, read_bytes_from_reader};
use crate::write_bytes;

#[inline]
pub fn read_u32_from_reader_be<R: Read + Seek>(reader: &mut R) -> u32 {
    read_bytes_from_reader!(u32, 4, reader, to_be)
}

#[inline]
pub fn read_u32_from_reader_le<R: Read + Seek>(reader: &mut R) -> u32 {
    read_bytes_from_reader!(u32, 4, reader, to_le)
}

#[inline]
pub fn read_i32_from_reader_be<R: Read + Seek>(reader: &mut R) -> i32 {
    read_bytes_from_reader!(i32, 4, reader, to_be)
}

#[inline]
pub fn read_i32_from_reader_le<R: Read + Seek>(reader: &mut R) -> i32 {
    read_bytes_from_reader!(i32, 4, reader, to_le)
}

#[inline]
pub fn read_u16_from_reader_be<R: Read + Seek>(reader: &mut R) -> u16 {
    read_bytes_from_reader!(u16, 2, reader, to_be)
}

#[inline]
pub fn read_u16_from_reader_le<R: Read + Seek>(reader: &mut R) -> u16 { 
    read_bytes_from_reader!(u16, 2, reader, to_le)
}

#[inline]
pub fn read_u8_from_reader<R: Read>(reader: &mut R) -> u8 {
    read_bytes_from_reader!(u8, 1, reader, to_le)
}



#[inline]
pub fn read_u32_be(buf: &[u8]) -> u32 {
    read_bytes!(u32, 4, buf, to_be)
}

#[inline]
pub fn read_u32_le(buf: &[u8]) -> u32 {
    read_bytes!(u32, 4, buf, to_le)
}

#[inline]
pub fn read_i32_be(buf: &[u8]) -> i32 {
    read_bytes!(i32, 4, buf, to_be)
}

#[inline]
pub fn read_i32_le(buf: &[u8]) -> i32 {
    read_bytes!(i32, 4, buf, to_le)
}

#[inline]
pub fn read_u16_be(buf: &[u8]) -> u16 {
    read_bytes!(u16, 2, buf, to_be)
}

#[inline]
pub fn read_u16_le(buf: &[u8]) -> u16 { 
    read_bytes!(u16, 2, buf, to_le)
}

#[inline]
pub fn read_u8(buf: &[u8]) -> u8 {
    read_bytes!(u8, 1, buf, to_le)
}



#[inline]
pub fn write_u32_be(buf: &mut [u8], n: u32) {
    write_bytes!(u32, 4, n, buf, to_be)
}

#[inline]
pub fn write_u32_le(buf: &mut [u8], n: u32) {
    write_bytes!(u32, 4, n, buf, to_le)
}

#[inline]
pub fn write_i32_be(buf: &mut [u8], n: i32) {
    write_bytes!(i32, 4, n, buf, to_be)
}

#[inline]
pub fn write_i32_le(buf: &mut [u8], n: i32) {
    write_bytes!(i32, 4, n, buf, to_le)
}

#[inline]
pub fn write_u16_be(buf: &mut [u8], n: u16) {
    write_bytes!(u16, 2, n, buf, to_be)
}

#[inline]
pub fn write_u16_le(buf: &mut [u8], n: u16) { 
    write_bytes!(u16, 2, n, buf, to_le)
}

#[inline]
pub fn write_u8(buf: &mut [u8], n: u8) {
    write_bytes!(u8, 1, n, buf, to_le)
}