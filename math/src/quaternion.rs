use std::f32::consts::PI;

use crate::{Vector3, Zero};

use crate::Quaternion;

pub trait Quat {
    fn from_euler_angles(pitch_yaw_roll_angles: Vector3) -> Quaternion;
    fn to_euler_angles(&self) -> Vector3;
}

impl Quat for Quaternion {
    fn to_euler_angles(&self) -> Vector3 {
        let mut angles = Vector3::zero();

        // roll
        let sinr_cosp = 2. * (self.s * self.v.x + self.v.y * self.v.z);
        let cosr_cosp = 1. - 2. * (self.v.x * self.v.x + self.v.y * self.v.y);
        angles.z = sinr_cosp.atan2(cosr_cosp);

        // pitch
        let sinp = 2. * (self.s * self.v.y - self.v.z * self.v.x);
        if sinp.abs() >= 1. {
            angles.x = (PI / 2.).copysign(sinp); // use 90 degrees if out of range
        } else {
            angles.x = sinp.asin();
        }

        // yaw
        let siny_cosp = 2. * (self.s * self.v.z + self.v.x * self.v.y);
        let cosy_cosp = 1. - 2. * (self.v.y * self.v.y + self.v.z * self.v.z);
        angles.y = siny_cosp.atan2(cosy_cosp);

        return angles;
    }

    fn from_euler_angles(angles: Vector3) -> Quaternion {
        // Abbreviations for the various angular functions
        let cy = (angles.y * 0.5).cos();
        let sy = (angles.y * 0.5).sin();
        let cp = (angles.x * 0.5).cos();
        let sp = (angles.x * 0.5).sin();
        let cr = (angles.z * 0.5).cos();
        let sr = (angles.z * 0.5).sin();

        Quaternion::new(
            cr * cp * cy + sr * sp * sy,
            sr * cp * cy - cr * sp * sy,
            cr * sp * cy + sr * cp * sy,
            cr * cp * sy - sr * sp * cy,
        )
    }
}
