#![allow(dead_code)]

#[path = "../src/launcher.rs"]
mod launcher;

#[test]
fn default_launcher_provides_shared_runtime_handles() {
    let launcher = launcher::Launcher::default();

    let _context = launcher.context();
    let _shared_data = launcher.shared_data();
    let _message_hub = launcher.message_hub();
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn wasm_panic_hook_binds_console_error_explicitly() {
    let source = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/platform/wasm.rs"),
    )
    .expect("WASM platform source must be available");
    assert!(source.contains("js_namespace = console, js_name = error"));
    assert!(source.contains("fn console_error(msg: &str)"));
    assert!(source.contains("console_error(&msg)"));
}
