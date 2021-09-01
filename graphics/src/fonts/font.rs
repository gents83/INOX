use crate::common::data_formats::*;
use image::*;
use nrg_math::*;
use nrg_platform::*;
use std::{
    collections::HashMap,
    num::NonZeroU16,
    path::{Path, PathBuf},
};
use ttf_parser::*;

use super::glyph::*;

const DEFAULT_FONT_COUNT: u8 = 255;
pub const DEFAULT_FONT_TEXTURE_SIZE: usize = 1024;
//12pt = 16px = 1em = 100%
pub const FONT_PT_TO_PIXEL: f32 = DEFAULT_DPI / (72. * 2048.);

pub struct FontData {
    filepath: PathBuf,
    metrics: Metrics,
    glyphs: Vec<Glyph>,
    char_to_glyph: HashMap<u32, NonZeroU16>,
    image: DynamicImage,
}

#[derive(Clone)]
pub struct TextData {
    pub text: String,
    pub position: Vector2,
    pub scale: f32,
    pub color: Vector4,
    pub spacing: Vector2,
}

impl Default for FontData {
    fn default() -> Self {
        Self {
            filepath: PathBuf::new(),
            metrics: Metrics::default(),
            glyphs: Vec::new(),
            char_to_glyph: HashMap::new(),
            image: DynamicImage::new_rgb8(2, 2),
        }
    }
}

impl FontData {
    #[inline]
    pub fn new(filepath: &Path) -> Self {
        FontData::new_ttf_font(filepath)
    }

    pub fn add_text(
        &mut self,
        text: &str,
        position: Vector2,
        scale: f32,
        color: Vector4,
        spacing: Vector2,
    ) -> MeshData {
        let data = TextData {
            text: String::from(text),
            position,
            scale,
            color,
            spacing,
        };
        self.create_mesh_from_text(&data)
    }

    #[inline]
    pub fn get_filepath(&self) -> &PathBuf {
        &self.filepath
    }

    #[inline]
    pub fn get_metrics(&self) -> &Metrics {
        &self.metrics
    }

    #[inline]
    pub fn get_glyph_index(&self, character: char) -> usize {
        FontData::get_glyph_index_from_map(&self.char_to_glyph, character)
    }

    #[inline]
    pub fn get_glyph(&self, index: usize) -> &Glyph {
        if index >= self.glyphs.len() {
            &self.glyphs[0]
        } else {
            &self.glyphs[index]
        }
    }

    pub fn create_mesh_from_text(&self, text_data: &TextData) -> MeshData {
        let mut mesh_data = MeshData::default();
        const VERTICES_COUNT: usize = 4;

        let mut prev_pos = text_data.position;
        let size = FONT_PT_TO_PIXEL * text_data.scale;
        let spacing_x = FONT_PT_TO_PIXEL * text_data.spacing.x;
        let spacing_y = FONT_PT_TO_PIXEL * text_data.spacing.y;

        for (i, c) in text_data.text.as_bytes().iter().enumerate() {
            let id = self.get_glyph_index(*c as _);
            let g = &self.glyphs[id];
            mesh_data.add_quad(
                Vector4::new(prev_pos.x, prev_pos.y, prev_pos.x + size, prev_pos.y + size),
                0.0,
                g.texture_coord,
                Some(i * VERTICES_COUNT),
            );

            if *c == b'\n' {
                prev_pos.x = text_data.position.x;
                prev_pos.y += size + spacing_y;
            } else {
                prev_pos.x += size * 0.5 + spacing_x;
            }
        }

        mesh_data.set_vertex_color(text_data.color);
        mesh_data
    }
}

impl FontData {
    fn new_ttf_font(filepath: &Path) -> Self {
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

        let image = FontData::create_texture(&mut glyphs, &max_glyph_metrics);
        Self {
            filepath: PathBuf::from(filepath),
            metrics: max_glyph_metrics,
            glyphs,
            char_to_glyph,
            image,
        }
    }

    pub fn get_texture(&self) -> RgbaImage {
        self.image.to_rgba8()
    }

    fn create_texture(glyphs: &mut [Glyph], metrics: &Metrics) -> DynamicImage {
        let size = DEFAULT_FONT_TEXTURE_SIZE;

        let mut image = DynamicImage::new_rgba8(size as _, size as _);

        let num_glyphs: u32 = glyphs.len() as _;
        let cell_size: u32 = (((size * size) as u32 / num_glyphs) as f64).sqrt().ceil() as u32;

        let mut row: u32 = 0;
        let mut column: u32 = 0;
        for g in glyphs.iter_mut() {
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

            let _offset = (cell_size as f32 * (g.metrics.width / metrics.horizontal_offset)
                - cell_size as f32)
                * 0.5;
            let x_pos: i32 = (starting_x as i32 - _offset as i32)
                .max(0)
                .min(size as i32 - cell_size as i32);

            g.render(|x, y, alpha| {
                let v = (alpha * 255.0).round() as u8;
                image.put_pixel(x_pos as u32 + x, starting_y + y, Rgba([v; 4]))
            });

            g.texture_coord = [
                (starting_x) as f32 / size as f32,
                (starting_y) as f32 / size as f32,
                (starting_x + cell_size) as f32 / size as f32,
                (starting_y + cell_size) as f32 / size as f32,
            ]
            .into();

            column += 1;
        }

        image
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
