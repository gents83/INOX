use sabi_serialize::*;

use crate::{
    DEFAULT_CAMERA_ASPECT_RATIO, DEFAULT_CAMERA_FAR, DEFAULT_CAMERA_FOV, DEFAULT_CAMERA_NEAR,
};

#[derive(Serializable, Debug, PartialEq, Clone)]
pub struct CameraData {
    pub aspect_ratio: f32,
    pub near: f32,
    pub far: f32,
    pub fov: f32, // in degrees
}

impl SerializeFile for CameraData {
    fn extension() -> &'static str {
        "camera"
    }
}

impl Default for CameraData {
    fn default() -> Self {
        Self {
            aspect_ratio: DEFAULT_CAMERA_ASPECT_RATIO,
            near: DEFAULT_CAMERA_NEAR,
            far: DEFAULT_CAMERA_FAR,
            fov: DEFAULT_CAMERA_FOV,
        }
    }
}
