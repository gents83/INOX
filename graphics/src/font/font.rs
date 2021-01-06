
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
        let mut glyphs: Vec<Glyph> = Vec::new();
        for id in 0..glyph_count 
        {
            let glyph_id = GlyphId(id as u16);  
            glyphs.push(Glyph::create(glyph_id, &face) );   
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
        let num_glyphs:u32 = self.glyphs.len() as _;
        let cell_size:u32 = (((size*size) as u32 / num_glyphs) as f64).sqrt().ceil() as u32;
        
        let mut image = GrayImage::new(size as _, size as _);

        let mut row: u32 = 0;
        let mut column: u32 = 0;
        for g in self.glyphs.iter_mut()
        {
            let mut starting_x = column * cell_size;
            if (starting_x + cell_size) > size as _ {
                starting_x = 0;
                column = 0;
                row += 1;
            }
            let starting_y = row * cell_size;
            if (starting_y + cell_size) > size as _ {
                break;
            }
        
            g.render(&face, 
                cell_size as _, 
                cell_size as _,  
                |x, y, alpha| {
                    image.put_pixel(starting_x + x, starting_y + y, Luma([(alpha * 255.0).round() as u8; 1]))
                }
            );
            
            column += 1;
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