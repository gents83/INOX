use std::sync::Arc;

use inox_common_script::CommonScriptPlugin;
use inox_core::{Context, Plugin};
use inox_nodes::LogicNodeRegistry;
use inox_resources::Singleton;

#[test]
fn exposes_the_expected_plugin_identity() {
    let plugin = CommonScriptPlugin::default();

    assert_eq!(plugin.name(), "inox_common_script");
}

#[test]
fn prepare_registers_common_nodes_and_unprepare_removes_the_registry() {
    let context = Arc::new(Context::default());
    let mut plugin = CommonScriptPlugin::default();
    plugin.prepare(&context);

    let mut names = Vec::new();
    LogicNodeRegistry::get(context.shared_data()).for_each_node(|node| {
        names.push(node.name().to_owned());
    });
    assert!(names.iter().any(|name| name == "RustExampleNode"));
    assert!(names.iter().any(|name| name == "ScriptInitNode"));
    assert!(names.iter().any(|name| name == "RotateNode"));

    plugin.unprepare(&context);
    assert!(context
        .shared_data()
        .get_singleton::<LogicNodeRegistry>()
        .is_none());
}
