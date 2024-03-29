#![cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]

#[macro_export]
macro_rules! debug_log {
    ($($t:tt)*) => {
        (println!("[DEBUG]: {}", &format_args!($($t)*).to_string()))
    }
}
