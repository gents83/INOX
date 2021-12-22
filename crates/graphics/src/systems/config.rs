use std::{env, path::PathBuf};

use sabi_resources::{ConfigBase, DATA_RAW_FOLDER};
use sabi_serialize::*;

use crate::{LoadOperation, RenderPassData, RenderTarget, StoreOperation};

#[derive(Default, Serializable, Debug, Clone)]
pub struct Config {
    pub render_passes: Vec<RenderPassData>,
    pub pipelines: Vec<PathBuf>,
}

impl SerializeFile for Config {
    fn extension() -> &'static str {
        "cfg"
    }
}
impl ConfigBase for Config {
    fn get_filename(&self) -> &'static str {
        "render.cfg"
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
    let mut config = Config::default();
    let mut registry = SerializableRegistry::default();
    registry.register_type::<PathBuf>();
    registry.register_type::<String>();
    registry.register_type::<LoadOperation>();
    registry.register_type::<StoreOperation>();
    registry.register_type::<RenderTarget>();
    registry.register_type::<RenderPassData>();
    registry.register_type::<Config>();

    let main_pass = RenderPassData {
        name: "MainPass".to_string(),
        load_color: LoadOperation::Clear,
        store_color: StoreOperation::Store,
        load_depth: LoadOperation::Clear,
        store_depth: StoreOperation::DontCare,
        render_target: RenderTarget::Texture,
        pipeline: PathBuf::new(),
        mesh_category_to_draw: vec!["SABI_Default".to_string(), "EditorWireframe".to_string()],
    };
    config.render_passes.push(main_pass);

    let path = data_raw_folder().join("config").join(config.get_filename());

    println!("Filepath = {:?}", path);

    write_to_file(&config, &path, &registry);
}
