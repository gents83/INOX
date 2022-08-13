use image::*;
use inox_math::{Vector2, Vector4};
use inox_platform::DEFAULT_DPI;
use inox_serialize::{Deserialize, Serialize};
use std::path::Path;
use ttf_parser::*;

use crate::{create_quad_with_texture, Glyph, MeshData, Metrics};

const DEFAULT_FONT_COUNT: u8 = 255;
pub const DEFAULT_FONT_TEXTURE_SIZE: usize = 1024;
//12pt = 16px = 1em = 100%
pub const FONT_PT_TO_PIXEL: f32 = DEFAULT_DPI / (72. * 2048.);

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "inox_serialize")]
pub struct FontData {
    metrics: Metrics,
    glyphs: Vec<Glyph>,
}

#[derive(Clone)]
pub struct TextData {
    pub text: String,
    pub position: Vector2,
    pub scale: f32,
    pub color: Vector4,
    pub spacing: Vector2,
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
    pub fn get_metrics(&self) -> &Metrics {
        &self.metrics
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
        let mut text_mesh_data = MeshData::default();

        let mut prev_pos = text_data.position;
        let size = FONT_PT_TO_PIXEL * text_data.scale;
        let spacing_x = FONT_PT_TO_PIXEL * text_data.spacing.x;
        let spacing_y = FONT_PT_TO_PIXEL * text_data.spacing.y;

        for c in text_data.text.as_bytes().iter() {
            let g = &self.glyphs[*c as usize];
            let mesh_data = create_quad_with_texture(
                Vector4::new(prev_pos.x, prev_pos.y, prev_pos.x + size, prev_pos.y + size),
                0.0,
                g.texture_coord,
            );
            text_mesh_data.append_mesh_data(mesh_data, false);
            if *c == b'\n' {
                prev_pos.x = text_data.position.x;
                prev_pos.y += size + spacing_y;
            } else {
                prev_pos.x += size * 0.5 + spacing_x;
            }
        }

        text_mesh_data.set_vertex_color(text_data.color);
        text_mesh_data
    }
}

impl FontData {
    fn new_ttf_font(filepath: &Path) -> Self {
        let font_data = ::std::fs::read(filepath).unwrap();

        let face = Face::from_slice(font_data.as_slice(), 0).unwrap();

        let mut max_glyph_metrics = Metrics::default();
        for character in 0..DEFAULT_FONT_COUNT {
            let metrics = Glyph::compute_metrics(character as _, &face);
            max_glyph_metrics = max_glyph_metrics.max(&metrics);
        }

        let mut glyphs: Vec<Glyph> = Vec::new();
        for character in 0..DEFAULT_FONT_COUNT {
            glyphs.push(Glyph::create(
                face.glyph_index(character as _).unwrap_or_default().0,
                &face,
                &max_glyph_metrics,
            ));
        }

        Self {
            metrics: max_glyph_metrics,
            glyphs,
        }
    }

    pub fn create_texture(&mut self) -> Vec<u8> {
        let size = DEFAULT_FONT_TEXTURE_SIZE;

        let mut image = DynamicImage::new_rgba8(size as _, size as _);

        let num_glyphs: u32 = self.glyphs.len() as _;
        let cell_size: u32 = (((size * size) as u32 / num_glyphs) as f64).sqrt().ceil() as u32;

        let mut row: u32 = 0;
        let mut column: u32 = 0;
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

            let _offset = (cell_size as f32 * (g.metrics.width / self.metrics.horizontal_offset)
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

        image.into_bytes()
    }
}
