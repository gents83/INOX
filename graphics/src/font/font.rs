
use std::{collections::HashMap, num::NonZeroU16};
use image::*;
use nrg_math::*;
use ttf_parser::*;
use super::glyph::*;

const FONT_COLUMNS: u32 = 24;

pub struct Font {
    image: GrayImage,
    glyphs: Vec<Glyph>,
    char_to_glyph: HashMap<u32, NonZeroU16>,
    units_per_em: f32,
}

impl Font {
    pub fn new(filepath: &str, size: usize) -> Self {
        let font_data = ::std::fs::read(filepath).unwrap();

        let face = Face::from_slice(font_data.as_slice(), 0).unwrap();

        let units_per_em = face.units_per_em().unwrap_or(1000) as f32;

        // Collect all the unique codepoint to glyph mappings.
        let mut char_to_glyph = HashMap::new();
        for subtable in face.character_mapping_subtables() {
            subtable.codepoints(|codepoint| {
                let mapping = match subtable.glyph_index(codepoint) {
                    Some(id) => id.0,
                    None => 0,
                };
                // Zero is a valid value for missing glyphs, so even if a mapping is zero, the
                // result is desireable.
                char_to_glyph.insert(codepoint, unsafe { NonZeroU16::new_unchecked(mapping) });
            });
        }
        
        let glyph_count = face.number_of_glyphs() as usize;
        let mut glyphs: Vec<Glyph> = Vec::with_capacity(glyph_count as usize);
        unsafe {
            glyphs.set_len(glyph_count as usize);
        }
        for id in 0..glyph_count 
        {
            let glyph_id = GlyphId(id as u16);  
            glyphs[id] = Glyph::create(glyph_id, &face);   
        }
        let mut font = Self {
            image: GrayImage::default(), 
            glyphs,
            char_to_glyph,
            units_per_em,
        };
        font.create_texture(&face, size);
        font
    }

    pub fn get_bitmap(&self) -> &GrayImage {
        &self.image
    }

    fn create_texture(&mut self, face: &Face, size: usize) {
        let num_glyphs:u32 = face.number_of_glyphs() as _;
        let rounded_to_next_pow2 = round_to_next_pow2(num_glyphs);
        let cell_size:u32 = (((size*size) as u32 / rounded_to_next_pow2) as f64).sqrt().ceil() as u32;
        let num_columns: u32 = size as u32 / cell_size ;
        
        let mut image = GrayImage::new(size as _, size as _);

        let mut row: u32 = 0;
        let mut column: u32 = 0;
        for id in 0..num_glyphs 
        {
            let starting_x = column * cell_size;
            let starting_y = row * cell_size;
        
            let g = &self.glyphs[id as usize];
            let mut glyph_data: Vec<f32> = vec![0.0; g.width as usize * g.height as usize + 4];

            self.glyphs[id as usize].render(&face, 
                cell_size as _, 
                cell_size as _,  
                &mut glyph_data,
                |x, y, alpha| {
                    image.put_pixel(starting_x + x, starting_y + y, Luma([(alpha * 255.0).round() as u8; 1]))
                }
            );
            
            column += 1;
            if column == num_columns {
                column = 0;
                row += 1;
            }
        }

        self.image = image;
    }
    
    #[inline]
    fn get_glyph_index(&self, character: char) -> usize {
        unsafe {
            ::std::mem::transmute::<Option<NonZeroU16>, u16>(self.char_to_glyph.get(&(character as u32)).copied())
                as usize
        }
    }
}