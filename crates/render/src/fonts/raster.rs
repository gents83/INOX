use inox_math::*;

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

    pub fn draw_line(&mut self, p0: Vector2, p1: Vector2) {
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
            let linestart: isize = (y * self.width) as isize;
            let dy = ((y + 1) as f32).min(p1.y) - (y as f32).max(p0.y);
            let xnext = x + dxdy * dy;
            let d = dy * dir;
            let (x0, x1) = if x < xnext { (x, xnext) } else { (xnext, x) };
            let x0floor = x0.floor();
            let x0i = x0floor as isize;
            let x1ceil = x1.ceil();
            let x1i = x1ceil as isize;
            if x1i <= x0i + 1 {
                let xmf = 0.5 * (x + xnext) - x0floor;
                let linestart_x0i = linestart + x0i;
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
                let linestart_x0i = linestart + x0i;
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
                        self.data[(linestart + xi) as usize] += d * s;
                    }
                    let a2 = a1 + (x1i - x0i - 3) as f32 * s;
                    self.data[(linestart + (x1i - 1)) as usize] += d * (1.0 - a2 - am);
                }
                self.data[(linestart + x1i) as usize] += d * am;
            }
            x = xnext;
        }
    }
}
