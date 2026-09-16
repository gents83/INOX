#[path = "../src/config.rs"]
mod config;

use config::Config;
use inox_resources::ConfigBase;
use inox_serialize::{deserialize, serialize, SerializeFile};

#[test]
fn viewer_config_uses_expected_runtime_filename_and_extension() {
    let config = Config::default();

    assert_eq!(config.get_filename(), "viewer.cfg");
    assert_eq!(Config::extension(), "cfg");
    assert!(config.opaque_pass_pipeline.as_os_str().is_empty());
    assert!(config.wireframe_pass_pipeline.as_os_str().is_empty());
    assert!(config.ui_pass_pipeline.as_os_str().is_empty());
}

#[test]
fn viewer_config_roundtrips_all_pipeline_paths() {
    let config = Config {
        opaque_pass_pipeline: "opaque.pipeline".into(),
        wireframe_pass_pipeline: "wireframe.pipeline".into(),
        ui_pass_pipeline: "ui.pipeline".into(),
    };

    let restored = deserialize::<Config>(&serialize(&config)).expect("config should deserialize");
    assert_eq!(restored, config);
}
