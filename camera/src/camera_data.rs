use nrg_math::{MatBase, Matrix4};
use nrg_serialize::*;

use crate::{DEFAULT_CAMERA_FAR, DEFAULT_CAMERA_FOV, DEFAULT_CAMERA_NEAR};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "nrg_serialize")]
pub struct CameraData {
    pub transform: Matrix4,
    pub near: f32,
    pub far: f32,
    pub fov: f32,
}

impl Default for CameraData {
    fn default() -> Self {
        Self {
            transform: Matrix4::default_identity(),
            near: DEFAULT_CAMERA_NEAR,
            far: DEFAULT_CAMERA_FAR,
            fov: DEFAULT_CAMERA_FOV,
        }
    }
}
