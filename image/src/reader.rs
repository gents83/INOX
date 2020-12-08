use core::panic;
use std::{fs::File, io::BufReader, path::Path};
use crate::PngReader;
use crate::JpgReader;
use crate::BmpReader;

use super::formats::*;

pub struct Image {
    pub data: Vec<u8>,
}

pub struct Reader {}

impl Reader {
    pub fn load<'a>(path: &Path) -> Image {
        let file = File::open(path).unwrap();
        let buffer = BufReader::new(file);
        let image_format = ImageFormat::from_extension(path);

        match image_format {
            Some(ImageFormat::PNG) => PngReader::create_from(buffer),
            Some(ImageFormat::BMP) => BmpReader::create_from(buffer),
            Some(ImageFormat::JPG) => JpgReader::create_from(buffer),
            _ => panic!("Unable to load image"),
        }
    }    
}