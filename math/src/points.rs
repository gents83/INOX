use super::cast_to::*;
use super::float::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point2D<T>
where T: Float {
    pub x: T,
    pub y: T,
}

impl<T> Default for Point2D<T>
where T: Float {
    fn default() -> Self {
        Self {
            x: T::zero(),
            y: T::zero(),
        }
    }
}

impl<T> Point2D<T>
where T: Float {
    pub fn new(x: T, y: T) -> Self {
        Self {
            x,
            y,
        }
    }

    pub fn scale(&self, scale: T) -> Self {
        Self {
            x: self.x * scale,
            y: self.y * scale,
        }
    }

    pub fn squared_distance(&self, other: Point2D<T>) -> T {
        let x = self.x - other.x;
        let y = self.y - other.y;
        x * x + y * y
    }

    pub fn midpoint(&self, other: Point2D<T>) -> Self {
        Self {
            x: (self.x + other.x) / cast(2.0).unwrap(),
            y: (self.y + other.y) / cast(2.0).unwrap(),
        }
    }
}





#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point3D<T>
where T: Float {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Default for Point3D<T>
where T: Float {
    fn default() -> Self {
        Self {
            x: T::zero(),
            y: T::zero(),
            z: T::zero(),
        }
    }
}

impl<T> Point3D<T>
where T: Float {
    pub fn new(x: T, y: T, z:T) -> Self {
        Self {
            x,
            y,
            z,
        }
    }

    pub fn scale(&self, scale: T) -> Self {
        Self {
            x: self.x * scale,
            y: self.y * scale,
            z: self.z * scale,
        }
    }

    pub fn squared_distance(&self, other: Point3D<T>) -> T {
        let x = self.x - other.x;
        let y = self.y - other.y;
        let z = self.z - other.z;
        x * x + y * y + z * z
    }

    pub fn distance(&self, other: Point3D<T>) -> T {
        self.squared_distance(other).sqrt()
    }

    pub fn midpoint(&self, other: Point3D<T>) -> Self {
        Self {
            x: (self.x + other.x) * cast(0.5).unwrap(),
            y: (self.y + other.y) * cast(0.5).unwrap(),
            z: (self.z + other.z) * cast(0.5).unwrap(),
        }
    }
}

pub type Point2f = Point2D<f32>;
pub type Point3f = Point3D<f32>;

#[inline]
pub fn lerp<T>(t: T, p0: Point2D<T>, p1: Point2D<T>) -> Point2D<T> 
where T: Float {
    Point2D::new(p0.x + t * (p1.x - p0.x), p0.y + t * (p1.y - p0.y))
}