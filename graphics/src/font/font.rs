
use std::{collections::HashMap, ffi::OsStr, num::NonZeroU16, path::Path};
use image::*;
use ttf_parser::*;
use nrg_math::*;
use crate::data_formats::*;
use crate::device::*;
use crate::material::*;
use crate::mesh::*;
use crate::pipeline::*;

use super::glyph::*;
use super::hershey::*;

pub struct Font {
    image: DynamicImage,
    glyphs: Vec<Glyph>,
    char_to_glyph: HashMap<u32, NonZeroU16>,
    material: Material,
}

impl Font {
    #[inline]
    pub fn new(device:&Device, ui_pipeline: &Pipeline, filepath: &str, size: usize) -> Self {
        let path = Path::new(filepath);
        match path.extension().unwrap().to_str() {
            Some("jhf") => Font::new_jhf_font(&device, &ui_pipeline, &filepath, size),
            _ => Font::new_ttf_font(&device, &ui_pipeline, &filepath, size),
        }
    }

    pub fn get_material(&self) -> &Material {
        &self.material
    }

    pub fn get_bitmap(&self) -> &DynamicImage {
        &self.image
    }

}


impl Font{    
    fn new_jhf_font(device:&Device, ui_pipeline: &Pipeline, filepath: &str, size: usize) -> Self {
        
        let font_data = ::std::fs::read(filepath).unwrap();
        let font = HersheyFont::from_data(font_data.as_slice());

        Font::new_ttf_font(&device, &ui_pipeline, &filepath, size)
    }

    fn new_ttf_font(device:&Device, ui_pipeline: &Pipeline, filepath: &str, size: usize) -> Self {
        let font_data = ::std::fs::read(filepath).unwrap();

        let face = Face::from_slice(font_data.as_slice(), 0).unwrap();

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
            image: DynamicImage::new_rgb8(1,1), 
            glyphs,
            char_to_glyph,
            material: Material::create(&device, &ui_pipeline),
        };

        font.create_texture(&device, &face, size);
        font
    }

    fn create_texture(&mut self, device:&Device, face: &Face, size: usize) {
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
            
            g.texture_coord = [(starting_x + cell_size) as f32 / size as f32, 
                               (starting_y) as f32 / size as f32,
                               (starting_x) as f32 / size as f32, 
                               (starting_y + cell_size) as f32 / size as f32].into();
            
            column += 1;
        }

        self.image = DynamicImage::ImageRgb8(DynamicImage::ImageLuma8(image.clone()).into_rgb8());
        
        self.material.add_texture_from_image(&device, &self.image);
    }

    pub fn create_text(&self, device:&Device, text: &str, position: Vector2f, scale: f32) -> Mesh {
        const VERTICES_COUNT: usize = 4;
        const INDICES_COUNT: usize = 6;

        let mut vertex_data: Vec<VertexData> = Vec::new();
        let mut indices_data: Vec<u32> = Vec::new();
        let mut prev_pos = position;
        for (i, c) in text.as_bytes().iter().enumerate() {
            let id = self.get_glyph_index(*c as _);
            
            let(mut vertices, mut indices) = Mesh::create_quad(Vector4f::new(prev_pos.x, prev_pos.y, prev_pos.x+scale, prev_pos.y+scale), 
                                                                                self.glyphs[id].texture_coord,
                                                                                Some(i * VERTICES_COUNT));
            
            assert_eq!(vertices.len(), VERTICES_COUNT, "Trying to create a quad with more than {} vertices", VERTICES_COUNT);
            assert_eq!(indices.len(), INDICES_COUNT, "Trying to create a quad with more than {} indices", INDICES_COUNT);
            vertex_data.append(&mut vertices);
            indices_data.append(&mut indices);

            prev_pos.x += scale;
        }

        let mut mesh = Mesh::create();
        mesh.set_vertices(&device, &vertex_data)
            .set_indices(&device, &indices_data);

        mesh
    }
    
    #[inline]
    fn get_glyph_index(&self, character: char) -> usize {
        unsafe {
            ::std::mem::transmute::<Option<NonZeroU16>, u16>(self.char_to_glyph.get(&(character as u32)).copied())
                as usize
        }
    }
}