use std::{env, path::PathBuf};

use sabi_resources::{ConfigBase, DATA_RAW_FOLDER};
use sabi_serialize::*;

#[derive(Serializable, Debug, Clone)]
pub struct Config {
    pub title: String,
    pub pos_x: u32,
    pub pos_y: u32,
    pub width: u32,
    pub height: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            title: String::new(),
            pos_x: 0,
            pos_y: 0,
            width: 1280,
            height: 720,
        }
    }
}

impl SerializeFile for Config {
    fn extension() -> &'static str {
        "cfg"
    }
}
impl ConfigBase for Config {
    fn get_filename(&self) -> &'static str {
        "app.cfg"
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
    registry.register_type::<String>();
    registry.register_type::<u32>();

    let path = data_raw_folder().join("config").join(config.get_filename());

    println!("Filepath = {:?}", path);

    write_to_file(&config, &path, &registry);
}
