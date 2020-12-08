use std::io::{BufRead, Seek};
use crate::reader::*;


pub struct Reader {}

impl Reader {
    pub fn create_from<'a, R: BufRead + Seek>(buffer: R) -> Image {
        Image {
            data: Vec::new(),
        }
    }    
}