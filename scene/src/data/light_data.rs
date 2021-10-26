use nrg_math::Vector3;
use nrg_serialize::*;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "nrg_serialize")]
pub enum LightType {
    Directional,
    Point,
    Spot(f32, f32), // inner_cone_angle, outer_cone_angle
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "nrg_serialize")]
pub struct LightData {
    pub light_type: LightType,
    pub color: Vector3,
    pub intensity: f32,
    pub range: f32,
}

impl Default for LightData {
    fn default() -> Self {
        Self {
            light_type: LightType::Point,
            color: Vector3::new(1.0, 1.0, 1.0),
            intensity: 1000.0,
            range: 1.,
        }
    }
}
