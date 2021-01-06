use nrg_math::*;


pub struct Raster<'a> {
    width: usize,
    height: usize,
    data: &'a mut Vec<f32>,
}

impl<'a> Raster<'a> {
    
    pub fn new(width: usize, height: usize, data: &'a mut Vec<f32>) -> Self {
        Self {
            width, 
            height, 
            data,
        }
    }

    pub fn draw_line(&mut self, p0: Point2f, p1: Point2f) {
        if (p0.y - p1.y).abs() <= core::f32::EPSILON {
            return;
        }
        let (dir, p0, p1) = if p0.y < p1.y {
            (1.0, p0, p1)
        } else {
            (-1.0, p1, p0)
        };
        let dxdy = (p1.x - p0.x) / (p1.y - p0.y);
        let mut x = p0.x;
        let y0 = p0.y as usize; // note: implicit max of 0 because usize (TODO: really true?)
        if p0.y < 0.0 {
            x -= p0.y * dxdy;
        }
        for y in y0..self.height.min(p1.y.ceil() as usize) {
            let linestart = y * self.width;
            let dy = ((y + 1) as f32).min(p1.y) - (y as f32).max(p0.y);
            let xnext = x + dxdy * dy;
            let d = dy * dir;
            let (x0, x1) = if x < xnext { (x, xnext) } else { (xnext, x) };
            let x0floor = x0.floor();
            let x0i = x0floor as i32;
            let x1ceil = x1.ceil();
            let x1i = x1ceil as i32;
            if x1i <= x0i + 1 {
                let xmf = 0.5 * (x + xnext) - x0floor;
                let linestart_x0i = linestart as isize + x0i as isize;
                if linestart_x0i < 0 {
                    continue; // oob index
                }
                self.data[linestart_x0i as usize] += d - d * xmf;
                self.data[linestart_x0i as usize + 1] += d * xmf;
            } else {
                let s = (x1 - x0).recip();
                let x0f = x0 - x0floor;
                let a0 = 0.5 * s * (1.0 - x0f) * (1.0 - x0f);
                let x1f = x1 - x1ceil + 1.0;
                let am = 0.5 * s * x1f * x1f;
                let linestart_x0i = linestart as isize + x0i as isize;
                if linestart_x0i < 0 {
                    continue; // oob index
                }
                self.data[linestart_x0i as usize] += d * a0;
                if x1i == x0i + 2 {
                    self.data[linestart_x0i as usize + 1] += d * (1.0 - a0 - am);
                } else {
                    let a1 = s * (1.5 - x0f);
                    self.data[linestart_x0i as usize + 1] += d * (a1 - a0);
                    for xi in x0i + 2..x1i - 1 {
                        self.data[linestart + xi as usize] += d * s;
                    }
                    let a2 = a1 + (x1i - x0i - 3) as f32 * s;
                    self.data[linestart + (x1i - 1) as usize] += d * (1.0 - a2 - am);
                }
                self.data[linestart + x1i as usize] += d * am;
            }
            x = xnext;
        }
    }

    pub fn draw_quad(&mut self, p0: Point2f, p1: Point2f, p2: Point2f) {
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
            let pn = lerp(t, lerp(t, p0, p1), lerp(t, p1, p2));
            self.draw_line(p, pn);
            p = pn;
        }
        self.draw_line(p, p2);
    }

    pub fn draw_cubic(&mut self, p0: Point2f, p1: Point2f, p2: Point2f, p3: Point2f) {
        self.tesselate_cubic(p0, p1, p2, p3, 0);
    }

    fn tesselate_cubic(&mut self, p0: Point2f, p1: Point2f, p2: Point2f, p3: Point2f, n: u8) {
        const OBJSPACE_FLATNESS: f32 = 0.35;
        const OBJSPACE_FLATNESS_SQUARED: f32 = OBJSPACE_FLATNESS * OBJSPACE_FLATNESS;
        const MAX_RECURSION_DEPTH: u8 = 16;

        let longlen = p0.squared_distance(p1) + p1.squared_distance(p2) + p2.squared_distance(p3);
        let shortlen = p0.squared_distance(p3);
        let flatness_squared = longlen - shortlen;

        if n < MAX_RECURSION_DEPTH && flatness_squared > OBJSPACE_FLATNESS_SQUARED {
            let p01 = lerp(0.5, p0, p1);
            let p12 = lerp(0.5, p1, p2);
            let p23 = lerp(0.5, p2, p3);

            let pa = lerp(0.5, p01, p12);
            let pb = lerp(0.5, p12, p23);

            let mp = lerp(0.5, pa, pb);

            self.tesselate_cubic(p0, p01, pa, mp, n + 1);
            self.tesselate_cubic(mp, pb, p23, p3, n + 1);
        } else {
            self.draw_line(p0, p3);
        }
    }

}