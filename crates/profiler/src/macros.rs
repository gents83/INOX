#[macro_export]
macro_rules! load_profiler_lib {
    () => {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use inox_filesystem::*;
            use std::path::PathBuf;
            use $crate::*;

            if INOX_PROFILER_LIB.is_none() {
                let library_name = library_filename("inox_profiler");
                let (path, filename) =
                    library::compute_folder_and_filename(PathBuf::from(library_name).as_path());
                let fullpath = path.join(filename);
                let library = Library::new(fullpath.to_str().unwrap());
                INOX_PROFILER_LIB = Some(library);
            }
        }
    };
}

#[macro_export]
macro_rules! get_gpu_profiler {
    () => {
        #[cfg(not(target_arch = "wasm32"))]
        unsafe {
            use $crate::gpu_profiler::*;
            use $crate::*;

            $crate::load_profiler_lib!();
            if GLOBAL_GPU_PROFILER.is_none() {
                if let Some(get_profiler_fn) = INOX_PROFILER_LIB
                    .as_ref()
                    .unwrap()
                    .get::<PfnGetGpuProfiler>(GET_GPU_PROFILER_FUNCTION_NAME)
                {
                    let profiler = get_profiler_fn.unwrap()();
                    GLOBAL_GPU_PROFILER.replace(profiler);
                }
            }
        }
    };
}

#[macro_export]
macro_rules! create_gpu_profiler {
    () => {
        #[cfg(not(target_arch = "wasm32"))]
        unsafe {
            use $crate::*;

            $crate::load_profiler_lib!();

            if let Some(create_fn) = INOX_PROFILER_LIB
                .as_ref()
                .unwrap()
                .get::<PfnCreateGpuProfiler>(CREATE_GPU_PROFILER_FUNCTION_NAME)
            {
                unsafe { create_fn.unwrap()() };
            }
        }
    };
}

#[macro_export]
macro_rules! gpu_profiler_pre_submit {
    ($encoder: expr) => {
        #[cfg(not(target_arch = "wasm32"))]
        unsafe {
            use $crate::gpu_profiler::*;

            $crate::get_gpu_profiler!();
            if let Some(profiler) = &GLOBAL_GPU_PROFILER {
                profiler.write().unwrap().resolve_queries($encoder);
            }
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
        #[cfg(not(target_arch = "wasm32"))]
        unsafe {
            use $crate::gpu_profiler::*;

            $crate::get_gpu_profiler!();

            let gpu_profiler = GLOBAL_GPU_PROFILER.as_ref().unwrap().read().unwrap();
            let string = format!("{}", &format_args!($($t)*).to_string());
            let _scoped_profiler = gpu_profiler.profile(string.as_str(), $encoder_or_pass, $device);
        }
    };
}
