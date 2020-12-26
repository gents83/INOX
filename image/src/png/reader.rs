use std::io::{BufRead, Seek};
use super::decoder::*;
use crate::decoder::*;
use crate::image::*;


pub struct Reader {}

impl Reader {
    pub fn create_from<'a, R: BufRead + Seek>(buffer: R) -> Image {
        let mut decoder = PngDecoder::new(buffer);
        let bytes = decoder.total_bytes();
        let mut buf = Vec::with_capacity(bytes as _);
        unsafe { buf.set_len(bytes as _); }
        decoder.read_image(buf.as_mut_slice());
        let dim = decoder.dimensions();
        Image {
            data: buf,
            width: dim.0,
            height: dim.1,
            channel_count: decoder.color_type().channel_count(),
            color_type: decoder.color_type(),
        }
    }    
}