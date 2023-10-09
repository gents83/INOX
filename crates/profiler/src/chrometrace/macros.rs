#[macro_export]
macro_rules! get_cpu_profiler {
    () => {
        #[cfg(not(target_arch = "wasm32"))]
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
        #[cfg(not(target_arch = "wasm32"))]
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
macro_rules! gpu_profiler_post_present {
    ($queue: expr) => {
        #[cfg(not(target_arch = "wasm32"))]
        unsafe {
            use $crate::gpu_profiler::*;

            use $crate::gpu_profiler::*;
            use $crate::*;

            $crate::get_cpu_profiler!();
            $crate::get_gpu_profiler!();
            if let Some(cpu_profiler) = &GLOBAL_CPU_PROFILER {
                if let Some(gpu_profiler) = &GLOBAL_GPU_PROFILER {
                    THREAD_PROFILER.with(|profiler| {
                        if profiler.borrow().is_none() {
                            let thread_profiler =
                                $crate::get_cpu_profiler().current_thread_profiler();
                            *profiler.borrow_mut() = Some(thread_profiler);
                        }
                        gpu_profiler
                            .write()
                            .unwrap()
                            .end_frame(profiler.borrow().as_ref().unwrap(), $queue);
                    });
                }
            }
        }
    };
}

#[macro_export]
macro_rules! start_profiler {
    () => {
        #[cfg(not(target_arch = "wasm32"))]
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
        #[cfg(not(target_arch = "wasm32"))]
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
        #[cfg(not(target_arch = "wasm32"))]
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
        #[cfg(not(target_arch = "wasm32"))]
        unsafe {
            use $crate::*;

            $crate::get_cpu_profiler!();

            if let Some(cpu_profiler) = &GLOBAL_CPU_PROFILER {
                cpu_profiler.write_profile_file()
            }
        }
    };
}

#[macro_export]
macro_rules! scoped_profile {
    ($($t:tt)*) => {
        use std::thread;
        use $crate::*;

        #[cfg(not(target_arch = "wasm32"))]
        $crate::get_cpu_profiler!();

        #[cfg(not(target_arch = "wasm32"))]
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
