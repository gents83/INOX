
#[macro_export]
macro_rules! debug_log {
    ($($t:tt)*) => {
        (println!("[DEBUG]: {}", &format_args!($($t)*).to_string()))
    }
}
