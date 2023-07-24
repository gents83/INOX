#[macro_export]
macro_rules! create_cpu_profiler {
    () => {};
}

#[macro_export]
macro_rules! gpu_profiler_post_present {
    () => {};
}

#[macro_export]
macro_rules! start_profiler {
    () => {};
}

#[macro_export]
macro_rules! stop_profiler {
    () => {};
}

#[macro_export]
macro_rules! register_profiler_thread {
    () => {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let name = String::from(thread::current().name().unwrap_or("main"));
            $crate::register_profiler_thread!(&name);
        }
    };
    ($name:expr) => {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use $crate::*;

            $crate::superluminal::set_current_thread_name($name);
        }
    };
}

#[macro_export]
macro_rules! write_profile_file {
    () => {};
}

#[macro_export]
macro_rules! scoped_profile {
    ($($t:tt)*) => {
        use $crate::*;

        #[cfg(not(target_arch = "wasm32"))]
        let _profile_scope = {
                let string = format!("{}", &format_args!($($t)*).to_string());
                let scoped_profiler =
                    Box::new($crate::ScopedProfile::new("CPU", string.as_str()));
                scoped_profiler
            };
    };
}
