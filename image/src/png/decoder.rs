use std::{collections::HashMap, io::{BufReader, Cursor, Read, Seek, Write}, marker::PhantomData};
use std::convert::TryFrom;
use crate::colors::*;
use crate::decoder::*;


pub struct ChunkRaw {
    pub ty: [u8; 4],
    pub len: u32,
    pub crc: u32,
    pub data: Vec<u8>,
  }
  
/// PNG decoder
pub struct PngDecoder<R: Read> {
    reader: R,
}

impl<R: Read + Seek> PngDecoder<R> {
    /// Create a new decoder that decodes from the stream ```r```
    pub fn new(buffer: R) -> Self {
        let mut decoder = Self {
            reader: buffer,
        };
        decoder.read_metadata();
        decoder
    }
    
    fn read_metadata(&mut self) {
        let signature: Vec<u8> = vec![0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
        let mut header = vec![0; signature.len()];
        self.reader.read_exact(&mut header);
        
        for i in 0..header.len() {
            if header[i] != signature[i] {
                panic!("Invalid signature in PNG file header");
            }
        }
    }
}


impl<'a, R: 'a + Read> ImageDecoder<'a> for PngDecoder<R> {
    fn dimensions(&self) -> (u32, u32) {
        //self.reader.info().size()
        (0,0)
    }

    fn color_type(&self) -> ColorType {
        //self.color_type
        ColorType::Rgba8
    }

    fn read_image(&mut self, buf: &mut [u8]) {
        assert_eq!(u64::try_from(buf.len()), Ok(self.total_bytes()));
        
    }
}