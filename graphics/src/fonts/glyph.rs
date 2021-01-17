
use ttf_parser::*;
use nrg_math::*;
use super::geometry::*;

pub const DEFAULT_FONT_SIZE:usize = 64;

pub struct Glyph {
    pub id: GlyphId,
    pub width: f32,
    pub height: f32,
    pub horizontal_offset: f32,
    pub vertical_offset: f32,
    pub is_upside_down: bool,
    pub texture_coord: Vector4f,
    pub data: Vec<f32>,
    pub lines: Vec<Line>,
}

impl Glyph {
    pub fn create(id: GlyphId, face:&Face) -> Self {
        let mut glyph_width: f32 = 0.0;
        let mut glyph_height: f32 = 0.0;
        let mut horizontal_offset: f32 = 0.0;
        let mut vertical_offset: f32 = 0.0;
        if let Some(bb) = face.glyph_bounding_box(id) {
            glyph_width = (bb.x_max - bb.x_min) as _;
            glyph_height = (bb.y_max - bb.y_min) as _;
            horizontal_offset = 0.0;
            vertical_offset = - bb.y_min as f32;
        }
        let glyph_v_bearing = match face.glyph_ver_side_bearing(id) {
            Some(a) => a,
            _ => 0,
        };
        let glyph_hor_advance = match face.glyph_hor_advance(id) {
            Some(a) => a,
            _ => 0,
        };

        glyph_width = glyph_hor_advance as _;
        glyph_height = (glyph_height as i16 + glyph_v_bearing) as _;

        let mut data = Vec::new();
        let mut lines: Vec<Line> = Vec::new();
        let scale_x = DEFAULT_FONT_SIZE as f32 / glyph_width as f32;
        let scale_y = DEFAULT_FONT_SIZE as f32 / glyph_height as f32;

        data =  vec![0.0; (glyph_width * glyph_height)  as _];
        let draw_offset = Vector2f::new(0.0, vertical_offset);
        let draw_scale = Vector2f::new(scale_x, scale_y);
        
        let mut geometry = Geometry::new(draw_offset, draw_scale, DEFAULT_FONT_SIZE as _, DEFAULT_FONT_SIZE as _, &mut data);
        face.outline_glyph(id, &mut geometry);
        lines = geometry.get_lines().clone();

        let is_upside_down = face.has_table(TableName::GlyphVariations) || face.has_table(TableName::GlyphData);

        Self {
            id,
            width: glyph_width as _,
            height: glyph_height as _,
            horizontal_offset,
            vertical_offset,
            is_upside_down,
            texture_coord: [0.0, 0.0, 1.0, 1.0].into(),
            data,
            lines,
        }
    }

    pub fn render<WritePixelFunc: FnMut(u32, u32, f32)>(&mut self, width: u32, height: u32, mut write_pixel_func: WritePixelFunc) {
        
        if self.width < f32::EPSILON || self.height < f32::EPSILON {
            return;
        }

        let scale_x = width as f32 / DEFAULT_FONT_SIZE as f32;
        let scale_y = height as f32 / DEFAULT_FONT_SIZE as f32;

        self.iterate_on_pixels( |index, alpha| {
                let idx = index as f32;
                let x: u32 = ((idx % DEFAULT_FONT_SIZE as f32) * scale_x) as _;
                let mut y: u32 = ((idx / DEFAULT_FONT_SIZE as f32) * scale_y) as _;
                if self.is_upside_down {
                    y = (height - y) % height;
                }
                write_pixel_func(x, y, alpha)
            }
        );
    }
    
    fn iterate_on_pixels<PerPixelFn: FnMut(usize, f32)>(&self, mut px_fn: PerPixelFn) {
        let mut accumulated_alpha = 0.0;
        self.data[..DEFAULT_FONT_SIZE * DEFAULT_FONT_SIZE]
            .iter()
            .enumerate()
            .for_each(|(index, c)| {
                accumulated_alpha += c;
                px_fn(index, accumulated_alpha.abs().min(1.0));
            });
    }
}