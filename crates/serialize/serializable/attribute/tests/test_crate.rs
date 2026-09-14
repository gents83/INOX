#[test]
fn proc_macro_crate_compiles_as_an_independent_package() {
    // Procedural macros are exercised by downstream crates such as
    // `inox_serializable`; this package intentionally has no runtime dependency
    // solely to make an integration test fixture.
    assert!(!std::env::var("CARGO_PKG_NAME").unwrap().is_empty());
}
