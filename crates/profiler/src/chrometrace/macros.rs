#[macro_export]
macro_rules! create_cpu_profiler {
    () => {
        #[cfg(not(target_arch = "wasm32"))]
        {
            $crate::GLOBAL_CPU_PROFILER.current_thread_profiler();
        }
    };
}

#[macro_export]
macro_rules! gpu_profiler_post_present {
    ($queue: expr) => {
        #[cfg(not(target_arch = "wasm32"))]
        {
            THREAD_PROFILER.with(|profiler| {
                if profiler.borrow().is_none() {
                    let thread_profiler = $crate::GLOBAL_CPU_PROFILER.current_thread_profiler();
                    *profiler.borrow_mut() = Some(thread_profiler);
                }
                let current_time = $crate::GLOBAL_CPU_PROFILER.get_elapsed_time();
                $crate::gpu_profiler::GLOBAL_GPU_PROFILER
                    .write()
                    .unwrap()
                    .end_frame(profiler.borrow().as_ref().unwrap(), $queue, current_time);
            });
        }
    };
}

#[macro_export]
macro_rules! start_profiler {
    () => {
        #[cfg(not(target_arch = "wasm32"))]
        {
            $crate::GLOBAL_CPU_PROFILER.start();
            $crate::gpu_profiler::GLOBAL_GPU_PROFILER
                .write()
                .unwrap()
                .enable(true);
        }
    };
}

#[macro_export]
macro_rules! stop_profiler {
    () => {
        #[cfg(not(target_arch = "wasm32"))]
        {
            $crate::GLOBAL_CPU_PROFILER.stop();
            $crate::gpu_profiler::GLOBAL_GPU_PROFILER
                .write()
                .unwrap()
                .enable(false);
        }
    };
}

#[macro_export]
macro_rules! register_profiler_thread {
    () => {
        #[cfg(not(target_arch = "wasm32"))]
        {
            $crate::GLOBAL_CPU_PROFILER.current_thread_profiler();
        }
    };
}

#[macro_export]
macro_rules! write_profile_file {
    () => {
        #[cfg(not(target_arch = "wasm32"))]
        {
            $crate::GLOBAL_CPU_PROFILER.write_profile_file()
        }
    };
}

#[macro_export]
macro_rules! scoped_profile {
    ($($t:tt)*) => {
        use std::thread;
        use $crate::*;

        #[cfg(not(target_arch = "wasm32"))]
        let _profile_scope = {
            if $crate::GLOBAL_CPU_PROFILER.is_started() {
                let string = format!("{}", &format_args!($($t)*).to_string());
                let scoped_profiler =
                    Box::new($crate::chrometrace::ScopedProfile::new($crate::GLOBAL_CPU_PROFILER.clone(), "CPU", string.as_str()));
                Some(scoped_profiler)
            } else {
                None
            }
        };
    };
}
