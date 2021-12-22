use std::{env, path::PathBuf};

use sabi_resources::{ConfigBase, DATA_RAW_FOLDER};
use sabi_serialize::*;

#[derive(Default, Serializable, Debug, Clone)]
pub struct Config {
    pub ui_scale: f32,
    pub ui_pipeline: PathBuf,
}

impl SerializeFile for Config {
    fn extension() -> &'static str {
        "cfg"
    }
}
impl ConfigBase for Config {
    fn get_filename(&self) -> &'static str {
        "ui.cfg"
    }
}

#[test]
fn config_tests() {
    create_empty_config_file();
}

fn data_raw_folder() -> PathBuf {
    env::current_dir()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(DATA_RAW_FOLDER)
}

#[allow(dead_code)]
fn create_empty_config_file() {
    let config = Config::default();
    let mut registry = SerializableRegistry::default();
    registry.register_type::<PathBuf>();
    registry.register_type::<u32>();
    registry.register_type::<Config>();

    let path = data_raw_folder().join("config").join(config.get_filename());

    println!("Filepath = {:?}", path);

    write_to_file(&config, &path, &registry);
}
