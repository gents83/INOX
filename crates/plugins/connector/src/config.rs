use sabi_resources::ConfigBase;
use sabi_serialize::*;

#[derive(Default, Serializable, Debug, Clone)]
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
