#![cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
#![allow(dead_code)]

#[path = "../src/exporter.rs"]
mod exporter;

use exporter::Exporter;

#[test]
fn exporter_starts_with_empty_state() {
    let exporter = Exporter::default();

    assert_eq!(
        std::mem::size_of_val(&exporter),
        std::mem::size_of::<Exporter>()
    );
}
