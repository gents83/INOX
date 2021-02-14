use nrg_core::*;
use nrg_math::*;
use nrg_serialize::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "nrg_serialize")]
pub struct Config {
    name: String,
    position: Vector2u,
    width: u32,
    height: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: String::from("NRG"),
            position: Vector2u::default(),
            width: 1280,
            height: 720,
        }
    }
}

impl Data for Config {}
impl ConfigBase for Config {
    fn get_filename(&self) -> &'static str {
        "window.cfg"
    }
}

impl Config {
    pub fn get_name(&self) -> &String {
        &self.name
    }
    pub fn get_width(&self) -> u32 {
        self.width
    }
    pub fn get_height(&self) -> u32 {
        self.height
    }
    pub fn get_resolution(&self) -> Vector2u {
        Vector2u::new(self.get_width(), self.get_height())
    }
    pub fn get_position(&self) -> &Vector2u {
        &self.position
    }
}
