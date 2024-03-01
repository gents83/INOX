use inox_math::{Degrees, NewAngle};
use inox_serialize::{Deserialize, Serialize, SerializeFile};

use inox_render::{DEFAULT_ASPECT_RATIO, DEFAULT_FAR, DEFAULT_FOV, DEFAULT_NEAR};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "inox_serialize")]
pub struct CameraData {
    pub aspect_ratio: f32,
    pub near: f32,
    pub far: f32,
    pub fov: Degrees,
}

impl SerializeFile for CameraData {
    fn extension() -> &'static str {
        "camera"
    }
}

impl Default for CameraData {
    fn default() -> Self {
        Self {
            aspect_ratio: DEFAULT_ASPECT_RATIO,
            near: DEFAULT_NEAR,
            far: DEFAULT_FAR,
            fov: Degrees::new(DEFAULT_FOV),
        }
    }
}
