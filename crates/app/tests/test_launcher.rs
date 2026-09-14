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
