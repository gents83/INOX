use inox_resources::ConfigBase;
use inox_serialize::{Deserialize, Serialize, SerializeFile};

#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(crate = "inox_serialize")]
pub struct Config {
    pub host_address: String,
    pub port: u32,
}

impl SerializeFile for Config {
    fn extension() -> &'static str {
        "cfg"
    }
}
impl ConfigBase for Config {
    fn get_filename(&self) -> &'static str {
        "connector.cfg"
    }
}
