use nrg_math::{Degrees, NewAngle};
use nrg_serialize::*;

use crate::{
    DEFAULT_CAMERA_ASPECT_RATIO, DEFAULT_CAMERA_FAR, DEFAULT_CAMERA_FOV, DEFAULT_CAMERA_NEAR,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "nrg_serialize")]
pub struct CameraData {
    pub aspect_ratio: f32,
    pub near: f32,
    pub far: f32,
    pub fov: Degrees,
}

impl Default for CameraData {
    fn default() -> Self {
        Self {
            aspect_ratio: DEFAULT_CAMERA_ASPECT_RATIO,
            near: DEFAULT_CAMERA_NEAR,
            far: DEFAULT_CAMERA_FAR,
            fov: Degrees::new(DEFAULT_CAMERA_FOV),
        }
    }
}
