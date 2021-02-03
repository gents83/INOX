
use std::{collections::HashMap, num::NonZeroU16, path::Path};
use image::*;
use ttf_parser::*;
use nrg_math::*;
use crate::api::data_formats::*;
use crate::api::device::*;
use crate::api::material::*;
use crate::api::mesh::*;
use crate::api::pipeline::*;

use super::glyph::*;
use super::hershey::*;

const DEFAULT_FONT_COUNT: u8 = 255;
const DEFAULT_FONT_TEXTURE_SIZE: usize = 1024;

pub struct Font {
    image: DynamicImage,
    metrics: Metrics,
    glyphs: Vec<Glyph>,
    char_to_glyph: HashMap<u32, NonZeroU16>,
    material: Material,
    device: Device,
}

impl Font {
    #[inline]
    pub fn new(device:&Device, ui_pipeline: &Pipeline, filepath: &str) -> Self {
        let path = Path::new(filepath);
        match path.extension().unwrap().to_str() {
            Some("jhf") => Font::new_jhf_font(&device, &ui_pipeline, &filepath),
            _ => Font::new_ttf_font(&device, &ui_pipeline, &filepath),
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
    fn new_jhf_font(device:&Device, ui_pipeline: &Pipeline, filepath: &str) -> Self {
        
        let _font_data = ::std::fs::read(filepath).unwrap();
        let _font = HersheyFont::from_data(_font_data.as_slice());
        //NOT SUPPORTED YET - returning normal ttf font
        Font::new_ttf_font(&device, &ui_pipeline, &filepath)
    }

    fn new_ttf_font(device:&Device, ui_pipeline: &Pipeline, filepath: &str) -> Self {
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
        let mut max_glyph_metrics = Metrics::default();
        for character in 0..DEFAULT_FONT_COUNT 
        {
            let glyph_id = GlyphId(character as u16);  
            let metrics = Glyph::compute_metrics(glyph_id, &face);
            max_glyph_metrics = max_glyph_metrics.max(&metrics);
        }

        let mut glyphs: Vec<Glyph> = Vec::new();
        for character in 0..DEFAULT_FONT_COUNT 
        {
            let glyph_id = GlyphId(character as u16);  
            glyphs.push(Glyph::create(glyph_id, &face, &max_glyph_metrics) );   
        }

        let mut font = Self {
            image: DynamicImage::new_rgb8(DEFAULT_FONT_TEXTURE_SIZE as _,DEFAULT_FONT_TEXTURE_SIZE as _), 
            metrics: max_glyph_metrics,
            glyphs,
            char_to_glyph,
            material: Material::create(&device, &ui_pipeline),
            device: device.clone(),
        };

        font.create_texture(DEFAULT_FONT_TEXTURE_SIZE);
        font
    }

    fn create_texture(&mut self, size: usize) {
        let num_glyphs:u32 = self.glyphs.len() as _;
        let cell_size:u32 = (((size*size) as u32 / num_glyphs) as f64).sqrt().ceil() as u32;
        
        let mut row: u32 = 0;
        let mut column: u32 = 0;
        let image = &mut self.image;
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
        
            g.render(|x, y, alpha| {
                    let v = (alpha * 255.0).round() as u8;
                    image.put_pixel(starting_x + x, starting_y + y, Rgba([v; 4]))
                }
            );
            
            g.texture_coord = [(starting_x + cell_size) as f32 / size as f32, 
                               (starting_y) as f32 / size as f32,
                               (starting_x) as f32 / size as f32, 
                               (starting_y + cell_size) as f32 / size as f32].into();
            
            column += 1;
        }
        
        self.material.add_texture_from_image(&self.image);
    }

    pub fn create_text(&self, text: &str, position: Vector2f, scale: f32) -> Mesh {
        const VERTICES_COUNT: usize = 4;
        const INDICES_COUNT: usize = 6;

        let mut vertex_data: Vec<VertexData> = Vec::new();
        let mut indices_data: Vec<u32> = Vec::new();
        let mut prev_pos = position;
        for (i, c) in text.as_bytes().iter().enumerate() {
            let id = self.get_glyph_index(*c as _);
            let g = &self.glyphs[id];
            let width = (DEFAULT_FONT_GLYPH_SIZE as f32 / self.metrics.width) * scale;
            let heigth = (DEFAULT_FONT_GLYPH_SIZE as f32 / self.metrics.height) * scale;
            let(mut vertices, mut indices) = Mesh::create_quad(Vector4f::new(prev_pos.x, prev_pos.y, prev_pos.x+width, prev_pos.y+heigth), 
                                                                                g.texture_coord,
                                                                                Some(i * VERTICES_COUNT));
            
            assert_eq!(vertices.len(), VERTICES_COUNT, "Trying to create a quad with more than {} vertices", VERTICES_COUNT);
            assert_eq!(indices.len(), INDICES_COUNT, "Trying to create a quad with more than {} indices", INDICES_COUNT);
            vertex_data.append(&mut vertices);
            indices_data.append(&mut indices);

            prev_pos.x += width * scale;
        }

        let mut mesh = Mesh::create(&self.device);
        mesh.set_vertices(&vertex_data)
            .set_indices(&indices_data);

        mesh
    }
    
    #[inline]
    fn get_glyph_index(&self, character: char) -> usize {
        Font::get_glyph_index_from_map(&self.char_to_glyph, character)
    }

    #[inline]
    fn get_glyph_index_from_map(char_to_glyph: &HashMap<u32, NonZeroU16>, character: char) -> usize {
        unsafe {
            ::std::mem::transmute::<Option<NonZeroU16>, u16>(char_to_glyph.get(&(character as u32)).copied())
                as usize
        }
    }
}