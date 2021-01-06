
use ttf_parser::*;
use nrg_math::*;
use super::geometry::*;

pub struct Glyph {
    pub id: GlyphId,
    pub width: f32,
    pub height: f32,
    pub vertical_offset: u16,
    pub is_upside_down: bool,
}

impl Glyph {
    pub fn create(id: GlyphId, face:&Face) -> Self {
        let mut glyph_width: usize = 0;
        let mut glyph_height: usize = 0;
        let mut offset: u16 = 0;
        if let Some(bb) = face.glyph_bounding_box(id) {
            glyph_width = (bb.x_max - bb.x_min) as usize;
            glyph_height = (bb.y_max - bb.y_min) as usize;
            offset = - bb.y_min as _;
        }
        let glyph_h_bearing = match face.glyph_hor_side_bearing(id) {
            Some(a) => a,
            _ => 0,
        };
        let glyph_v_bearing = match face.glyph_ver_side_bearing(id) {
            Some(a) => a,
            _ => 0,
        };
        let glyph_hor_advance = match face.glyph_hor_advance(id) {
            Some(a) => a,
            _ => 0,
        };
        let glyph_ver_advance = match face.glyph_ver_advance(id) {
            Some(a) => a,
            _ => 0,
        };

        glyph_width = glyph_hor_advance as _;
        glyph_height = (glyph_height as i16 + glyph_v_bearing) as _;

        let is_upside_down = face.has_table(TableName::GlyphVariations) || face.has_table(TableName::GlyphData);

        Self {
            id,
            width: glyph_width as _,
            height: glyph_height as _,
            vertical_offset: offset,
            is_upside_down,
        }
    }

    pub fn render<WritePixelFunc: FnMut(u32, u32, f32)>(&mut self, face: &Face, width: u32, height: u32, data: &mut Vec<f32>, mut write_pixel_func: WritePixelFunc) {
        let scale_x = width as f32 / self.width;
        let scale_y = height as f32 / self.height;
        let offset = Point2f::new(0.0, self.vertical_offset as _);
        
        let mut geometry = Geometry::new(offset, self.width as _, self.height as _, data);
        face.outline_glyph(self.id, &mut geometry);

        self.iterate_on_pixels(data, 
            self.width as _, self.height as _, 
            |index, alpha| {
                let idx = index as f32;
                let x: u32 = ((idx % self.width) * scale_x) as _;
                let mut y: u32 = (idx / self.width * scale_y) as _;
                if self.is_upside_down {
                    y = (height - y) % height;
                }
                write_pixel_func(x, y, alpha)
            }
        );
    }
    
    fn iterate_on_pixels<PerPixelFn: FnMut(usize, f32)>(&self, image_data:&Vec<f32>, width:usize, height:usize, mut px_fn: PerPixelFn) {
        let mut accumulated_alpha = 0.0;
        image_data[..width * height]
            .iter()
            .enumerate()
            .for_each(|(index, c)| {
                accumulated_alpha += c;
                px_fn(index, accumulated_alpha.abs().min(1.0));
            });
    }
}