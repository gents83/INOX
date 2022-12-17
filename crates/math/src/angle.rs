use cgmath::{Deg, Rad};

pub type Degrees = Deg<f32>;
pub type Radians = Rad<f32>;

pub trait NewAngle {
    fn new(angle: f32) -> Self;
}

impl NewAngle for Degrees {
    fn new(angle_in_degrees: f32) -> Degrees {
        Deg(angle_in_degrees)
    }
}
impl NewAngle for Radians {
    fn new(angle_in_radians: f32) -> Radians {
        Rad(angle_in_radians)
    }
}
