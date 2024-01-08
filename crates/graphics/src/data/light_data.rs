use inox_bitmask::bitmask;
use inox_serialize::{Deserialize, Serialize, SerializeFile};

use crate::print_field_size;

#[bitmask]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(crate = "inox_serialize")]
pub enum LightType {
    None = 0,
    Directional = 1,
    Point = 1 << 1,
    Spot = 1 << 2,
}

#[repr(C, align(16))]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
#[serde(crate = "inox_serialize")]
pub struct LightData {
    pub position: [f32; 3],
    pub light_type: u32,
    pub direction: [f32; 3],
    pub intensity: f32,
    pub color: [f32; 3],
    pub range: f32,
    pub inner_cone_angle: f32,
    pub outer_cone_angle: f32,
    pub _padding1: f32,
    pub _padding2: f32,
}

impl SerializeFile for LightData {
    fn extension() -> &'static str {
        "light"
    }
}

impl Default for LightData {
    fn default() -> Self {
        Self {
            position: [0.; 3],
            light_type: LightType::Directional.into(),
            direction: [-0.074, -0.198, -0.643],
            intensity: 1.,
            color: [1.; 3],
            range: -1.,
            inner_cone_angle: 0.,
            outer_cone_angle: core::f32::consts::PI / 4.,
            _padding1: 0.,
            _padding2: 0.,
        }
    }
}

impl LightData {
    #[allow(deref_nullptr)]
    pub fn debug_size(alignment_size: usize) {
        let total_size = std::mem::size_of::<Self>();
        println!("LightData info: Total size [{total_size}]");

        let mut s = 0;
        print_field_size!(s, position, [f32; 3], 1);
        print_field_size!(s, light_type, u32, 1);
        print_field_size!(s, color, [f32; 4], 1);
        print_field_size!(s, intensity, f32, 1);
        print_field_size!(s, range, f32, 1);
        print_field_size!(s, inner_cone_angle, f32, 1);
        print_field_size!(s, outer_cone_angle, f32, 1);

        println!(
            "Alignment result: {} -> {}",
            if s == total_size && s % alignment_size == 0 {
                "OK"
            } else {
                "TO ALIGN"
            },
            (s as f32 / alignment_size as f32).ceil() as usize * alignment_size
        );
    }
}
