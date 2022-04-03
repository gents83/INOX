#[macro_export]
macro_rules! load_profiler_lib {
    () => {
        #[cfg(all(not(target_arch = "wasm32")))]
        #[cfg(debug_assertions)]
        {
            use std::path::PathBuf;
            use $crate::*;

            if SABI_PROFILER_LIB.is_none() {
                let library_name = library_filename("inox_profiler");
                let (path, filename) =
                    library::compute_folder_and_filename(PathBuf::from(library_name).as_path());
                let fullpath = path.join(filename);
                let library = Library::new(fullpath.to_str().unwrap());
                SABI_PROFILER_LIB = Some(library);
            }
        }
    };
}

#[macro_export]
macro_rules! get_profiler {
    () => {
        #[cfg(all(not(target_arch = "wasm32")))]
        #[cfg(debug_assertions)]
        unsafe {
            use $crate::*;
            $crate::load_profiler_lib!();
            if GLOBAL_PROFILER.is_none() {
                if let Some(get_profiler_fn) = SABI_PROFILER_LIB
                    .as_ref()
                    .unwrap()
                    .get::<PfnGetProfiler>(GET_PROFILER_FUNCTION_NAME)
                {
                    let profiler = get_profiler_fn.unwrap()();
                    GLOBAL_PROFILER.replace(profiler);
                }
            }
        }
    };
}

#[macro_export]
macro_rules! create_profiler {
    () => {
        #[cfg(all(not(target_arch = "wasm32")))]
        #[cfg(debug_assertions)]
        unsafe {
            use $crate::*;

            $crate::load_profiler_lib!();

            if let Some(create_fn) = SABI_PROFILER_LIB
                .as_ref()
                .unwrap()
                .get::<PfnCreateProfiler>(CREATE_PROFILER_FUNCTION_NAME)
            {
                unsafe { create_fn.unwrap()() };
            }
        }
    };
}

#[macro_export]
macro_rules! start_profiler {
    () => {
        #[cfg(all(not(target_arch = "wasm32")))]
        #[cfg(debug_assertions)]
        unsafe {
            use $crate::*;

            $crate::get_profiler!();

            if let Some(profiler) = &GLOBAL_PROFILER {
                profiler.start();
            }
        }
    };
}

#[macro_export]
macro_rules! stop_profiler {
    () => {
        #[cfg(all(not(target_arch = "wasm32")))]
        #[cfg(debug_assertions)]
        unsafe {
            use $crate::*;

            $crate::get_profiler!();

            if let Some(profiler) = &GLOBAL_PROFILER {
                profiler.stop();
            }
        }
    };
}

#[macro_export]
macro_rules! register_thread {
    () => {
        #[cfg(all(not(target_arch = "wasm32")))]
        #[cfg(debug_assertions)]
        unsafe {
            use $crate::*;

            $crate::get_profiler!();

            if let Some(profiler) = &GLOBAL_PROFILER {
                profiler.current_thread_profiler();
            }
        }
    };
}

#[macro_export]
macro_rules! write_profile_file {
    () => {
        #[cfg(all(not(target_arch = "wasm32")))]
        #[cfg(debug_assertions)]
        unsafe {
            use $crate::*;

            $crate::get_profiler!();

            if let Some(profiler) = &GLOBAL_PROFILER {
                profiler.write_profile_file()
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
        #[cfg(debug_assertions)]
        $crate::get_profiler!();

        #[cfg(all(not(target_arch = "wasm32")))]
        #[cfg(debug_assertions)]
        let _profile_scope = if let Some(profiler) = unsafe { &GLOBAL_PROFILER } {
            if profiler.is_started() {
                let string = format!("{}", &format_args!($($t)*).to_string());
                let scoped_profiler =
                    Box::new($crate::ScopedProfile::new(profiler.clone(), "", string.as_str()));
                Some(scoped_profiler)
            } else {
                None
            }
        } else {
            None
        };
    };
}
