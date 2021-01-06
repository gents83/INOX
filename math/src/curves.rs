use super::cast_to::*;
use super::float::*;
use super::points::*;
use super::vector::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CubicCurve2D<T> 
where T: Float {
    a: Point2D<T>,
    b: Point2D<T>,
    c: Point2D<T>,
    d: Point2D<T>,
}

impl<T> CubicCurve2D<T> 
where T: Float {
    pub fn new(a: Point2D<T>, b: Point2D<T>, c: Point2D<T>, d: Point2D<T>) -> Self {
        Self {
            a,
            b,
            c,
            d,
        }
    }

    pub fn scale(&self, scale: T) -> Self {
        Self {
            a: self.a.scale(scale),
            b: self.b.scale(scale),
            c: self.c.scale(scale),
            d: self.d.scale(scale),
        }
    }

    pub fn is_flat(&self, threshold: T) -> bool {
        let vec = Vector4::new(
            self.a.squared_distance(self.b),
            self.b.squared_distance(self.c),
            self.c.squared_distance(self.d),
            self.a.squared_distance(self.d),
        ).sqrt();
        (vec.x + vec.y + vec.z) < threshold * vec.w
    }

    pub fn split(&self) -> (CubicCurve2D<T>, CubicCurve2D<T>) {
        let q0 = self.a.midpoint(self.b);
        let q1 = self.b.midpoint(self.c);
        let q2 = self.c.midpoint(self.d);
        let r0 = q0.midpoint(q1);
        let r1 = q1.midpoint(q2);
        let s0 = r0.midpoint(r1);
        (CubicCurve2D::new(self.a, q0, r0, s0), CubicCurve2D::new(s0, r1, q2, self.d))
    }

    /// The point at time t in the curve.
    pub fn get_point(&self, t: T) -> Point2D<T> {
        let tm: T = T::one() - t;
        let a: T = tm * tm * tm;
        let b: T = cast::<f32, T>(3.0).unwrap() * (tm * tm) * t;
        let c: T = cast::<f32, T>(3.0).unwrap() * tm * (t * t);
        let d: T = t * t * t;

        let x = a * self.a.x + b * self.b.x + c * self.c.x + d * self.d.x;
        let y = a * self.a.y + b * self.b.y + c * self.c.y + d * self.d.y;
        Point2D::new(x, y)
    }

    /// The slope of the tangent line at time t.
    pub fn get_slope(&self, t: T) -> (T, T) {
        let tm: T = T::one() - t;
        let a: T = cast::<f32, T>(3.0).unwrap() * (tm * tm);
        let b: T = cast::<f32, T>(6.0).unwrap() * tm * t;
        let c: T = cast::<f32, T>(3.0).unwrap() * (t * t);

        let x: T = a * (self.b.x - self.a.x) + b * (self.c.x - self.b.x) + c * (self.d.x - self.c.x);
        let y: T = a * (self.b.y - self.a.y) + b * (self.c.y - self.b.y) + c * (self.d.y - self.c.y);
        (x, y)
    }

    /// The angle of the tangent line at time t in rads.
    pub fn get_angle(&self, t: T) -> T {
        let (x, y) = self.get_slope(t);
        Float::abs(Float::atan2(x, y))
    }
}




#[derive(Copy, Clone, Debug, PartialEq)]
pub struct QuadraticCurve2D<T> 
where T: Float{
    a: Point2D<T>,
    b: Point2D<T>,
    c: Point2D<T>,
}

impl<T> QuadraticCurve2D<T>
where T:Float {
    pub fn new(a: Point2D<T>, b: Point2D<T>, c: Point2D<T>) -> Self {
        Self {
            a,
            b,
            c,
        }
    }

    pub fn scale(&self, scale: T) -> Self {
        Self {
            a: self.a.scale(scale),
            b: self.b.scale(scale),
            c: self.c.scale(scale),
        }
    }

    pub fn is_flat(&self, threshold: T) -> bool {
        let vec = Vector4::new(
            self.a.squared_distance(self.b),
            self.b.squared_distance(self.c),
            self.a.squared_distance(self.c),
            T::one(),
        )
        .sqrt();
        (vec.x + vec.y) < threshold * vec.z
    }

    pub fn split(&self) -> (QuadraticCurve2D<T>, QuadraticCurve2D<T>) {
        let q0 = self.a.midpoint(self.b);
        let q1 = self.b.midpoint(self.c);
        let r0 = q0.midpoint(q1);
        (QuadraticCurve2D::new(self.a, q0, r0), QuadraticCurve2D::new(r0, q1, self.c))
    }

    /// The point at time t in the curve.
    pub fn get_point(&self, t: T) -> Point2D<T> {
        let tm: T = T::one() - t;
        let a: T = tm * tm;
        let b: T = cast::<f32, T>(2.0).unwrap() * tm * t;
        let c: T = t * t;

        let x: T = a * self.a.x + b * self.b.x + c * self.c.x;
        let y: T = a * self.a.y + b * self.b.y + c * self.c.y;
        Point2D::new(x, y)
    }

    /// The slope of the tangent line at time t.
    pub fn get_slope(&self, t: T) -> (T, T) {
        let tm: T = T::one() - t;
        let a: T = cast::<f32, T>(2.0).unwrap() * tm;
        let b: T = cast::<f32, T>(2.0).unwrap() * t;

        let x: T = a * (self.b.x - self.a.x) + b * (self.c.x - self.b.x);
        let y: T = a * (self.b.y - self.a.y) + b * (self.c.y - self.b.y);
        (x, y)
    }

    /// The angle of the tangent line at time t in rads.
    pub fn get_angle(&self, t: T) -> T {
        let (x, y) = self.get_slope(t);
        Float::abs(Float::atan2(x, y))
    }
}


pub type CubicCurve = CubicCurve2D<f32>;
pub type QuadraticCurve = QuadraticCurve2D<f32>;