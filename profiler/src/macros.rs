#[macro_export]
macro_rules! load_profiler_lib {
    () => {
        #[cfg(debug_assertions)]
        unsafe {
            use nrg_platform::*;
            use std::path::PathBuf;
            use $crate::*;

            if NRG_PROFILER_LIB.is_none() {
                let library_name = library_filename("nrg_profiler");
                let (path, filename) =
                    library::compute_folder_and_filename(PathBuf::from(library_name));
                let fullpath = path.join(filename);
                let library = Library::new(fullpath);
                NRG_PROFILER_LIB = Some(library);
            }
        }
    };
}

#[macro_export]
macro_rules! create_profiler {
    () => {
        #[cfg(debug_assertions)]
        unsafe {
            use nrg_platform::*;
            use std::path::PathBuf;
            use $crate::*;

            $crate::load_profiler_lib!();
            if let Some(create_fn) = NRG_PROFILER_LIB
                .as_ref()
                .unwrap()
                .get::<PFNCreateProfiler>(CREATE_PROFILER_FUNCTION_NAME)
            {
                unsafe { create_fn.unwrap()() };
            }
        }
    };
}

#[macro_export]
macro_rules! start_profiler {
    () => {
        #[cfg(debug_assertions)]
        unsafe {
            use nrg_platform::*;
            use std::path::PathBuf;
            use $crate::*;

            $crate::load_profiler_lib!();
            if let Some(start_fn) = NRG_PROFILER_LIB
                .as_ref()
                .unwrap()
                .get::<PFNStartProfiler>(START_PROFILER_FUNCTION_NAME)
            {
                unsafe { start_fn.unwrap()() };
            }
        }
    };
}

#[macro_export]
macro_rules! stop_profiler {
    () => {
        #[cfg(debug_assertions)]
        unsafe {
            use nrg_platform::*;
            use std::path::PathBuf;
            use $crate::*;

            $crate::load_profiler_lib!();
            if let Some(stop_fn) = NRG_PROFILER_LIB
                .as_ref()
                .unwrap()
                .get::<PFNStopProfiler>(STOP_PROFILER_FUNCTION_NAME)
            {
                unsafe { stop_fn.unwrap()() };
            }
        }
    };
}

#[macro_export]
macro_rules! register_thread_into_profiler_with_name {
    ($string:expr) => {
        #[cfg(debug_assertions)]
        unsafe {
            use nrg_platform::*;
            use std::path::PathBuf;
            use $crate::*;

            $crate::load_profiler_lib!();
            if let Some(register_thread_fn) = NRG_PROFILER_LIB
                .as_ref()
                .unwrap()
                .get::<PFNRegisterThread>(REGISTER_THREAD_FUNCTION_NAME)
            {
                unsafe { register_thread_fn.unwrap()($string.as_bytes().as_ptr()) };
            }
        }
    };
}

#[macro_export]
macro_rules! register_thread_into_profiler {
    () => {
        #[cfg(debug_assertions)]
        unsafe {
            use nrg_platform::*;
            use std::path::PathBuf;
            use $crate::*;

            $crate::load_profiler_lib!();
            if let Some(register_thread_fn) = NRG_PROFILER_LIB
                .as_ref()
                .unwrap()
                .get::<PFNRegisterThread>(REGISTER_THREAD_FUNCTION_NAME)
            {
                unsafe { register_thread_fn.unwrap()(::std::ptr::null()) };
            }
        }
    };
}

#[macro_export]
macro_rules! write_profile_file {
    () => {
        #[cfg(debug_assertions)]
        unsafe {
            use nrg_platform::*;
            use std::path::PathBuf;
            use $crate::*;

            $crate::load_profiler_lib!();
            if let Some(write_profile_fn) = NRG_PROFILER_LIB
                .as_ref()
                .unwrap()
                .get::<PFNWriteProfileFile>(WRITE_PROFILE_FILE_FUNCTION_NAME)
            {
                unsafe { write_profile_fn.unwrap()() };
            }
        }
    };
}

#[macro_export]
macro_rules! scoped_profile {
    ($string:expr) => {
        #[cfg(debug_assertions)]
        let _profile_scope = $crate::ScopedProfile::new(module_path!(), $string);
    };
}
