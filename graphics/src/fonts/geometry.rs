use super::raster::*;
use nrg_math::*;
use ttf_parser::*;

#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub start: Vector2,
    pub end: Vector2,
}

pub struct Geometry<'a> {
    rasterizer: Raster<'a>,
    offset: Vector2,
    scale: Vector2,
    current_point: Vector2,
    last_movement: Option<Vector2>,
    lines: Vec<Line>,
}

impl<'a> Geometry<'a> {
    pub fn new(
        offset: Vector2,
        scale: Vector2,
        width: usize,
        height: usize,
        data: &'a mut Vec<f32>,
    ) -> Self {
        Self {
            rasterizer: Raster::new(width, height, data),
            offset,
            scale,
            current_point: Vector2::default_zero(),
            last_movement: None,
            lines: Vec::new(),
        }
    }

    pub fn get_lines(&self) -> &Vec<Line> {
        &self.lines
    }
}

impl<'a> OutlineBuilder for Geometry<'a> {
    fn move_to(&mut self, x0: f32, y0: f32) {
        let next_point = Vector2::new(
            (x0 + self.offset.x) * self.scale.x,
            (y0 + self.offset.y) * self.scale.y,
        );
        self.current_point = next_point;
        self.last_movement = Some(next_point);
    }

    fn line_to(&mut self, x0: f32, y0: f32) {
        let next_point = Vector2::new(
            (x0 + self.offset.x) * self.scale.x,
            (y0 + self.offset.y) * self.scale.y,
        );
        self.draw_line(self.current_point, next_point);
        self.current_point = next_point;
    }

    fn quad_to(&mut self, x0: f32, y0: f32, x1: f32, y1: f32) {
        let control_point = Vector2::new(
            (x0 + self.offset.x) * self.scale.x,
            (y0 + self.offset.y) * self.scale.y,
        );
        let next_point = Vector2::new(
            (x1 + self.offset.x) * self.scale.x,
            (y1 + self.offset.y) * self.scale.y,
        );

        self.draw_quad(self.current_point, control_point, next_point);
        self.current_point = next_point;
    }

    fn curve_to(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, x2: f32, y2: f32) {
        let first_control = Vector2::new(
            (x0 + self.offset.x) * self.scale.x + self.offset.x,
            y0 * self.scale.y + self.offset.y,
        );
        let second_control = Vector2::new(
            (x1 + self.offset.x) * self.scale.x,
            (y1 + self.offset.y) * self.scale.y,
        );
        let next_point = Vector2::new(
            (x2 + self.offset.x) * self.scale.x,
            (y2 + self.offset.y) * self.scale.y,
        );

        self.draw_cubic(
            self.current_point,
            first_control,
            second_control,
            next_point,
        );
        self.current_point = next_point;
    }

    fn close(&mut self) {
        if let Some(m) = self.last_movement {
            self.draw_line(self.current_point, m);
        }
    }
}

impl<'a> Geometry<'a> {
    fn draw_line(&mut self, start: Vector2, end: Vector2) {
        self.lines.push(Line { start, end });
        self.rasterizer.draw_line(start, end);
    }

    fn draw_quad(&mut self, p0: Vector2, p1: Vector2, p2: Vector2) {
        let devx = p0.x - 2.0 * p1.x + p2.x;
        let devy = p0.y - 2.0 * p1.y + p2.y;
        let devsq = devx * devx + devy * devy;
        if devsq < 0.333 {
            self.draw_line(p0, p2);
            return;
        }
        let tol = 3.0;
        let n = 1 + (tol * devsq).sqrt().sqrt().floor() as usize;
        let mut p = p0;
        let nrecip = (n as f32).recip();
        let mut t = 0.0;
        for _i in 0..n - 1 {
            t += nrecip;
            let pn = lerp_v2(t, lerp_v2(t, p0, p1), lerp_v2(t, p1, p2));
            self.draw_line(p, pn);
            p = pn;
        }
        self.draw_line(p, p2);
    }

    fn draw_cubic(&mut self, p0: Vector2, p1: Vector2, p2: Vector2, p3: Vector2) {
        self.tesselate_cubic(p0, p1, p2, p3, 0);
    }

    fn tesselate_cubic(&mut self, p0: Vector2, p1: Vector2, p2: Vector2, p3: Vector2, n: u8) {
        const OBJSPACE_FLATNESS: f32 = 0.35;
        const OBJSPACE_FLATNESS_SQUARED: f32 = OBJSPACE_FLATNESS * OBJSPACE_FLATNESS;
        const MAX_RECURSION_DEPTH: u8 = 16;

        let longlen = p0.squared_distance(p1) + p1.squared_distance(p2) + p2.squared_distance(p3);
        let shortlen = p0.squared_distance(p3);
        let flatness_squared = longlen - shortlen;

        if n < MAX_RECURSION_DEPTH && flatness_squared > OBJSPACE_FLATNESS_SQUARED {
            let p01 = lerp_v2(0.5, p0, p1);
            let p12 = lerp_v2(0.5, p1, p2);
            let p23 = lerp_v2(0.5, p2, p3);

            let pa = lerp_v2(0.5, p01, p12);
            let pb = lerp_v2(0.5, p12, p23);

            let mp = lerp_v2(0.5, pa, pb);

            self.tesselate_cubic(p0, p01, pa, mp, n + 1);
            self.tesselate_cubic(mp, pb, p23, p3, n + 1);
        } else {
            self.draw_line(p0, p3);
        }
    }
}
