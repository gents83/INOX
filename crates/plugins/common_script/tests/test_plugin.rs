use inox_common_script::CommonScriptPlugin;
use inox_core::Plugin;

#[test]
fn exposes_the_expected_plugin_identity() {
    let plugin = CommonScriptPlugin::default();

    assert_eq!(plugin.name(), "inox_common_script");
}
