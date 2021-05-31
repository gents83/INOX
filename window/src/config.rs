use nrg_graphics::PipelineData;
use nrg_math::*;
use nrg_resources::*;
use nrg_serialize::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "nrg_serialize")]
pub struct VkData {
    pub debug_validation_layers: bool,
}

impl Default for VkData {
    fn default() -> Self {
        Self {
            debug_validation_layers: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "nrg_serialize")]
pub struct Config {
    name: String,
    position: Vector2,
    width: u32,
    height: u32,
    vk_data: VkData,
    pipelines: Vec<PipelineData>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: String::from("NRG"),
            position: Vector2::default_zero(),
            width: 1280,
            height: 720,
            vk_data: VkData::default(),
            pipelines: Vec::new(),
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
    pub fn get_resolution(&self) -> Vector2 {
        Vector2::new(self.get_width() as _, self.get_height() as _)
    }
    pub fn get_position(&self) -> &Vector2 {
        &self.position
    }
    pub fn get_pipelines(&self) -> &Vec<PipelineData> {
        &self.pipelines
    }
    pub fn is_debug_validation_layers_enabled(&self) -> bool {
        self.vk_data.debug_validation_layers
    }
}
