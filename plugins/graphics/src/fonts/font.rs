use crate::common::data_formats::*;
use image::*;
use nrg_math::*;
use std::{collections::HashMap, num::NonZeroU16, path::PathBuf};
use ttf_parser::*;

use super::glyph::*;

const DEFAULT_FONT_COUNT: u8 = 255;
const DEFAULT_FONT_TEXTURE_SIZE: usize = 1024;

pub struct Font {
    image: DynamicImage,
    metrics: Metrics,
    glyphs: Vec<Glyph>,
    char_to_glyph: HashMap<u32, NonZeroU16>,
    text_data: Vec<TextData>,
}

#[derive(Clone)]
struct TextData {
    text: String,
    position: Vector2f,
    scale: f32,
    color: Vector3f,
}

impl Font {
    #[inline]
    pub fn new(filepath: PathBuf) -> Self {
        Font::new_ttf_font(filepath)
    }

    pub fn add_text(
        &mut self,
        text: &str,
        position: Vector2f,
        scale: f32,
        color: Vector3f,
    ) -> MeshData {
        let data = TextData {
            text: String::from(text),
            position,
            scale,
            color,
        };
        let mesh_data = self.create_mesh_from_text(&data);
        self.text_data.push(data);
        mesh_data
    }

    pub fn clear(&mut self) {
        self.text_data.clear();
    }

    pub fn get_bitmap(&self) -> &DynamicImage {
        &self.image
    }
}

impl Font {
    fn new_ttf_font(filepath: PathBuf) -> Self {
        let font_data = ::std::fs::read(filepath.as_path()).unwrap();

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
        for character in 0..DEFAULT_FONT_COUNT {
            let glyph_id = GlyphId(character as u16);
            let metrics = Glyph::compute_metrics(glyph_id, &face);
            max_glyph_metrics = max_glyph_metrics.max(&metrics);
        }

        let mut glyphs: Vec<Glyph> = Vec::new();
        for character in 0..DEFAULT_FONT_COUNT {
            let glyph_id = GlyphId(character as u16);
            glyphs.push(Glyph::create(glyph_id, &face, &max_glyph_metrics));
        }

        let mut font = Self {
            image: DynamicImage::new_rgb8(
                DEFAULT_FONT_TEXTURE_SIZE as _,
                DEFAULT_FONT_TEXTURE_SIZE as _,
            ),
            metrics: max_glyph_metrics,
            glyphs,
            char_to_glyph,
            text_data: Vec::new(),
        };

        font.create_texture(DEFAULT_FONT_TEXTURE_SIZE);
        font
    }

    fn create_texture(&mut self, size: usize) {
        let num_glyphs: u32 = self.glyphs.len() as _;
        let cell_size: u32 = (((size * size) as u32 / num_glyphs) as f64).sqrt().ceil() as u32;

        let mut row: u32 = 0;
        let mut column: u32 = 0;
        let image = &mut self.image;
        for g in self.glyphs.iter_mut() {
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
            });

            g.texture_coord = [
                (starting_x + cell_size) as f32 / size as f32,
                (starting_y) as f32 / size as f32,
                (starting_x) as f32 / size as f32,
                (starting_y + cell_size) as f32 / size as f32,
            ]
            .into();

            column += 1;
        }
    }

    fn create_mesh_from_text(&mut self, text_data: &TextData) -> MeshData {
        let mut mesh_data = MeshData::default();
        const VERTICES_COUNT: usize = 4;

        let mut prev_pos = text_data.position;
        let width = (DEFAULT_FONT_GLYPH_SIZE as f32 / self.metrics.width) * text_data.scale;
        let heigth = (DEFAULT_FONT_GLYPH_SIZE as f32 / self.metrics.height) * text_data.scale;

        for (i, c) in text_data.text.as_bytes().iter().enumerate() {
            if *c == b'\n' {
                prev_pos.x = text_data.position.x - width * text_data.scale;
                prev_pos.y += heigth * text_data.scale;
            }

            let id = self.get_glyph_index(*c as _);
            let g = &self.glyphs[id];
            mesh_data.add_quad(
                Vector4f::new(
                    prev_pos.x,
                    prev_pos.y,
                    prev_pos.x + width,
                    prev_pos.y + heigth,
                ),
                g.texture_coord,
                Some(i * VERTICES_COUNT),
            );

            prev_pos.x += width * text_data.scale;
        }

        mesh_data.set_vertex_color(text_data.color);
        mesh_data
    }

    #[inline]
    fn get_glyph_index(&self, character: char) -> usize {
        Font::get_glyph_index_from_map(&self.char_to_glyph, character)
    }

    #[inline]
    fn get_glyph_index_from_map(
        char_to_glyph: &HashMap<u32, NonZeroU16>,
        character: char,
    ) -> usize {
        unsafe {
            ::std::mem::transmute::<Option<NonZeroU16>, u16>(
                char_to_glyph.get(&(character as u32)).copied(),
            ) as usize
        }
    }
}
