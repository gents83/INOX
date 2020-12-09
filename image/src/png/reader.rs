use std::io::{BufRead, Seek};
use crate::image::*;


pub struct Reader {}

impl Reader {
    pub fn create_from<'a, R: BufRead + Seek>(_buffer: R) -> Image {
        Image::default()
    }    
}