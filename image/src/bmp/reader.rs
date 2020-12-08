use std::io::{BufRead, Seek};
use super::decoder::*;
use crate::decoder::*;
use crate::reader::*;

pub struct Reader {}

impl Reader {
    pub fn create_from<'a, R: BufRead + Seek>(buffer: R) -> Image {
        let decoder = BmpDecoder::new(buffer);
        let bytes = decoder.total_bytes();
        let mut buf = Vec::with_capacity(bytes as _);
        unsafe { buf.set_len(bytes as _); }
        decoder.read_image(buf.as_mut_slice());
        Image {
            data: buf,
        }
    }    
}