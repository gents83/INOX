use ttf_parser::*;
use nrg_math::*;
use super::raster::*;

pub struct Geometry<'a> {
    rasterizer: Raster<'a>,
    current_point: Point2f,
    last_movement: Option<Point2f>,
}

impl<'a> Geometry<'a> {
    pub fn new(width: usize, height: usize, data: &'a mut Vec<f32>) -> Self {
        Self {
            rasterizer: Raster::new(width, height, data),
            current_point: Point2f::default(),
            last_movement: None,
        }
    }
}

impl<'a> OutlineBuilder for Geometry<'a> {
    fn move_to(&mut self, x0: f32, y0: f32) {
        let next_point = Point2f::new(x0, y0);
        self.current_point = next_point;
        self.last_movement = Some(next_point);
    }

    fn line_to(&mut self, x0: f32, y0: f32) {
        let next_point = Point2f::new(x0, y0);
        self.rasterizer.draw_line(self.current_point, next_point);
        self.current_point = next_point;
    }

    fn quad_to(&mut self, x0: f32, y0: f32, x1: f32, y1: f32) {
        let control_point = Point2f::new(x0, y0);
        let next_point = Point2f::new(x1, y1);

        self.rasterizer.draw_quad(self.current_point, control_point, next_point);
        self.current_point = next_point;
    }

    fn curve_to(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, x2: f32, y2: f32) {
        let first_control = Point2f::new(x0, y0);
        let second_control = Point2f::new(x1, y1);
        let next_point = Point2f::new(x2, y2);

        self.rasterizer.draw_cubic(self.current_point, first_control, second_control, next_point);
        self.current_point = next_point;
    }

    fn close(&mut self) {
        if let Some(m) = self.last_movement {
            self.rasterizer.draw_line(self.current_point, m);
        }
    }
}