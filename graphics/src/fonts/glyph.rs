use super::geometry::*;
use nrg_math::*;
use ttf_parser::*;

pub const DEFAULT_FONT_GLYPH_SIZE: usize = 64;

pub struct Metrics {
    pub width: f32,
    pub height: f32,
    pub horizontal_offset: f32,
    pub vertical_offset: f32,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
            horizontal_offset: 0.0,
            vertical_offset: 0.0,
        }
    }
}

pub struct Glyph {
    pub id: GlyphId,
    pub metrics: Metrics,
    is_upside_down: bool,
    pub texture_coord: Vector4,
    pub data: Vec<f32>,
    pub lines: Vec<Line>,
}

impl Metrics {
    pub fn max(&self, other: &Metrics) -> Self {
        Self {
            width: f32::max(self.width, other.width),
            height: f32::max(self.height, other.height),
            horizontal_offset: f32::max(self.horizontal_offset, other.horizontal_offset),
            vertical_offset: f32::max(self.vertical_offset, other.vertical_offset),
        }
    }
}

impl Glyph {
    pub fn compute_metrics(id: GlyphId, face: &Face) -> Metrics {
        let mut bb_width: f32 = 0.0;
        let mut glyph_height: f32 = 0.0;
        let mut vertical_offset: f32 = 0.0;
        if let Some(bb) = face.glyph_bounding_box(id) {
            glyph_height = (bb.y_max - bb.y_min) as _;
            bb_width = (bb.x_max - bb.x_min) as _;
            vertical_offset = -bb.y_min as f32;
        }
        let horizontal_offset = match face.glyph_hor_side_bearing(id) {
            Some(a) => a as _,
            _ => 0.0,
        };
        let glyph_v_bearing = match face.glyph_ver_side_bearing(id) {
            Some(a) => a,
            _ => 0,
        };
        let glyph_hor_advance = match face.glyph_hor_advance(id) {
            Some(a) => a,
            _ => 0,
        };
        let glyph_width = glyph_hor_advance as _;
        glyph_height = (glyph_height as i16 + glyph_v_bearing) as _;

        Metrics {
            width: glyph_width,
            height: glyph_height,
            horizontal_offset: horizontal_offset + bb_width,
            vertical_offset,
        }
    }

    pub fn create(id: GlyphId, face: &Face, max_metrics: &Metrics) -> Self {
        let metrics = Glyph::compute_metrics(id, face);
        let scale_x = DEFAULT_FONT_GLYPH_SIZE as f32 / max_metrics.width as f32;
        let scale_y = DEFAULT_FONT_GLYPH_SIZE as f32 / max_metrics.height as f32;

        let mut data = vec![0.0; (max_metrics.width * max_metrics.height) as _];
        let draw_offset = Vector2::new(max_metrics.horizontal_offset, max_metrics.vertical_offset);
        let draw_scale = Vector2::new(scale_x, scale_y);

        let mut geometry = Geometry::new(
            draw_offset,
            draw_scale,
            DEFAULT_FONT_GLYPH_SIZE as _,
            DEFAULT_FONT_GLYPH_SIZE as _,
            &mut data,
        );
        face.outline_glyph(id, &mut geometry);
        let lines = geometry.get_lines().clone();

        let is_upside_down =
            face.has_table(TableName::GlyphVariations) || face.has_table(TableName::GlyphData);

        Self {
            id,
            metrics,
            is_upside_down,
            texture_coord: [0.0, 0.0, 1.0, 1.0].into(),
            data,
            lines,
        }
    }

    pub fn render<WritePixelFunc: FnMut(u32, u32, f32)>(
        &mut self,
        mut write_pixel_func: WritePixelFunc,
    ) {
        if self.metrics.width < f32::EPSILON || self.metrics.height < f32::EPSILON {
            return;
        }

        self.iterate_on_pixels(|index, alpha| {
            let idx = index as f32;
            let x: u32 = (idx % DEFAULT_FONT_GLYPH_SIZE as f32) as _;
            let mut y: u32 = (idx / DEFAULT_FONT_GLYPH_SIZE as f32) as _;
            if self.is_upside_down {
                y = (DEFAULT_FONT_GLYPH_SIZE as u32 - y) % DEFAULT_FONT_GLYPH_SIZE as u32;
            }
            write_pixel_func(x, y, alpha)
        });
    }
    fn iterate_on_pixels<PerPixelFn: FnMut(usize, f32)>(&self, mut px_fn: PerPixelFn) {
        let mut accumulated_alpha = 0.0;
        self.data[..DEFAULT_FONT_GLYPH_SIZE * DEFAULT_FONT_GLYPH_SIZE]
            .iter()
            .enumerate()
            .for_each(|(index, c)| {
                accumulated_alpha += c;
                px_fn(index, accumulated_alpha.abs().min(1.0));
            });
    }
}
