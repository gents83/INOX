#[path = "../src/config.rs"]
mod config;

use config::Config;
use inox_resources::ConfigBase;
use inox_serialize::SerializeFile;

#[test]
fn viewer_config_uses_expected_runtime_filename_and_extension() {
    let config = Config::default();

    assert_eq!(config.get_filename(), "viewer.cfg");
    assert_eq!(Config::extension(), "cfg");
    assert!(config.opaque_pass_pipeline.as_os_str().is_empty());
    assert!(config.wireframe_pass_pipeline.as_os_str().is_empty());
    assert!(config.ui_pass_pipeline.as_os_str().is_empty());
}
