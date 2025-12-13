#[macro_export]
macro_rules! gpu_profiler_pre_submit {
    ($encoder: expr) => {
        #[allow(unused_assignments, unused_mut)]
        #[cfg(not(target_arch = "wasm32"))]
        {
            $crate::gpu_profiler::GLOBAL_GPU_PROFILER
                .write()
                .unwrap()
                .resolve_queries($encoder);
        }
    };
}

/// Easy to use profiling scope.
///
/// Example:
/// ```ignore
/// {
///     wgpu_scoped_profiler!("name of your scope", &mut encoder, &device);
///     // wgpu commands go here
///     //i.e.: render_pass.draw(0..3, 0..1);
/// }
/// ```
#[macro_export]
macro_rules! gpu_scoped_profile {
    ($encoder_or_pass:expr, $device:expr, $($t:tt)*) => {
        #[allow(unused_assignments, unused_mut)]
        #[cfg(not(target_arch = "wasm32"))]
        {
            let gpu_profiler = $crate::gpu_profiler::GLOBAL_GPU_PROFILER.as_ref().read().unwrap();
            let string = format!("{}", &format_args!($($t)*).to_string());
            let _scoped_profiler = gpu_profiler.profile(string.as_str(), $encoder_or_pass, $device);
        }
    };
}
