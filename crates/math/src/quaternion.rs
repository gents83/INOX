use std::f32::consts::PI;

use crate::Vector3;
pub use cgmath::Zero;

pub type Quaternion = cgmath::Quaternion<f32>;

pub trait Quat {
    fn from_euler_angles(roll_yaw_pitch: Vector3) -> Quaternion;
    fn to_euler_angles(&self) -> Vector3;
}

impl Quat for Quaternion {
    fn to_euler_angles(&self) -> Vector3 {
        let mut roll_yaw_pitch = Vector3::zero();

        // roll (x-axis rotation)
        let sinr_cosp = 2. * (self.s * self.v.x + self.v.y * self.v.z);
        let cosr_cosp = 1. - 2. * (self.v.x * self.v.x + self.v.y * self.v.y);
        roll_yaw_pitch.x = sinr_cosp.atan2(cosr_cosp);

        // pitch (z-axis rotation)
        let sinp = 2. * (self.s * self.v.y - self.v.z * self.v.x);
        if sinp.abs() >= 1. {
            roll_yaw_pitch.y = (PI / 2.).copysign(sinp); // use 90 degrees if out of range
        } else {
            roll_yaw_pitch.y = sinp.asin();
        }

        // yaw (y-axis rotation)
        let siny_cosp = 2. * (self.s * self.v.z + self.v.x * self.v.y);
        let cosy_cosp = 1. - 2. * (self.v.y * self.v.y + self.v.z * self.v.z);
        roll_yaw_pitch.z = siny_cosp.atan2(cosy_cosp);

        roll_yaw_pitch
    }

    fn from_euler_angles(roll_yaw_pitch: Vector3) -> Quaternion {
        // Abbreviations for the various angular functions
        // yaw (Y), pitch (Z), roll (X)
        let cy = (roll_yaw_pitch.z * 0.5).cos();
        let sy = (roll_yaw_pitch.z * 0.5).sin();
        let cp = (roll_yaw_pitch.y * 0.5).cos();
        let sp = (roll_yaw_pitch.y * 0.5).sin();
        let cr = (roll_yaw_pitch.x * 0.5).cos();
        let sr = (roll_yaw_pitch.x * 0.5).sin();

        Quaternion::new(
            cr * cp * cy + sr * sp * sy,
            sr * cp * cy - cr * sp * sy,
            cr * sp * cy + sr * cp * sy,
            cr * cp * sy - sr * sp * cy,
        )
    }
}
