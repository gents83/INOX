#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
#[test]
fn debug_log_macro_is_available_on_native_targets() {
    let result: () = inox_log::debug_log!("test message {}", 1);
    assert_eq!(result, ());
}
