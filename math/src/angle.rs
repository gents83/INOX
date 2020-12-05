
#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub struct Radians<T>(pub T);
#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub struct Degree<T>(pub T);


impl From<Radians<f64>> for Degree<f64> {
    fn from(angle: Radians<f64>) -> Degree<f64> {
        Degree::new(angle.0.to_degrees())
    }
}
impl From<Radians<f32>> for Degree<f32> {
    fn from(angle: Radians<f32>) -> Degree<f32> {
        Degree::new(angle.0.to_degrees())
    }
}

impl From<Degree<f64>> for Radians<f64> {
    fn from(angle: Degree<f64>) -> Radians<f64> {
        Radians::new(angle.0.to_radians())
    }
}
impl From<Degree<f32>> for Radians<f32> {
    fn from(angle: Degree<f32>) -> Radians<f32> {
        Radians::new(angle.0.to_radians())
    }
}
impl From<Degree<u64>> for Radians<u64> {
    fn from(angle: Degree<u64>) -> Radians<u64> {
        Radians::new((angle.0 as f64).to_radians() as _)
    }
}
impl From<Degree<u32>> for Radians<u32> {
    fn from(angle: Degree<u32>) -> Radians<u32> {
        Radians::new((angle.0 as f32).to_radians() as _)
    }
}

impl<T> Degree<T> {
    fn new(angle_in_degrees: T) -> Degree<T> {
        Degree(angle_in_degrees)
    }
}

impl<T> Radians<T> {
    fn new(angle_in_radians: T) -> Radians<T> {
        Radians(angle_in_radians)
    }
}