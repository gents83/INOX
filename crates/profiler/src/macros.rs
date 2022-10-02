#[macro_export]
macro_rules! load_profiler_lib {
    () => {
        #[cfg(all(not(target_arch = "wasm32")))]
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
macro_rules! get_cpu_profiler {
    () => {
        #[cfg(all(not(target_arch = "wasm32")))]
        unsafe {
            use $crate::*;

            $crate::load_profiler_lib!();
            if GLOBAL_CPU_PROFILER.is_none() {
                if let Some(get_profiler_fn) = INOX_PROFILER_LIB
                    .as_ref()
                    .unwrap()
                    .get::<PfnGetCpuProfiler>(GET_CPU_PROFILER_FUNCTION_NAME)
                {
                    let profiler = get_profiler_fn.unwrap()();
                    GLOBAL_CPU_PROFILER.replace(profiler);
                }
            }
        }
    };
}

#[macro_export]
macro_rules! create_cpu_profiler {
    () => {
        #[cfg(all(not(target_arch = "wasm32")))]
        unsafe {
            use $crate::*;

            $crate::load_profiler_lib!();

            if let Some(create_fn) = INOX_PROFILER_LIB
                .as_ref()
                .unwrap()
                .get::<PfnCreateCpuProfiler>(CREATE_CPU_PROFILER_FUNCTION_NAME)
            {
                unsafe { create_fn.unwrap()() };
            }
        }
    };
}

#[macro_export]
macro_rules! get_gpu_profiler {
    () => {
        #[cfg(all(not(target_arch = "wasm32")))]
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
    ($device: expr, $queue: expr, $start: expr) => {
        #[cfg(all(not(target_arch = "wasm32")))]
        unsafe {
            use $crate::*;

            $crate::load_profiler_lib!();

            if let Some(create_fn) = INOX_PROFILER_LIB
                .as_ref()
                .unwrap()
                .get::<PfnCreateGpuProfiler>(CREATE_GPU_PROFILER_FUNCTION_NAME)
            {
                unsafe { create_fn.unwrap()($device, $queue, $start) };
            }
        }
    };
}

#[macro_export]
macro_rules! gpu_profiler_pre_submit {
    ($encoder: expr) => {
        #[cfg(all(not(target_arch = "wasm32")))]
        unsafe {
            use $crate::gpu_profiler::*;

            $crate::get_gpu_profiler!();
            if let Some(profiler) = &GLOBAL_GPU_PROFILER {
                profiler.write().unwrap().resolve_queries($encoder);
            }
        }
    };
}

#[macro_export]
macro_rules! gpu_profiler_post_present {
    () => {
        #[cfg(all(not(target_arch = "wasm32")))]
        unsafe {
            use $crate::gpu_profiler::*;

            use $crate::gpu_profiler::*;
            use $crate::*;

            $crate::get_cpu_profiler!();
            $crate::get_gpu_profiler!();

            if let Some(gpu_profiler) = &GLOBAL_GPU_PROFILER {
                let _ = gpu_profiler.write().unwrap().end_frame();
                if let Some(cpu_profiler) = &GLOBAL_CPU_PROFILER {
                    let mut wgpu_results = Vec::new();
                    while let Some(mut results) =
                        gpu_profiler.write().unwrap().process_finished_frame()
                    {
                        wgpu_results.append(&mut results);
                    }
                    if !wgpu_results.is_empty() {
                        let start_time = cpu_profiler.start_time() as f64;
                        THREAD_PROFILER.with(|profiler| {
                            if profiler.borrow().is_none() {
                                let thread_profiler =
                                    $crate::get_cpu_profiler().current_thread_profiler();
                                *profiler.borrow_mut() = Some(thread_profiler);
                            }
                            wgpu_results.iter().for_each(|r| {
                                profiler.borrow().as_ref().unwrap().push_sample(
                                    "GPU".to_string(),
                                    r.label.to_string(),
                                    r.cpu_time.start - start_time,
                                    r.cpu_time.end - start_time,
                                );
                            });
                        });
                    }
                }
            }
        }
    };
}

#[macro_export]
macro_rules! start_profiler {
    () => {
        #[cfg(all(not(target_arch = "wasm32")))]
        unsafe {
            use $crate::*;

            $crate::get_cpu_profiler!();

            if let Some(profiler) = &GLOBAL_CPU_PROFILER {
                profiler.start();
            }

            use $crate::gpu_profiler::*;

            $crate::get_gpu_profiler!();
            if let Some(profiler) = &GLOBAL_GPU_PROFILER {
                profiler.write().unwrap().enable(true);
            }
        }
    };
}

#[macro_export]
macro_rules! stop_profiler {
    () => {
        #[cfg(all(not(target_arch = "wasm32")))]
        unsafe {
            use $crate::*;

            $crate::get_cpu_profiler!();

            if let Some(profiler) = &GLOBAL_CPU_PROFILER {
                profiler.stop();
            }

            use $crate::gpu_profiler::*;

            $crate::get_gpu_profiler!();
            if let Some(profiler) = &GLOBAL_GPU_PROFILER {
                profiler.write().unwrap().enable(false);
            }
        }
    };
}

#[macro_export]
macro_rules! register_profiler_thread {
    () => {
        #[cfg(all(not(target_arch = "wasm32")))]
        unsafe {
            use $crate::*;

            $crate::get_cpu_profiler!();

            if let Some(profiler) = &GLOBAL_CPU_PROFILER {
                profiler.current_thread_profiler();
            }
        }
    };
}

#[macro_export]
macro_rules! write_profile_file {
    () => {
        #[cfg(all(not(target_arch = "wasm32")))]
        unsafe {
            use $crate::*;

            $crate::get_cpu_profiler!();

            if let Some(cpu_profiler) = &GLOBAL_CPU_PROFILER {
                cpu_profiler.write_profile_file()
            }
        }
    };
}
/*
($($t:tt)*) => {
        (println!("[DEBUG]: {}", &format_args!($($t)*).to_string()))
    }
     */
#[macro_export]
macro_rules! scoped_profile {
    ($($t:tt)*) => {
        use std::thread;
        use $crate::*;

        #[cfg(all(not(target_arch = "wasm32")))]
        $crate::get_cpu_profiler!();

        #[cfg(all(not(target_arch = "wasm32")))]
        let _profile_scope = if let Some(profiler) = unsafe { &GLOBAL_CPU_PROFILER } {
            if profiler.is_started() {
                let string = format!("{}", &format_args!($($t)*).to_string());
                let scoped_profiler =
                    Box::new($crate::ScopedProfile::new(profiler.clone(), "CPU", string.as_str()));
                Some(scoped_profiler)
            } else {
                None
            }
        } else {
            None
        };
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
        #[cfg(all(not(target_arch = "wasm32")))]
        unsafe {
            use $crate::gpu_profiler::*;

            $crate::get_gpu_profiler!();

            let mut gpu_profiler = GLOBAL_GPU_PROFILER.as_ref().unwrap().write().unwrap();
            let string = format!("{}", &format_args!($($t)*).to_string());
            let _scoped_profiler = Box::new($crate::scope::Scope::start(string.as_str(), &mut gpu_profiler, $encoder_or_pass, $device));
        }
    };
}
