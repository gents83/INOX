use std::{io::{Cursor, Read, Seek, SeekFrom}, marker::PhantomData};
use std::cmp::{self, Ordering};
use std::convert::TryFrom;
use std::slice::ChunksMut;
use std::iter::{repeat, Iterator, Rev};
use crate::colors::*;
use crate::decoder::*;
use crate::utils::*;

const BITMAPCOREHEADER_SIZE: u32 = 12;
const BITMAPINFOHEADER_SIZE: u32 = 40;
const BITMAPV2HEADER_SIZE: u32 = 52;
const BITMAPV3HEADER_SIZE: u32 = 56;
const BITMAPV4HEADER_SIZE: u32 = 108;
const BITMAPV5HEADER_SIZE: u32 = 124;

const MAX_WIDTH_HEIGHT: i32 = 0xFFFF;
const MAX_INITIAL_PIXELS: usize = 8192 * 4096;

const RLE_ESCAPE: u8 = 0;
const RLE_ESCAPE_EOL: u8 = 0;
const RLE_ESCAPE_EOF: u8 = 1;
const RLE_ESCAPE_DELTA: u8 = 2;

static LOOKUP_TABLE_3_BIT_TO_8_BIT: [u8; 8] = [0, 36, 73, 109, 146, 182, 219, 255];
static LOOKUP_TABLE_4_BIT_TO_8_BIT: [u8; 16] = [
    0, 17, 34, 51, 68, 85, 102, 119, 136, 153, 170, 187, 204, 221, 238, 255,
];
static LOOKUP_TABLE_5_BIT_TO_8_BIT: [u8; 32] = [
    0, 8, 16, 25, 33, 41, 49, 58, 66, 74, 82, 90, 99, 107, 115, 123, 132, 140, 148, 156, 165, 173,
    181, 189, 197, 206, 214, 222, 230, 239, 247, 255,
];
static LOOKUP_TABLE_6_BIT_TO_8_BIT: [u8; 64] = [
    0, 4, 8, 12, 16, 20, 24, 28, 32, 36, 40, 45, 49, 53, 57, 61, 65, 69, 73, 77, 81, 85, 89, 93,
    97, 101, 105, 109, 113, 117, 121, 125, 130, 134, 138, 142, 146, 150, 154, 158, 162, 166, 170,
    174, 178, 182, 186, 190, 194, 198, 202, 206, 210, 215, 219, 223, 227, 231, 235, 239, 243, 247,
    251, 255,
];

static R5_G5_B5_COLOR_MASK: Bitfields = Bitfields {
    r: Bitfield { len: 5, shift: 10 },
    g: Bitfield { len: 5, shift: 5 },
    b: Bitfield { len: 5, shift: 0 },
    a: Bitfield { len: 0, shift: 0 },
};
const R8_G8_B8_COLOR_MASK: Bitfields = Bitfields {
    r: Bitfield { len: 8, shift: 24 },
    g: Bitfield { len: 8, shift: 16 },
    b: Bitfield { len: 8, shift: 8 },
    a: Bitfield { len: 0, shift: 0 },
};

#[derive(PartialEq)]
enum BmpHeaderType {
    Core,
    Info,
    V2,
    V3,
    V4,
    V5,
}

#[derive(PartialEq)]
enum FormatFullBytes {
    RGB24,
    RGB32,
    RGBA32,
    Format888,
}

#[derive(PartialEq, Copy, Clone)]
enum ImageType {
    Palette,
    RGB16,
    RGB24,
    RGB32,
    RGBA32,
    RLE8,
    RLE4,
    Bitfields16,
    Bitfields32,
}


#[derive(PartialEq, Eq)]
struct Bitfield {
    shift: u32,
    len: u32,
}

impl Bitfield {
    fn from_mask(mask: u32, max_len: u32) -> Bitfield {
        if mask == 0 {
            return Bitfield { shift: 0, len: 0 };
        }
        let mut shift = mask.trailing_zeros();
        let mut len = (!(mask >> shift)).trailing_zeros();
        if len != mask.count_ones() {
            panic!("Bitmask not contiguos");
        }
        if len + shift > max_len {
            panic!("Bitmask invalid");
        }
        if len > 8 {
            shift += len - 8;
            len = 8;
        }
        Bitfield { 
            shift: shift, 
            len: len
        }
    }

    fn read(&self, data: u32) -> u8 {
        let data = data >> self.shift;
        match self.len {
            1 => ((data & 0b1) * 0xff) as u8,
            2 => ((data & 0b11) * 0x55) as u8,
            3 => LOOKUP_TABLE_3_BIT_TO_8_BIT[(data & 0b00_0111) as usize],
            4 => LOOKUP_TABLE_4_BIT_TO_8_BIT[(data & 0b00_1111) as usize],
            5 => LOOKUP_TABLE_5_BIT_TO_8_BIT[(data & 0b01_1111) as usize],
            6 => LOOKUP_TABLE_6_BIT_TO_8_BIT[(data & 0b11_1111) as usize],
            7 => ((data & 0x7f) << 1 | (data & 0x7f) >> 6) as u8,
            8 => (data & 0xff) as u8,
            _ => panic!(),
        }
    }
}


#[derive(PartialEq, Eq)]
struct Bitfields {
    r: Bitfield,
    g: Bitfield,
    b: Bitfield,
    a: Bitfield,
}



impl Bitfields {
    fn from_mask(r_mask: u32, g_mask: u32, b_mask: u32, a_mask: u32, max_len: u32) -> Bitfields {
        let bitfields = Bitfields {
            r: Bitfield::from_mask(r_mask, max_len),
            g: Bitfield::from_mask(g_mask, max_len),
            b: Bitfield::from_mask(b_mask, max_len),
            a: Bitfield::from_mask(a_mask, max_len),
        };
        if bitfields.r.len == 0 || bitfields.g.len == 0 || bitfields.b.len == 0 {
            panic!("Bitfield mask missing");
        }
        bitfields
    }
}

enum Chunker<'a> {
    FromTop(ChunksMut<'a, u8>),
    FromBottom(Rev<ChunksMut<'a, u8>>),
}

pub(crate) struct RowIterator<'a> {
    chunks: Chunker<'a>,
}

impl<'a> Iterator for RowIterator<'a> {
    type Item = &'a mut [u8];

    #[inline(always)]
    fn next(&mut self) -> Option<&'a mut [u8]> {
        match self.chunks {
            Chunker::FromTop(ref mut chunks) => chunks.next(),
            Chunker::FromBottom(ref mut chunks) => chunks.next(),
        }
    }
}


fn extend_buffer(buffer: &mut Vec<u8>, full_size: usize, blank: bool) -> &mut [u8] {
    let old_size = buffer.len();
    let extend = full_size - buffer.len();

    buffer.extend(repeat(0xFF).take(extend));
    assert_eq!(buffer.len(), full_size);

    let ret = if extend >= old_size {
        // If the full buffer length is more or equal to twice the initial one, we can simply
        // copy the data in the lower part of the buffer to the end of it and input from there.
        let (new, old) = buffer.split_at_mut(extend);
        old.copy_from_slice(&new[..old_size]);
        new
    } else {
        // If the full size is less than twice the initial buffer, we have to
        // copy in two steps
        let overlap = old_size - extend;

        // First we copy the data that fits into the bit we extended.
        let (lower, upper) = buffer.split_at_mut(old_size);
        upper.copy_from_slice(&lower[overlap..]);

        // Then we slide the data that hasn't been copied yet to the top of the buffer
        let (new, old) = lower.split_at_mut(extend);
        old[..overlap].copy_from_slice(&new[..overlap]);
        new
    };
    if blank {
        for b in ret.iter_mut() {
            *b = 0;
        }
    };
    ret
}

fn with_rows<F>(
    buffer: &mut Vec<u8>,
    width: i32,
    height: i32,
    channels: usize,
    top_down: bool,
    mut func: F,
) -> ::std::io::Result<()>
where
    F: FnMut(&mut [u8]) -> ::std::io::Result<()>,
{
    // An overflow should already have been checked for when this is called,
    // though we check anyhow, as it somehow seems to increase performance slightly.
    let row_width = channels.checked_mul(width as usize).unwrap();
    let full_image_size = row_width.checked_mul(height as usize).unwrap();

    if !top_down {
        for row in buffer.chunks_mut(row_width).rev() {
            func(row)?;
        }

        // If we need more space, extend the buffer.
        if buffer.len() < full_image_size {
            let new_space = extend_buffer(buffer, full_image_size, false);
            for row in new_space.chunks_mut(row_width).rev() {
                func(row)?;
            }
        }
    } else {
        for row in buffer.chunks_mut(row_width) {
            func(row)?;
        }
        if buffer.len() < full_image_size {
            // If the image is stored in top-down order, we can simply use the extend function
            // from vec to extend the buffer..
            let extend = full_image_size - buffer.len();
            buffer.extend(repeat(0xFF).take(extend));
            let len = buffer.len();
            for row in buffer[len - row_width..].chunks_mut(row_width) {
                func(row)?;
            }
        };
    }
    Ok(())
}


fn set_8bit_pixel_run<'a, T: Iterator<Item = &'a u8>>(
    pixel_iter: &mut ChunksMut<u8>,
    palette: &[(u8, u8, u8)],
    indices: T,
    n_pixels: usize,
) -> bool {
    for idx in indices.take(n_pixels) {
        if let Some(pixel) = pixel_iter.next() {
            let (r, g, b) = palette[*idx as usize];
            pixel[0] = r;
            pixel[1] = g;
            pixel[2] = b;
        } else {
            return false;
        }
    }
    true
}

fn set_4bit_pixel_run<'a, T: Iterator<Item = &'a u8>>(
    pixel_iter: &mut ChunksMut<u8>,
    palette: &[(u8, u8, u8)],
    indices: T,
    mut n_pixels: usize,
) -> bool {
    for idx in indices {
        macro_rules! set_pixel {
            ($i:expr) => {
                if n_pixels == 0 {
                    break;
                }
                if let Some(pixel) = pixel_iter.next() {
                    let (r, g, b) = palette[$i as usize];
                    pixel[0] = r;
                    pixel[1] = g;
                    pixel[2] = b;
                } else {
                    return false;
                }
                n_pixels -= 1;
            };
        }
        set_pixel!(idx >> 4);
        set_pixel!(idx & 0xf);
    }
    true
}

fn set_2bit_pixel_run<'a, T: Iterator<Item = &'a u8>>(
    pixel_iter: &mut ChunksMut<u8>,
    palette: &[(u8, u8, u8)],
    indices: T,
    mut n_pixels: usize,
) -> bool {
    for idx in indices {
        macro_rules! set_pixel {
            ($i:expr) => {
                if n_pixels == 0 {
                    break;
                }
                if let Some(pixel) = pixel_iter.next() {
                    let (r, g, b) = palette[$i as usize];
                    pixel[0] = r;
                    pixel[1] = g;
                    pixel[2] = b;
                } else {
                    return false;
                }
                n_pixels -= 1;
            };
        }
        set_pixel!((idx >> 6) & 0x3u8);
        set_pixel!((idx >> 4) & 0x3u8);
        set_pixel!((idx >> 2) & 0x3u8);
        set_pixel!( idx       & 0x3u8);
    }
    true
}

fn set_1bit_pixel_run<'a, T: Iterator<Item = &'a u8>>(
    pixel_iter: &mut ChunksMut<u8>,
    palette: &[(u8, u8, u8)],
    indices: T,
) {
    for idx in indices {
        let mut bit = 0x80;
        loop {
            if let Some(pixel) = pixel_iter.next() {
                let (r, g, b) = palette[((idx & bit) != 0) as usize];
                pixel[0] = r;
                pixel[1] = g;
                pixel[2] = b;
            } else {
                return;
            }

            bit >>= 1;
            if bit == 0 {
                break;
            }
        }
    }
}

fn num_bytes(width: i32, length: i32, channels: usize) -> Option<usize> {
    if width <= 0 || length <= 0 {
        None
    } else {
        match channels.checked_mul(width as usize) {
            Some(n) => n.checked_mul(length as usize),
            None => None,
        }
    }
}

fn blank_bytes<'a, T: Iterator<Item = &'a mut [u8]>>(iterator: T) {
    for chunk in iterator {
        for b in chunk {
            *b = 0;
        }
    }
}


enum RLEInsn {
    EndOfFile,
    EndOfRow,
    Delta(u8, u8),
    Absolute(u8, Vec<u8>),
    PixelRun(u8, u8),
}

struct RLEInsnIterator<'a, R: 'a + Read> {
    r: &'a mut R,
    image_type: ImageType,
}

impl<'a, R: Read> Iterator for RLEInsnIterator<'a, R> {
    type Item = RLEInsn;

    fn next(&mut self) -> Option<RLEInsn> {
        let control_byte = read_u8(self.r);

        match control_byte {
            RLE_ESCAPE => {
                let op = read_u8(self.r);

                match op {
                    RLE_ESCAPE_EOL => Some(RLEInsn::EndOfRow),
                    RLE_ESCAPE_EOF => Some(RLEInsn::EndOfFile),
                    RLE_ESCAPE_DELTA => {
                        let xdelta = read_u8(self.r);
                        let ydelta = read_u8(self.r);
                        Some(RLEInsn::Delta(xdelta, ydelta))
                    }
                    _ => {
                        let mut length = op as usize;
                        if self.image_type == ImageType::RLE4 {
                            length = (length + 1) / 2;
                        }
                        length += length & 1;
                        let mut buffer = vec![0; length];
                        match self.r.read_exact(&mut buffer) {
                            Ok(()) => Some(RLEInsn::Absolute(op, buffer)),
                            Err(_) => None,
                        }
                    }
                }
            }
            _ => match read_u8(self.r) {
                palette_index => Some(RLEInsn::PixelRun(control_byte, palette_index)),
                _ => None,
            },
        }
    }
}



pub struct BmpDecoder<R> {
    reader: R,

    bmp_header_type: BmpHeaderType,

    width: i32,
    height: i32,
    data_offset: u64,
    top_down: bool,
    no_file_header: bool,
    add_alpha_channel: bool,
    has_loaded_metadata: bool,
    image_type: ImageType,

    bit_count: u16,
    colors_used: u32,
    palette: Option<Vec<(u8, u8, u8)>>,
    bitfields: Option<Bitfields>,
}


impl<R: Read + Seek> BmpDecoder<R> {
    /// Create a new decoder that decodes from the stream ```r```
    pub fn new(buffer: R) -> Self {
        let mut decoder = Self {
            reader: buffer,

            bmp_header_type: BmpHeaderType::Info,

            width: 0,
            height: 0,
            data_offset: 0,
            top_down: false,
            no_file_header: false,
            add_alpha_channel: false,
            has_loaded_metadata: false,
            image_type: ImageType::Palette,

            bit_count: 0,
            colors_used: 0,
            palette: None,
            bitfields: None,
        };
        decoder.read_metadata();
        decoder
    }
    
    fn num_channels(&self) -> usize {
        if self.add_alpha_channel {
            4
        } else {
            3
        }
    }

    fn bytes_per_color(&self) -> usize {
        match self.bmp_header_type {
            BmpHeaderType::Core => 3,
            _ => 4,
        }
    }

    fn get_palette_size(&mut self) -> usize {
        match self.colors_used {
            0 => 1 << self.bit_count,
            _ => {
                if self.colors_used > 1 << self.bit_count {
                    panic!("Exceeded maximum palette size");
                }
                self.colors_used as usize
            }
        }
    }

    fn read_metadata(&mut self) {
        if !self.has_loaded_metadata {
            self.read_file_header();
            
            let bmp_header_offset = self.reader.seek(SeekFrom::Current(0)).unwrap();
            let bmp_header_size = read_u32_le(&mut self.reader);
            let bmp_header_end = bmp_header_offset + u64::from(bmp_header_size);
            
            self.bmp_header_type = match bmp_header_size {
                BITMAPCOREHEADER_SIZE => BmpHeaderType::Core,
                BITMAPINFOHEADER_SIZE => BmpHeaderType::Info,
                BITMAPV2HEADER_SIZE => BmpHeaderType::V2,
                BITMAPV3HEADER_SIZE => BmpHeaderType::V3,
                BITMAPV4HEADER_SIZE => BmpHeaderType::V4,
                BITMAPV5HEADER_SIZE => BmpHeaderType::V5,
                _ => panic!("Invalid BMP header size"),
            };
            
            match self.bmp_header_type {
                BmpHeaderType::Core => {
                    self.read_bitmap_core_header();
                }
                BmpHeaderType::Info | BmpHeaderType::V2 | BmpHeaderType::V3 | BmpHeaderType::V4 | BmpHeaderType::V5 => {
                    self.read_bitmap_info_header();
                }
            };
            
            match self.image_type {
                ImageType::Bitfields16 | ImageType::Bitfields32 => self.read_bitmasks(),
                _ => {},
            };
            
            self.reader.seek(SeekFrom::Start(bmp_header_end));

            match self.image_type {
                ImageType::Palette | ImageType::RLE4 | ImageType::RLE8 => self.read_palette(),
                _ => {}
            };

            if self.no_file_header {
                // Use the offset of the end of metadata instead of reading a BMP file header.
                self.data_offset = self.reader.seek(SeekFrom::Current(0)).unwrap();
            }

            self.has_loaded_metadata = true;
        }
    }

    fn read_file_header(&mut self) {        
        if self.no_file_header {
            return;
        }        
        let mut signature = [0; 2];
        self.reader.read_exact(&mut signature);        

        if signature != b"BM"[..] {
            panic!("Invalid signature in BMP file header");
        }
        
        let filesize = read_u32_le(&mut self.reader);
        let reserved = read_u32_le(&mut self.reader);

        self.data_offset = u64::from(read_u32_le(&mut self.reader));
    }

    fn read_bitmap_core_header(&mut self) {
        self.width = i32::from(read_u16_le(&mut self.reader));
        self.height = i32::from(read_u16_le(&mut self.reader));

        if read_u16_le(&mut self.reader) != 1 {
            panic!("More than one plane in BMP file header");
        }
        
        self.bit_count = read_u16_le(&mut self.reader);
        self.image_type = match self.bit_count {
            1 | 4 | 8 => ImageType::Palette,
            24 => ImageType::RGB24,
            _ => panic!("Invalid channel width in BMP file header"),
        };
    }
    
    fn read_bitmap_info_header(&mut self) {
        self.width = read_i32_le(&mut self.reader);
        self.height = read_i32_le(&mut self.reader);

        // Width can not be negative
        if self.width < 0 || self.width > MAX_WIDTH_HEIGHT || self.height > MAX_WIDTH_HEIGHT {
            panic!("Invalid width in BMP file header");
        }
        if self.height == i32::min_value() {
            panic!("Invalid height in BMP file header");
        }

        if self.height < 0 {
            self.height *= -1;
            self.top_down = true;
        }
        
        if read_u16_le(&mut self.reader) != 1 {
            panic!("More than one plane in BMP file header");
        }
        self.bit_count = read_u16_le(&mut self.reader);
        let image_type_u32 = read_u32_le(&mut self.reader);
        if self.top_down && image_type_u32 != 0 && image_type_u32 != 3 {
            panic!("Top down images cannot be compressed");
        }
        
        self.image_type = match image_type_u32 {
            0 => match self.bit_count {
                1 | 2 | 4 | 8 => ImageType::Palette,
                16 => ImageType::RGB16,
                24 => ImageType::RGB24,
                32 if self.add_alpha_channel => ImageType::RGBA32,
                32 => ImageType::RGB32,
                _ =>  panic!("Invalid channel width in BMP file header"),
            },
            1 => match self.bit_count {
                8 => ImageType::RLE8,
                _ =>  panic!("Invalid channel width in BMP file header"),
            },
            2 => match self.bit_count {
                4 => ImageType::RLE4,
                _ =>  panic!("Invalid channel width in BMP file header"),
            },
            3 => match self.bit_count {
                16 => ImageType::Bitfields16,
                32 => ImageType::Bitfields32,
                _ =>  panic!("Invalid channel width in BMP file header"),
            },
            4 => panic!("JPG compression not supported in BMP file header"),
            5 => panic!("PNG compression not supported in BMP file header"),
            11 | 12 | 13 => panic!("CMYK format not supported in BMP file header"),
            _ => panic!("Unknown compression not supported in BMP file header"),
        };

        let _data_size = read_u32_le(&mut self.reader);
        let _printing_width = read_u32_le(&mut self.reader);
        let _printing_height = read_u32_le(&mut self.reader);
        
        self.colors_used = read_u32_le(&mut self.reader);
        
        let _important_colors = read_u32_le(&mut self.reader);
    }

    fn read_bitmasks(&mut self) {
        let r_mask = read_u32_le(&mut self.reader);
        let g_mask = read_u32_le(&mut self.reader);
        let b_mask = read_u32_le(&mut self.reader);

        let a_mask = match self.bmp_header_type {
            BmpHeaderType::V3 | BmpHeaderType::V4 | BmpHeaderType::V5 => {
                read_u32_le(&mut self.reader)
            }
            _ => 0,
        };

        self.bitfields = match self.image_type {
            ImageType::Bitfields16 => Some(Bitfields::from_mask(r_mask, g_mask, b_mask, a_mask, 16)),
            ImageType::Bitfields32 => Some(Bitfields::from_mask(r_mask, g_mask, b_mask, a_mask, 32)),
            _ => None,
        };

        if self.bitfields.is_some() && a_mask != 0 {
            self.add_alpha_channel = true;
        }
    }
    
    fn read_palette(&mut self) {
        const MAX_PALETTE_SIZE: usize = 256; // Palette indices are u8.

        let bytes_per_color = self.bytes_per_color();
        let palette_size = self.get_palette_size();
        let max_length = MAX_PALETTE_SIZE * bytes_per_color;

        let length = palette_size * bytes_per_color;
        let mut buf = Vec::with_capacity(max_length);

        buf.resize(cmp::min(length, max_length), 0);
        let _ = self.reader.by_ref().read_exact(&mut buf);

        match length.cmp(&max_length) {
            Ordering::Greater => { let _ = self.reader.seek(SeekFrom::Current((length - max_length) as i64)); ()},
            Ordering::Less => { buf.resize(max_length, 0); () },
            Ordering::Equal => (),
        }

        let p: Vec<(u8, u8, u8)> = (0..MAX_PALETTE_SIZE)
            .map(|i| {
                let b = buf[bytes_per_color * i];
                let g = buf[bytes_per_color * i + 1];
                let r = buf[bytes_per_color * i + 2];
                (r, g, b)
            })
            .collect();

        self.palette = Some(p);
    }

    fn create_pixel_data(&self) -> Vec<u8> {
        let row_width = self.num_channels() * self.width as usize;
        let max_pixels = self.num_channels() * MAX_INITIAL_PIXELS;
        let max_starting_size = max_pixels + row_width - (max_pixels % row_width);
        vec![0xFF; cmp::min(row_width * self.height as usize, max_starting_size)]
    }

    fn rows<'a>(&self, pixel_data: &'a mut [u8]) -> RowIterator<'a> {
        let stride = self.width as usize * self.num_channels();
        if self.top_down {
            RowIterator {
                chunks: Chunker::FromTop(pixel_data.chunks_mut(stride)),
            }
        } else {
            RowIterator {
                chunks: Chunker::FromBottom(pixel_data.chunks_mut(stride).rev()),
            }
        }
    }
    
    fn read_palettized_pixel_data(&mut self) -> Vec<u8> {
        let mut pixel_data = self.create_pixel_data();
        let num_channels = self.num_channels();
        let row_byte_length = ((i32::from(self.bit_count) * self.width + 31) / 32 * 4) as usize;
        let mut indices = vec![0; row_byte_length];
        let palette = self.palette.as_ref().unwrap();
        let bit_count = self.bit_count;
        let reader = &mut self.reader;
        let width = self.width as usize;

        reader.seek(SeekFrom::Start(self.data_offset));

        with_rows(
            &mut pixel_data,
            self.width,
            self.height,
            num_channels,
            self.top_down,
            |row| {
                reader.read_exact(&mut indices)?;
                let mut pixel_iter = row.chunks_mut(num_channels);
                match bit_count {
                    1 => {
                        set_1bit_pixel_run(&mut pixel_iter, palette, indices.iter());
                    }
                    2 => {
                        set_2bit_pixel_run(&mut pixel_iter, palette, indices.iter(), width);
                    }
                    4 => {
                        set_4bit_pixel_run(&mut pixel_iter, palette, indices.iter(), width);
                    }
                    8 => {
                        set_8bit_pixel_run(&mut pixel_iter, palette, indices.iter(), width);
                    }
                    _ => panic!(),
                };
                Ok(())
            },
        );

        pixel_data
    }

    fn read_16_bit_pixel_data(&mut self, bitfields: Option<&Bitfields>) -> Vec<u8> {
        let mut pixel_data = self.create_pixel_data();
        let num_channels = self.num_channels();
        let row_padding_len = self.width as usize % 2 * 2;
        let row_padding = &mut [0; 2][..row_padding_len];
        let bitfields = match bitfields {
            Some(b) => b,
            None => self.bitfields.as_ref().unwrap(),
        };
        let reader = &mut self.reader;

        reader.seek(SeekFrom::Start(self.data_offset));

        with_rows(
            &mut pixel_data,
            self.width,
            self.height,
            num_channels,
            self.top_down,
            |row| {
                for pixel in row.chunks_mut(num_channels) {
                    let data = u32::from(read_u16_le(reader));

                    pixel[0] = bitfields.r.read(data);
                    pixel[1] = bitfields.g.read(data);
                    pixel[2] = bitfields.b.read(data);
                    if num_channels == 4 {
                        pixel[3] = bitfields.a.read(data);
                    }
                }
                reader.read_exact(row_padding)
            },
        );

        pixel_data
    }

    /// Read image data from a reader in 32-bit formats that use bitfields.
    fn read_32_bit_pixel_data(&mut self) -> Vec<u8> {
        let mut pixel_data = self.create_pixel_data();
        let num_channels = self.num_channels();

        let bitfields = self.bitfields.as_ref().unwrap();

        let reader = &mut self.reader;
        reader.seek(SeekFrom::Start(self.data_offset));

        with_rows(
            &mut pixel_data,
            self.width,
            self.height,
            num_channels,
            self.top_down,
            |row| {
                for pixel in row.chunks_mut(num_channels) {
                    let data = read_u32_le(reader);

                    pixel[0] = bitfields.r.read(data);
                    pixel[1] = bitfields.g.read(data);
                    pixel[2] = bitfields.b.read(data);
                    if num_channels == 4 {
                        pixel[3] = bitfields.a.read(data);
                    }
                }
                Ok(())
            },
        );

        pixel_data
    }

    /// Read image data from a reader where the colours are stored as 8-bit values (24 or 32-bit).
    fn read_full_byte_pixel_data(&mut self, format: &FormatFullBytes) -> Vec<u8> {
        let mut pixel_data = self.create_pixel_data();
        let num_channels = self.num_channels();
        let row_padding_len = match *format {
            FormatFullBytes::RGB24 => (4 - (self.width as usize * 3) % 4) % 4,
            _ => 0,
        };
        let row_padding = &mut [0; 4][..row_padding_len];

        self.reader.seek(SeekFrom::Start(self.data_offset));

        let reader = &mut self.reader;

        with_rows(
            &mut pixel_data,
            self.width,
            self.height,
            num_channels,
            self.top_down,
            |row| {
                for pixel in row.chunks_mut(num_channels) {
                    if *format == FormatFullBytes::Format888 {
                        read_u8(reader);
                    }

                    // Read the colour values (b, g, r).
                    // Reading 3 bytes and reversing them is significantly faster than reading one
                    // at a time.
                    reader.read_exact(&mut pixel[0..3]);
                    pixel[0..3].reverse();

                    if *format == FormatFullBytes::RGB32 {
                        read_u8(reader);
                    }

                    // Read the alpha channel if present
                    if *format == FormatFullBytes::RGBA32 {
                        reader.read_exact(&mut pixel[3..4])?;
                    }
                }
                reader.read_exact(row_padding)
            },
        );

        pixel_data
    }

    fn read_rle_data(&mut self, image_type: ImageType) -> Vec<u8> {
        // Seek to the start of the actual image data.
        self.reader.seek(SeekFrom::Start(self.data_offset));

        let full_image_size =
            num_bytes(self.width, self.height, self.num_channels()).ok_or_else(|| {
                panic!("Image format or kind or dimensions unsupported");
            }).unwrap();
        let mut pixel_data = self.create_pixel_data();
        let (skip_pixels, skip_rows, eof_hit) =
            self.read_rle_data_step(&mut pixel_data, image_type, 0, 0);
        // Extend the buffer if there is still data left.
        // If eof_hit is true, it means that we hit an end-of-file marker in the last step and
        // we won't extend the buffer further to avoid small files with a large specified size causing memory issues.
        // This is only a rudimentary check, a file could still create a large buffer, but the
        // file would now have to at least have some data in it.
        if pixel_data.len() < full_image_size && !eof_hit {
            let new = extend_buffer(&mut pixel_data, full_image_size, true);
            self.read_rle_data_step(new, image_type, skip_pixels, skip_rows);
        }
        pixel_data
    }

    fn read_rle_data_step(
        &mut self,
        mut pixel_data: &mut [u8],
        image_type: ImageType,
        skip_pixels: u8,
        skip_rows: u8,
    ) -> (u8, u8, bool) {
        let num_channels = self.num_channels();

        let mut delta_rows_left = 0;
        let mut delta_pixels_left = skip_pixels;
        let mut eof_hit = false;

        // Scope the borrowing of pixel_data by the row iterator.
        {
            // Handling deltas in the RLE scheme means that we need to manually
            // iterate through rows and pixels.  Even if we didn't have to handle
            // deltas, we have to ensure that a single runlength doesn't straddle
            // two rows.
            let mut row_iter = self.rows(&mut pixel_data);
            // If we have previously hit a delta value,
            // blank the rows that are to be skipped.
            blank_bytes((&mut row_iter).take(skip_rows.into()));
            let mut insns_iter = RLEInsnIterator {
                r: &mut self.reader,
                image_type,
            };
            let p = self.palette.as_ref().unwrap();

            'row_loop: while let Some(row) = row_iter.next() {
                let mut pixel_iter = row.chunks_mut(num_channels);
                // Blank delta skipped pixels if any.
                blank_bytes((&mut pixel_iter).take(delta_pixels_left.into()));
                delta_pixels_left = 0;

                'rle_loop: loop {
                    if let Some(insn) = insns_iter.next() {
                        match insn {
                            RLEInsn::EndOfFile => {
                                blank_bytes(pixel_iter);
                                blank_bytes(row_iter);
                                eof_hit = true;
                                break 'row_loop;
                            }
                            RLEInsn::EndOfRow => {
                                blank_bytes(pixel_iter);
                                break 'rle_loop;
                            }
                            RLEInsn::Delta(x_delta, y_delta) => {
                                if y_delta > 0 {
                                    for n in 1..y_delta {
                                        if let Some(row) = row_iter.next() {
                                            // The msdn site on bitmap compression doesn't specify
                                            // what happens to the values skipped when encountering
                                            // a delta code, however IE and the windows image
                                            // preview seems to replace them with black pixels,
                                            // so we stick to that.
                                            for b in row {
                                                *b = 0;
                                            }
                                        } else {
                                            delta_pixels_left = x_delta;
                                            // We've reached the end of the buffer.
                                            delta_rows_left = y_delta - n;
                                            break 'row_loop;
                                        }
                                    }
                                }

                                for _ in 0..x_delta {
                                    if let Some(pixel) = pixel_iter.next() {
                                        for b in pixel {
                                            *b = 0;
                                        }
                                    } else {
                                        // We can't go any further in this row.
                                        break;
                                    }
                                }
                            }
                            RLEInsn::Absolute(length, indices) => {
                                // Absolute mode cannot span rows, so if we run
                                // out of pixels to process, we should stop
                                // processing the image.
                                match image_type {
                                    ImageType::RLE8 => {
                                        if !set_8bit_pixel_run(
                                            &mut pixel_iter,
                                            p,
                                            indices.iter(),
                                            length as usize,
                                        ) {
                                            break 'row_loop;
                                        }
                                    }
                                    ImageType::RLE4 => {
                                        if !set_4bit_pixel_run(
                                            &mut pixel_iter,
                                            p,
                                            indices.iter(),
                                            length as usize,
                                        ) {
                                            break 'row_loop;
                                        }
                                    }
                                    _ => panic!(),
                                }
                            }
                            RLEInsn::PixelRun(n_pixels, palette_index) => {
                                // A pixel run isn't allowed to span rows, but we
                                // simply continue on to the next row if we run
                                // out of pixels to set.
                                match image_type {
                                    ImageType::RLE8 => {
                                        if !set_8bit_pixel_run(
                                            &mut pixel_iter,
                                            p,
                                            repeat(&palette_index),
                                            n_pixels as usize,
                                        ) {
                                            break 'rle_loop;
                                        }
                                    }
                                    ImageType::RLE4 => {
                                        if !set_4bit_pixel_run(
                                            &mut pixel_iter,
                                            p,
                                            repeat(&palette_index),
                                            n_pixels as usize,
                                        ) {
                                            break 'rle_loop;
                                        }
                                    }
                                    _ => panic!(),
                                }
                            }
                        }
                    } 
                    else {
                        panic!("Out of data and we still had rows to fill")
                    }
                }
            }
        }
        (delta_pixels_left, delta_rows_left, eof_hit)
    }

    
    fn read_image_data(&mut self, buf: &mut [u8]) {
        let data = match self.image_type {
            ImageType::Palette => self.read_palettized_pixel_data(),
            ImageType::RGB16 => self.read_16_bit_pixel_data(Some(&R5_G5_B5_COLOR_MASK)),
            ImageType::RGB24 => self.read_full_byte_pixel_data(&FormatFullBytes::RGB24),
            ImageType::RGB32 => self.read_full_byte_pixel_data(&FormatFullBytes::RGB32),
            ImageType::RGBA32 => self.read_full_byte_pixel_data(&FormatFullBytes::RGBA32),
            ImageType::RLE8 => self.read_rle_data(ImageType::RLE8),
            ImageType::RLE4 => self.read_rle_data(ImageType::RLE4),
            ImageType::Bitfields16 => match self.bitfields {
                Some(_) => self.read_16_bit_pixel_data(None),
                None => panic!("Bitfield mask missing 16bit"),
            },
            ImageType::Bitfields32 => match self.bitfields {
                Some(R8_G8_B8_COLOR_MASK) => {
                    self.read_full_byte_pixel_data(&FormatFullBytes::Format888)
                }
                Some(_) => self.read_32_bit_pixel_data(),
                None => panic!("Bitfield mask missing 32bit"),
            },
        };

        buf.copy_from_slice(&data);
    }
}

pub struct BmpReader<R>(Cursor<Vec<u8>>, PhantomData<R>);
impl<R> Read for BmpReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> ::std::io::Result<usize> {
        self.0.read(buf)
    }
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> ::std::io::Result<usize> {
        if self.0.position() == 0 && buf.is_empty() {
            ::std::mem::swap(buf, self.0.get_mut());
            Ok(buf.len())
        } else {
            self.0.read_to_end(buf)
        }
    }
}

impl<'a, R: 'a + Read + Seek> ImageDecoder<'a> for BmpDecoder<R> {
    type Reader = BmpReader<R>;

    fn dimensions(&self) -> (u32, u32) {
        (self.width as u32, self.height as u32)
    }

    fn color_type(&self) -> ColorType {
        if self.add_alpha_channel {
            ColorType::Rgba8
        } else {
            ColorType::Rgb8
        }
    }

    fn read_image(mut self, buf: &mut [u8]) {
        assert_eq!(u64::try_from(buf.len()), Ok(self.total_bytes()));
        self.read_image_data(buf)
    }
}
