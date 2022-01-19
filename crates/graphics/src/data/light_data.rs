use sabi_serialize::{Deserialize, Serialize, SerializeFile};

use crate::print_field_size;

pub const MAX_NUM_LIGHTS: usize = 64;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "sabi_serialize")]
pub enum LightType {
    None = 0,
    Directional = 1,
    Point = 2,
    Spot = 3,
}

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
#[serde(crate = "sabi_serialize")]
pub struct LightData {
    pub position: [f32; 3],
    pub light_type: u32,
    pub color: [f32; 4],
    pub intensity: f32,
    pub range: f32,
    pub inner_cone_angle: f32,
    pub outer_cone_angle: f32,
}

impl SerializeFile for LightData {
    fn extension() -> &'static str {
        "light"
    }
}

impl LightData {
    #[allow(deref_nullptr)]
    pub fn debug_size(alignment_size: usize) {
        let total_size = std::mem::size_of::<Self>();
        println!("LightData info: Total size [{}]", total_size);

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
