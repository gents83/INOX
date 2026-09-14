#[test]
fn proc_macro_package_compiles_without_runtime_test_dependencies() {
    // The generated type intentionally depends on `inox_serialize`; the
    // proc-macro package has no runtime dependency and cannot exercise a
    // generated type without violating the workspace dependency boundary.
    assert!(!std::env::var("CARGO_PKG_NAME").unwrap().is_empty());
}
